import { useState, useEffect, useCallback } from "react";
import {
  Layout,
  Plus,
  Trash2,
  Zap,
  Gamepad2,
  Cog,
  Sparkles,
  Package,
  Save,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import Modal from "../components/common/Modal";
import type { InstanceListItem } from "../lib/tauri";
import { useInstancesStore } from "../stores/instances";

// ── Types ────────────────────────────────────────────────────────

interface TemplateMod {
  name: string;
  slug: string;
  source: string;
  project_id: string;
  description: string;
}

interface TemplateInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
  game_version: string;
  loader: string;
  loader_version: string | null;
  mods: TemplateMod[];
  is_custom: boolean;
  category: string;
}

type Category = "all" | "vanilla" | "performance" | "modded" | "custom";

const CATEGORIES: { key: Category; label: string; icon: typeof Layout }[] = [
  { key: "all", label: "All", icon: Layout },
  { key: "vanilla", label: "Vanilla", icon: Gamepad2 },
  { key: "performance", label: "Performance", icon: Zap },
  { key: "modded", label: "Modded", icon: Cog },
  { key: "custom", label: "Custom", icon: Sparkles },
];

const LOADER_COLORS: Record<string, string> = {
  vanilla: "bg-zinc-700 text-zinc-200",
  fabric: "bg-violet-900/60 text-violet-300",
  forge: "bg-orange-900/60 text-orange-300",
  neoforge: "bg-red-900/60 text-red-300",
  quilt: "bg-pink-900/60 text-pink-300",
};

// ── Page ─────────────────────────────────────────────────────────

export function Templates() {
  const [templates, setTemplates] = useState<TemplateInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeCategory, setActiveCategory] = useState<Category>("all");
  const [selectedTemplate, setSelectedTemplate] = useState<TemplateInfo | null>(null);
  const [instanceName, setInstanceName] = useState("");
  const [creating, setCreating] = useState(false);

  // Save-as-template state
  const [showSaveModal, setShowSaveModal] = useState(false);
  const [saveInstance, setSaveInstance] = useState<InstanceListItem | null>(null);
  const [saveName, setSaveName] = useState("");
  const [saveDesc, setSaveDesc] = useState("");
  const [saving, setSaving] = useState(false);

  const navigate = useNavigate();
  const instances = useInstancesStore((s) => s.instances);
  const refreshInstances = useInstancesStore((s) => s.fetchInstances);

  const loadTemplates = useCallback(async () => {
    setLoading(true);
    try {
      const result: TemplateInfo[] = await invoke("list_templates");
      setTemplates(result);
    } catch (e) {
      console.error("Failed to load templates:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTemplates();
  }, [loadTemplates]);

  // ── Filtered list ──────────────────────────────────────────────

  const filtered =
    activeCategory === "all"
      ? templates
      : templates.filter((t) => t.category === activeCategory);

  // ── Create instance from template ──────────────────────────────

  const handleCreate = async () => {
    if (!selectedTemplate || !instanceName.trim()) return;
    setCreating(true);
    try {
      const result: InstanceListItem = await invoke(
        "create_instance_from_template",
        {
          templateId: selectedTemplate.id,
          name: instanceName.trim(),
          gameVersion: null,
        }
      );
      setSelectedTemplate(null);
      setInstanceName("");
      await refreshInstances();
      navigate(`/instance/${result.id}`);
    } catch (e) {
      console.error("Failed to create instance:", e);
    } finally {
      setCreating(false);
    }
  };

  // ── Delete custom template ─────────────────────────────────────

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_custom_template", { templateId: id });
      await loadTemplates();
    } catch (err) {
      console.error("Failed to delete template:", err);
    }
  };

  // ── Save instance as template ──────────────────────────────────

  const handleSaveAsTemplate = async () => {
    if (!saveInstance || !saveName.trim()) return;
    setSaving(true);
    try {
      await invoke("save_as_template", {
        instanceId: saveInstance.id,
        templateName: saveName.trim(),
        description: saveDesc.trim(),
      });
      setShowSaveModal(false);
      setSaveName("");
      setSaveDesc("");
      setSaveInstance(null);
      await loadTemplates();
    } catch (e) {
      console.error("Failed to save template:", e);
    } finally {
      setSaving(false);
    }
  };

  // ── Render ─────────────────────────────────────────────────────

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="flex items-center gap-2 text-2xl font-bold">
            <Package className="h-6 w-6 text-green-400" />
            Templates
          </h1>
          <p className="mt-1 text-sm text-zinc-400">
            Quick-start presets for new instances
          </p>
        </div>

        {/* Save Instance as Template */}
        {instances.length > 0 && (
          <button
            onClick={() => {
              setSaveInstance(instances[0]);
              setShowSaveModal(true);
            }}
            className="flex items-center gap-2 rounded-lg bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-200 transition hover:bg-zinc-700"
          >
            <Save className="h-4 w-4" />
            Save Instance as Template
          </button>
        )}
      </div>

      {/* Category tabs */}
      <div className="flex gap-2">
        {CATEGORIES.map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setActiveCategory(key)}
            className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition ${
              activeCategory === key
                ? "bg-green-600/20 text-green-400"
                : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
            }`}
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </button>
        ))}
      </div>

      {/* Template grid */}
      {loading ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <div
              key={i}
              className="h-44 animate-pulse rounded-xl border border-zinc-800 bg-zinc-900"
            />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-zinc-700 py-16 text-zinc-500">
          <Package className="mb-3 h-10 w-10" />
          <p className="text-sm">
            {activeCategory === "custom"
              ? "No custom templates yet. Save an instance as a template to get started."
              : "No templates in this category."}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {filtered.map((template) => (
            <TemplateCard
              key={template.id}
              template={template}
              onSelect={() => {
                setSelectedTemplate(template);
                setInstanceName(template.name);
              }}
              onDelete={
                template.is_custom
                  ? (e) => handleDelete(template.id, e)
                  : undefined
              }
            />
          ))}
        </div>
      )}

      {/* ── Create Instance Modal ─────────────────────────────────── */}
      <Modal
        isOpen={!!selectedTemplate}
        onClose={() => {
          setSelectedTemplate(null);
          setInstanceName("");
        }}
        title="Create Instance from Template"
      >
        {selectedTemplate && (
          <div className="space-y-4">
            {/* Template preview */}
            <div className="flex items-start gap-3 rounded-lg bg-zinc-800/50 p-3">
              <span className="text-2xl">{selectedTemplate.icon}</span>
              <div className="min-w-0 flex-1">
                <p className="font-medium">{selectedTemplate.name}</p>
                <p className="text-xs text-zinc-400">
                  {selectedTemplate.description}
                </p>
                <div className="mt-2 flex flex-wrap gap-1.5">
                  <span
                    className={`rounded px-1.5 py-0.5 text-xs font-medium ${
                      LOADER_COLORS[selectedTemplate.loader] ||
                      "bg-zinc-700 text-zinc-200"
                    }`}
                  >
                    {selectedTemplate.loader}
                  </span>
                  {selectedTemplate.mods.length > 0 && (
                    <span className="rounded bg-zinc-700 px-1.5 py-0.5 text-xs text-zinc-300">
                      {selectedTemplate.mods.length} mod
                      {selectedTemplate.mods.length !== 1 ? "s" : ""}
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Mods list */}
            {selectedTemplate.mods.length > 0 && (
              <div className="space-y-1">
                <p className="text-xs font-medium uppercase tracking-wider text-zinc-500">
                  Included mods
                </p>
                {selectedTemplate.mods.map((m) => (
                  <div
                    key={m.slug}
                    className="flex items-center justify-between rounded bg-zinc-800/50 px-3 py-1.5 text-sm"
                  >
                    <span>{m.name}</span>
                    <span className="text-xs text-zinc-500">{m.source}</span>
                  </div>
                ))}
              </div>
            )}

            {/* Instance name input */}
            <div>
              <label className="mb-1 block text-sm font-medium text-zinc-300">
                Instance Name
              </label>
              <input
                type="text"
                value={instanceName}
                onChange={(e) => setInstanceName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                placeholder="My Instance"
                autoFocus
                className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 outline-none focus:border-green-500"
              />
            </div>

            {/* Actions */}
            <div className="flex justify-end gap-2 pt-2">
              <button
                onClick={() => {
                  setSelectedTemplate(null);
                  setInstanceName("");
                }}
                className="rounded-lg px-4 py-2 text-sm text-zinc-400 transition hover:bg-zinc-800 hover:text-zinc-200"
              >
                Cancel
              </button>
              <button
                onClick={handleCreate}
                disabled={!instanceName.trim() || creating}
                className="flex items-center gap-2 rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-green-500 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {creating ? (
                  <>
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white" />
                    Creating…
                  </>
                ) : (
                  <>
                    <Plus className="h-4 w-4" />
                    Create Instance
                  </>
                )}
              </button>
            </div>
          </div>
        )}
      </Modal>

      {/* ── Save as Template Modal ────────────────────────────────── */}
      <Modal
        isOpen={showSaveModal}
        onClose={() => {
          setShowSaveModal(false);
          setSaveInstance(null);
          setSaveName("");
          setSaveDesc("");
        }}
        title="Save Instance as Template"
      >
        <div className="space-y-4">
          {/* Instance picker */}
          {instances.length > 1 && (
            <div>
              <label className="mb-1 block text-sm font-medium text-zinc-300">
                Select Instance
              </label>
              <select
                value={saveInstance?.id || ""}
                onChange={(e) => {
                  const inst = instances.find((i) => i.id === e.target.value);
                  if (inst) setSaveInstance(inst);
                }}
                className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-green-500"
              >
                {instances.map((inst) => (
                  <option key={inst.id} value={inst.id}>
                    {inst.icon || "📦"} {inst.name} ({inst.loader} {inst.game_version})
                  </option>
                ))}
              </select>
            </div>
          )}

          {/* Template name */}
          <div>
            <label className="mb-1 block text-sm font-medium text-zinc-300">
              Template Name
            </label>
            <input
              type="text"
              value={saveName}
              onChange={(e) => setSaveName(e.target.value)}
              placeholder="My Custom Template"
              autoFocus
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 outline-none focus:border-green-500"
            />
          </div>

          {/* Description */}
          <div>
            <label className="mb-1 block text-sm font-medium text-zinc-300">
              Description
            </label>
            <textarea
              value={saveDesc}
              onChange={(e) => setSaveDesc(e.target.value)}
              rows={3}
              placeholder="Describe what this template includes…"
              className="w-full resize-none rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 outline-none focus:border-green-500"
            />
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-2 pt-2">
            <button
              onClick={() => {
                setShowSaveModal(false);
                setSaveInstance(null);
                setSaveName("");
                setSaveDesc("");
              }}
              className="rounded-lg px-4 py-2 text-sm text-zinc-400 transition hover:bg-zinc-800 hover:text-zinc-200"
            >
              Cancel
            </button>
            <button
              onClick={handleSaveAsTemplate}
              disabled={!saveName.trim() || saving}
              className="flex items-center gap-2 rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-green-500 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {saving ? (
                <>
                  <div className="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white" />
                  Saving…
                </>
              ) : (
                <>
                  <Save className="h-4 w-4" />
                  Save Template
                </>
              )}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

// ── Template Card ────────────────────────────────────────────────

function TemplateCard({
  template,
  onSelect,
  onDelete,
}: {
  template: TemplateInfo;
  onSelect: () => void;
  onDelete?: (e: React.MouseEvent) => void;
}) {
  return (
    <button
      onClick={onSelect}
      className="group relative flex flex-col rounded-xl border border-zinc-800 bg-zinc-900 p-4 text-left transition hover:border-zinc-600 hover:bg-zinc-800/80"
    >
      {/* Delete button for custom templates */}
      {onDelete && (
        <button
          onClick={onDelete}
          title="Delete template"
          className="absolute right-3 top-3 rounded-md p-1 text-zinc-600 opacity-0 transition hover:bg-red-900/40 hover:text-red-400 group-hover:opacity-100"
        >
          <Trash2 className="h-4 w-4" />
        </button>
      )}

      <div className="flex items-start gap-3">
        <span className="text-2xl">{template.icon}</span>
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-semibold">{template.name}</h3>
          <p className="mt-0.5 line-clamp-2 text-xs text-zinc-400">
            {template.description}
          </p>
        </div>
      </div>

      <div className="mt-auto flex items-center gap-2 pt-4">
        <span
          className={`rounded px-1.5 py-0.5 text-xs font-medium ${
            LOADER_COLORS[template.loader] || "bg-zinc-700 text-zinc-200"
          }`}
        >
          {template.loader}
        </span>
        {template.mods.length > 0 && (
          <span className="rounded bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-400">
            {template.mods.length} mod{template.mods.length !== 1 ? "s" : ""}
          </span>
        )}
        {template.is_custom && (
          <span className="rounded bg-green-900/40 px-1.5 py-0.5 text-xs text-green-400">
            custom
          </span>
        )}
      </div>
    </button>
  );
}
