import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Palette,
  Sliders,
  RotateCcw,
  Save,
  Loader2,
  Monitor,
} from "lucide-react";

interface GraphicsSettings {
  render_distance: number;
  simulation_distance: number;
  fov: number;
  gui_scale: number;
  max_fps: number;
  vsync: boolean;
  graphics: string;
  smooth_lighting: number;
  particles: number;
  entity_shadows: boolean;
  biome_blend: number;
  clouds: string;
  fullscreen: boolean;
  mipmap_levels: number;
}

interface Props {
  instanceId: string;
}

const DEFAULTS: GraphicsSettings = {
  render_distance: 12,
  simulation_distance: 8,
  fov: 70,
  gui_scale: 3,
  max_fps: 0,
  vsync: false,
  graphics: "fancy",
  smooth_lighting: 2,
  particles: 0,
  entity_shadows: true,
  biome_blend: 2,
  clouds: "fancy",
  fullscreen: false,
  mipmap_levels: 4,
};

export default function GraphicsSettingsTab({ instanceId }: Props) {
  const [settings, setSettings] = useState<GraphicsSettings>(DEFAULTS);
  const [loading, setLoading] = useState(true);
  const [applying, setApplying] = useState(false);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    try {
      const s: GraphicsSettings = await invoke("get_graphics_settings", {
        instanceId,
      });
      setSettings(s);
    } catch {
      setSettings(DEFAULTS);
    } finally {
      setLoading(false);
    }
  }, [instanceId]);

  useEffect(() => {
    load();
  }, [load]);

  const handleSave = async () => {
    setSaved(false);
    try {
      await invoke("update_graphics_settings", { instanceId, settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save:", err);
    }
  };

  const handleApply = async () => {
    setApplying(true);
    try {
      await invoke("update_graphics_settings", { instanceId, settings });
      await invoke("apply_graphics_settings", { instanceId });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to apply:", err);
    } finally {
      setApplying(false);
    }
  };

  const handleReset = () => setSettings(DEFAULTS);

  const update = <K extends keyof GraphicsSettings>(
    key: K,
    value: GraphicsSettings[K]
  ) => {
    setSettings((s) => ({ ...s, [key]: value }));
    setSaved(false);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 size={24} className="animate-spin text-zinc-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Performance section */}
      <div>
        <h3 className="text-sm font-semibold text-zinc-400 uppercase mb-3 flex items-center gap-2">
          <Sliders size={16} />
          Performance
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Slider
            label="Render Distance"
            value={settings.render_distance}
            min={2}
            max={32}
            unit=" chunks"
            onChange={(v) => update("render_distance", v)}
          />
          <Slider
            label="Simulation Distance"
            value={settings.simulation_distance}
            min={2}
            max={32}
            unit=" chunks"
            onChange={(v) => update("simulation_distance", v)}
          />
          <Slider
            label="Max FPS"
            value={settings.max_fps}
            min={0}
            max={260}
            step={10}
            unit=""
            labelExtra={settings.max_fps === 0 ? "Unlimited" : ""}
            onChange={(v) => update("max_fps", v)}
          />
          <Select
            label="Graphics"
            value={settings.graphics}
            options={[
              { value: "fast", label: "Fast" },
              { value: "fancy", label: "Fancy" },
            ]}
            onChange={(v) => update("graphics", v)}
          />
          <Select
            label="Clouds"
            value={settings.clouds}
            options={[
              { value: "off", label: "Off" },
              { value: "fast", label: "Fast" },
              { value: "fancy", label: "Fancy" },
            ]}
            onChange={(v) => update("clouds", v)}
          />
          <Select
            label="Particles"
            value={String(settings.particles)}
            options={[
              { value: "0", label: "All" },
              { value: "1", label: "Decreased" },
              { value: "2", label: "Minimal" },
            ]}
            onChange={(v) => update("particles", Number(v))}
          />
        </div>
      </div>

      {/* Visuals section */}
      <div>
        <h3 className="text-sm font-semibold text-zinc-400 uppercase mb-3 flex items-center gap-2">
          <Monitor size={16} />
          Visuals
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Slider
            label="FOV"
            value={settings.fov}
            min={30}
            max={110}
            unit="°"
            onChange={(v) => update("fov", v)}
          />
          <Slider
            label="GUI Scale"
            value={settings.gui_scale}
            min={1}
            max={5}
            unit=""
            onChange={(v) => update("gui_scale", v)}
          />
          <Slider
            label="Mipmap Levels"
            value={settings.mipmap_levels}
            min={0}
            max={4}
            unit=""
            onChange={(v) => update("mipmap_levels", v)}
          />
          <Slider
            label="Biome Blend"
            value={settings.biome_blend}
            min={0}
            max={7}
            unit="x"
            onChange={(v) => update("biome_blend", v)}
          />
          <Select
            label="Smooth Lighting"
            value={String(settings.smooth_lighting)}
            options={[
              { value: "0", label: "Off" },
              { value: "1", label: "Minimum" },
              { value: "2", label: "Maximum" },
            ]}
            onChange={(v) => update("smooth_lighting", Number(v))}
          />
          <Toggle
            label="Entity Shadows"
            value={settings.entity_shadows}
            onChange={(v) => update("entity_shadows", v)}
          />
          <Toggle
            label="VSync"
            value={settings.vsync}
            onChange={(v) => update("vsync", v)}
          />
          <Toggle
            label="Fullscreen"
            value={settings.fullscreen}
            onChange={(v) => update("fullscreen", v)}
          />
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3 pt-2">
        <button
          onClick={handleSave}
          className="bg-zinc-700 hover:bg-zinc-600 text-white px-4 py-2 rounded-lg text-sm flex items-center gap-2"
        >
          <Save size={14} />
          Save
        </button>
        <button
          onClick={handleApply}
          disabled={applying}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm flex items-center gap-2"
        >
          {applying ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <Palette size={14} />
          )}
          Apply to options.txt
        </button>
        <button
          onClick={handleReset}
          className="text-zinc-400 hover:text-white px-3 py-2 rounded-lg text-sm flex items-center gap-1"
        >
          <RotateCcw size={14} />
          Reset
        </button>
        {saved && (
          <span className="text-green-400 text-sm">✓ Saved</span>
        )}
      </div>
    </div>
  );
}

// --- Sub-components ---

function Slider({
  label,
  value,
  min,
  max,
  step = 1,
  unit,
  labelExtra,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit: string;
  labelExtra?: string;
  onChange: (v: number) => void;
}) {
  return (
    <div className="bg-zinc-800/50 rounded-lg p-3 border border-zinc-700/50">
      <div className="flex justify-between text-sm mb-2">
        <span className="text-zinc-300">{label}</span>
        <span className="text-white font-mono">
          {value}
          {unit}
          {labelExtra && (
            <span className="text-zinc-500 ml-1">({labelExtra})</span>
          )}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-blue-500"
      />
    </div>
  );
}

function Select({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="bg-zinc-800/50 rounded-lg p-3 border border-zinc-700/50">
      <label className="text-sm text-zinc-300 mb-1 block">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-zinc-700 text-white rounded px-2 py-1.5 text-sm border border-zinc-600"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="bg-zinc-800/50 rounded-lg p-3 border border-zinc-700/50 flex items-center justify-between">
      <span className="text-sm text-zinc-300">{label}</span>
      <button
        onClick={() => onChange(!value)}
        className={`w-10 h-5 rounded-full relative transition-colors ${
          value ? "bg-blue-600" : "bg-zinc-600"
        }`}
      >
        <div
          className={`w-4 h-4 rounded-full bg-white absolute top-0.5 transition-transform ${
            value ? "translate-x-5" : "translate-x-0.5"
          }`}
        />
      </button>
    </div>
  );
}
