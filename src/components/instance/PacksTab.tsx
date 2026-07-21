import { useState, useEffect, useCallback } from "react";
import { Palette, Trash2, ToggleLeft, ToggleRight, Loader2 } from "lucide-react";
import { listInstalledPacks, togglePack, deletePack, type InstalledPackInfo } from "../../lib/tauri";

interface Props {
  instanceId: string;
  packType: "resourcepacks" | "shaderpacks";
}

export function PacksTab({ instanceId, packType }: Props) {
  const [packs, setPacks] = useState<InstalledPackInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchPacks = useCallback(async () => {
    setLoading(true);
    try {
      const p = await listInstalledPacks(instanceId, packType);
      setPacks(p);
    } catch (err) {
      console.error("Failed to fetch packs:", err);
    }
    setLoading(false);
  }, [instanceId, packType]);

  useEffect(() => {
    fetchPacks();
  }, [fetchPacks]);

  const handleToggle = async (fileName: string) => {
    try {
      await togglePack(instanceId, packType, fileName);
      await fetchPacks();
    } catch (err) {
      console.error("Toggle failed:", err);
    }
  };

  const handleDelete = async (fileName: string) => {
    try {
      await deletePack(instanceId, packType, fileName);
      await fetchPacks();
    } catch (err) {
      console.error("Delete failed:", err);
    }
  };

  const label = packType === "resourcepacks" ? "Resource Pack" : "Shader Pack";

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12 text-slate-500">
        <Loader2 size={20} className="animate-spin mr-2" />
        Loading {label.toLowerCase()}s...
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-sm text-slate-400">
          {packs.length} {label.toLowerCase()}{packs.length !== 1 ? "s" : ""} installed
        </p>
        <p className="text-xs text-slate-500">
          Place files in the <code className="bg-slate-700 px-1 rounded">{packType}/</code> folder
        </p>
      </div>

      {packs.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 text-slate-500">
          <Palette size={32} className="mb-3 text-slate-600" />
          <p className="text-sm font-medium text-slate-400">No {label.toLowerCase()}s installed</p>
          <p className="text-xs mt-1">
            Download {label.toLowerCase()}s and place them in the {packType}/ folder
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {packs.map((pack) => (
            <div
              key={pack.file_name}
              className={`flex items-center gap-3 bg-slate-800 rounded-lg p-3 border border-slate-700 ${
                !pack.enabled ? "opacity-50" : ""
              }`}
            >
              <Palette size={16} className="text-slate-400 shrink-0" />
              <div className="flex-1 min-w-0">
                <p className="text-sm text-white font-medium truncate">{pack.file_name}</p>
                <p className="text-xs text-slate-500">
                  {pack.enabled ? "Enabled" : "Disabled"}
                </p>
              </div>
              <button
                onClick={() => handleToggle(pack.file_name)}
                className="text-slate-400 hover:text-white transition-colors"
                title={pack.enabled ? "Disable" : "Enable"}
              >
                {pack.enabled ? (
                  <ToggleRight size={20} className="text-emerald-400" />
                ) : (
                  <ToggleLeft size={20} />
                )}
              </button>
              <button
                onClick={() => handleDelete(pack.file_name)}
                className="text-slate-400 hover:text-red-400 transition-colors"
                title="Delete"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
