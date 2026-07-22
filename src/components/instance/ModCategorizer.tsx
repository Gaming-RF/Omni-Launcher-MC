import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Tag,
  RefreshCw,
  Loader2,
  Check,
  AlertTriangle,
  XCircle,
  HelpCircle,
} from "lucide-react";

interface ModCategory {
  category: string;
  subcategories: string[];
  description: string;
}

interface CategorizedMod {
  mod_id: string;
  name: string;
  file_name: string;
  detected_category: ModCategory;
  detected_loaders: string[];
  detected_game_versions: string[];
  compatibility: string;
}

interface Props {
  instanceId: string;
}

const CATEGORY_COLORS: Record<string, string> = {
  performance: "bg-green-600/20 text-green-400",
  optimization: "bg-emerald-600/20 text-emerald-400",
  worldgen: "bg-blue-600/20 text-blue-400",
  tech: "bg-orange-600/20 text-orange-400",
  magic: "bg-purple-600/20 text-purple-400",
  adventure: "bg-yellow-600/20 text-yellow-400",
  utility: "bg-cyan-600/20 text-cyan-400",
  library: "bg-zinc-600/20 text-zinc-400",
  qol: "bg-pink-600/20 text-pink-400",
  cosmetic: "bg-indigo-600/20 text-indigo-400",
  other: "bg-zinc-700/20 text-zinc-500",
};

const COMPAT_ICONS: Record<string, { icon: typeof Check; color: string }> = {
  compatible: { icon: Check, color: "text-green-400" },
  maybe: { icon: AlertTriangle, color: "text-yellow-400" },
  incompatible: { icon: XCircle, color: "text-red-400" },
  unknown: { icon: HelpCircle, color: "text-zinc-500" },
};

export default function ModCategorizer({ instanceId }: Props) {
  const [mods, setMods] = useState<CategorizedMod[]>([]);
  const [loading, setLoading] = useState(false);
  const [groupByCategory, setGroupByCategory] = useState(true);
  const [filterCategory, setFilterCategory] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result: CategorizedMod[] = await invoke(
        "categorize_instance_mods",
        { instanceId }
      );
      setMods(result);
    } catch (err) {
      console.error("Failed to categorize:", err);
    } finally {
      setLoading(false);
    }
  }, [instanceId]);

  useEffect(() => {
    load();
  }, [load]);

  const categories = [...new Set(mods.map((m) => m.detected_category.category))].sort();

  const filtered = filterCategory
    ? mods.filter((m) => m.detected_category.category === filterCategory)
    : mods;

  const grouped = groupByCategory
    ? categories.reduce(
        (acc, cat) => ({
          ...acc,
          [cat]: filtered.filter((m) => m.detected_category.category === cat),
        }),
        {} as Record<string, CategorizedMod[]>
      )
    : null;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Tag size={20} />
          Mod Categories
          {mods.length > 0 && (
            <span className="text-sm text-zinc-500 font-normal">
              ({mods.length} mods)
            </span>
          )}
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setGroupByCategory(!groupByCategory)}
            className={`px-3 py-1.5 rounded-lg text-xs ${
              groupByCategory
                ? "bg-blue-600/20 text-blue-400"
                : "bg-zinc-800 text-zinc-400"
            }`}
          >
            Group by category
          </button>
          <button
            onClick={load}
            disabled={loading}
            className="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-1.5 rounded-lg text-sm flex items-center gap-1"
          >
            {loading ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <RefreshCw size={14} />
            )}
            Scan
          </button>
        </div>
      </div>

      {/* Category filters */}
      <div className="flex flex-wrap gap-1.5">
        <button
          onClick={() => setFilterCategory(null)}
          className={`px-2 py-1 rounded text-xs ${
            !filterCategory
              ? "bg-blue-600 text-white"
              : "bg-zinc-800 text-zinc-400"
          }`}
        >
          All
        </button>
        {categories.map((cat) => (
          <button
            key={cat}
            onClick={() =>
              setFilterCategory(filterCategory === cat ? null : cat)
            }
            className={`px-2 py-1 rounded text-xs ${
              filterCategory === cat
                ? "bg-blue-600 text-white"
                : CATEGORY_COLORS[cat] || CATEGORY_COLORS.other
            }`}
          >
            {cat} (
            {mods.filter((m) => m.detected_category.category === cat).length})
          </button>
        ))}
      </div>

      {/* Mod list */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 size={24} className="animate-spin text-zinc-500" />
        </div>
      ) : mods.length === 0 ? (
        <div className="text-center py-12 text-zinc-500">
          <Tag size={48} className="mx-auto mb-3 opacity-30" />
          <p>No mods to categorize.</p>
        </div>
      ) : grouped ? (
        // Grouped view
        <div className="space-y-4">
          {categories
            .filter((cat) => grouped[cat]?.length > 0)
            .map((cat) => (
              <div key={cat}>
                <h4 className="text-sm font-medium text-zinc-400 mb-2 capitalize">
                  {grouped[cat][0]?.detected_category.description || cat}
                </h4>
                <div className="space-y-1">
                  {grouped[cat].map((mod) => (
                    <ModRow key={mod.mod_id} mod={mod} />
                  ))}
                </div>
              </div>
            ))}
        </div>
      ) : (
        // Flat view
        <div className="space-y-1">
          {filtered.map((mod) => (
            <ModRow key={mod.mod_id} mod={mod} />
          ))}
        </div>
      )}
    </div>
  );
}

function ModRow({ mod }: { mod: CategorizedMod }) {
  const compat = COMPAT_ICONS[mod.compatibility] || COMPAT_ICONS.unknown;
  const CompatIcon = compat.icon;
  return (
    <div className="bg-zinc-800/50 rounded-lg px-3 py-2 flex items-center gap-3 border border-zinc-700/50">
      <span
        className={`px-2 py-0.5 rounded text-xs font-medium ${
          CATEGORY_COLORS[mod.detected_category.category] ||
          CATEGORY_COLORS.other
        }`}
      >
        {mod.detected_category.category}
      </span>
      <span className="text-sm text-white flex-1 truncate">{mod.name}</span>
      {mod.detected_loaders.map((l) => (
        <span
          key={l}
          className="text-xs bg-zinc-700 text-zinc-300 px-1.5 py-0.5 rounded"
        >
          {l}
        </span>
      ))}
      <CompatIcon size={14} className={compat.color} title={mod.compatibility} />
    </div>
  );
}
