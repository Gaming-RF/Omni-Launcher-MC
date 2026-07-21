import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Package,
  Trash2,
  FolderOpen,
  Eye,
  EyeOff,
  Palette,
  Loader2,
  Sun,
} from "lucide-react";

interface ResourcePackInfo {
  name: string;
  filename: string;
  pack_type: string;
  description: string | null;
  enabled: boolean;
  size_bytes: number;
}

interface Props {
  instanceId: string;
}

type Tab = "resourcepacks" | "shaders";

export default function ResourcePacksTab({ instanceId }: Props) {
  const [activeTab, setActiveTab] = useState<Tab>("resourcepacks");
  const [packs, setPacks] = useState<ResourcePackInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const cmd =
        activeTab === "resourcepacks" ? "list_resource_packs" : "list_shaders";
      const result: ResourcePackInfo[] = await invoke(cmd, { instanceId });
      setPacks(result);
    } catch (err) {
      console.error("Failed to load packs:", err);
    } finally {
      setLoading(false);
    }
  }, [instanceId, activeTab]);

  useEffect(() => {
    load();
  }, [load]);

  const handleToggle = async (filename: string, enabled: boolean) => {
    try {
      const cmd =
        activeTab === "resourcepacks"
          ? "toggle_resource_pack"
          : "toggle_shader";
      await invoke(cmd, { instanceId, filename, enabled });
      await load();
    } catch (err) {
      console.error("Failed to toggle:", err);
    }
  };

  const handleDelete = async (filename: string) => {
    if (!confirm(`Delete "${filename}"? This cannot be undone.`)) return;
    try {
      const cmd =
        activeTab === "resourcepacks"
          ? "delete_resource_pack"
          : "delete_shader";
      await invoke(cmd, { instanceId, filename });
      await load();
    } catch (err) {
      console.error("Failed to delete:", err);
    }
  };

  const handleOpenFolder = async () => {
    try {
      const cmd =
        activeTab === "resourcepacks"
          ? "open_resource_packs_folder"
          : "open_shaders_folder";
      await invoke(cmd, { instanceId });
    } catch (err) {
      console.error("Failed to open folder:", err);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="space-y-4">
      {/* Tab switcher */}
      <div className="flex items-center gap-2">
        <button
          onClick={() => setActiveTab("resourcepacks")}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            activeTab === "resourcepacks"
              ? "bg-blue-600 text-white"
              : "bg-zinc-800 text-zinc-400 hover:text-white"
          }`}
        >
          <Package size={14} className="inline mr-1" />
          Resource Packs
        </button>
        <button
          onClick={() => setActiveTab("shaders")}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            activeTab === "shaders"
              ? "bg-purple-600 text-white"
              : "bg-zinc-800 text-zinc-400 hover:text-white"
          }`}
        >
          <Sun size={14} className="inline mr-1" />
          Shaders
        </button>
        <div className="flex-1" />
        <button
          onClick={handleOpenFolder}
          className="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-2 rounded-lg text-sm flex items-center gap-1"
        >
          <FolderOpen size={14} />
          Open Folder
        </button>
      </div>

      {/* Pack list */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 size={24} className="animate-spin text-zinc-500" />
        </div>
      ) : packs.length === 0 ? (
        <div className="text-center py-12 text-zinc-500">
          <Package size={48} className="mx-auto mb-3 opacity-30" />
          <p className="text-sm">
            No {activeTab === "resourcepacks" ? "resource packs" : "shaders"}{" "}
            installed.
          </p>
          <p className="text-xs mt-1">
            Drop {activeTab === "resourcepacks" ? ".zip files" : "shader packs"}{" "}
            into the folder or install from Discover.
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {packs.map((pack) => (
            <div
              key={pack.filename}
              className={`bg-zinc-800/50 rounded-lg p-3 flex items-center gap-3 border border-zinc-700/50 transition-opacity ${
                !pack.enabled ? "opacity-50" : ""
              }`}
            >
              {/* Icon */}
              <div
                className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                  activeTab === "shaders"
                    ? "bg-purple-600/20 text-purple-400"
                    : "bg-blue-600/20 text-blue-400"
                }`}
              >
                {activeTab === "shaders" ? (
                  <Palette size={20} />
                ) : (
                  <Package size={20} />
                )}
              </div>

              {/* Info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-white text-sm font-medium truncate">
                    {pack.name}
                  </span>
                  {!pack.enabled && (
                    <span className="text-xs bg-zinc-700 text-zinc-400 px-1.5 py-0.5 rounded">
                      Disabled
                    </span>
                  )}
                </div>
                {pack.description && (
                  <p className="text-xs text-zinc-500 truncate">
                    {pack.description}
                  </p>
                )}
                <p className="text-xs text-zinc-600">
                  {formatBytes(pack.size_bytes)}
                </p>
              </div>

              {/* Actions */}
              <div className="flex items-center gap-1">
                <button
                  onClick={() => handleToggle(pack.filename, !pack.enabled)}
                  className="p-2 rounded-lg hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors"
                  title={pack.enabled ? "Disable" : "Enable"}
                >
                  {pack.enabled ? <Eye size={16} /> : <EyeOff size={16} />}
                </button>
                <button
                  onClick={() => handleDelete(pack.filename)}
                  className="p-2 rounded-lg hover:bg-red-900/50 text-zinc-400 hover:text-red-400 transition-colors"
                  title="Delete"
                >
                  <Trash2 size={16} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
