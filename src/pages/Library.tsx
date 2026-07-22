import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Database,
  Trash2,
  Link,
  Loader2,
  Package,
  HardDrive,
} from "lucide-react";

interface LibraryItem {
  id: string;
  name: string;
  file_name: string;
  item_type: string;
  source: string | null;
  file_size: number;
  added_at: string;
  used_by: string[];
}

type FilterType = "all" | "mods" | "resourcepacks" | "shaderpacks";

export default function Library() {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<FilterType>("all");
  const [cleaning, setCleaning] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const type = filter === "all" ? null : filter;
      const result: LibraryItem[] = await invoke("list_library_items", {
        itemType: type,
      });
      setItems(result);
    } catch (err) {
      console.error("Failed to load library:", err);
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    load();
  }, [load]);

  const handleCleanup = async () => {
    setCleaning(true);
    try {
      const [removed, freed]: [number, number] = await invoke("cleanup_library");
      alert(`Removed ${removed} orphaned items, freed ${formatBytes(freed)}`);
      await load();
    } catch (err) {
      console.error("Cleanup failed:", err);
    } finally {
      setCleaning(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const totalSize = items.reduce((s, i) => s + i.file_size, 0);
  const totalSaved = items.filter((i) => i.used_by.length > 1).reduce((s, i) => s + i.file_size * (i.used_by.length - 1), 0);

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white flex items-center gap-3">
          <Database size={28} />
          Resource Library
        </h1>
        <div className="flex items-center gap-3">
          {totalSaved > 0 && (
            <span className="text-sm text-green-400 bg-green-400/10 px-3 py-1 rounded-full">
              💾 {formatBytes(totalSaved)} saved through dedup
            </span>
          )}
          <button
            onClick={handleCleanup}
            disabled={cleaning}
            className="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-2 rounded-lg text-sm flex items-center gap-1"
          >
            {cleaning ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Trash2 size={14} />
            )}
            Cleanup Orphans
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-2">
        {(["all", "mods", "resourcepacks", "shaderpacks"] as FilterType[]).map(
          (t) => (
            <button
              key={t}
              onClick={() => setFilter(t)}
              className={`px-3 py-1.5 rounded-lg text-sm ${
                filter === t
                  ? "bg-blue-600 text-white"
                  : "bg-zinc-800 text-zinc-400 hover:text-white"
              }`}
            >
              {t === "all" ? "All" : t === "resourcepacks" ? "Resource Packs" : t === "shaderpacks" ? "Shaders" : "Mods"}
            </button>
          )
        )}
        <span className="ml-auto text-sm text-zinc-500 self-center">
          {items.length} items · {formatBytes(totalSize)} total
        </span>
      </div>

      {/* List */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 size={24} className="animate-spin text-zinc-500" />
        </div>
      ) : items.length === 0 ? (
        <div className="text-center py-12 text-zinc-500">
          <Package size={48} className="mx-auto mb-3 opacity-30" />
          <p>Library is empty.</p>
          <p className="text-xs mt-1">
            Install mods to add them here. Use "Import to Library" from instance
            mods.
          </p>
        </div>
      ) : (
        <div className="space-y-1">
          {items.map((item) => (
            <div
              key={item.id}
              className="bg-zinc-800/50 rounded-lg px-4 py-2.5 flex items-center gap-3 border border-zinc-700/50"
            >
              <HardDrive size={16} className="text-zinc-500 shrink-0" />
              <div className="flex-1 min-w-0">
                <span className="text-sm text-white truncate block">
                  {item.name}
                </span>
                <span className="text-xs text-zinc-500">
                  {item.item_type} · {formatBytes(item.file_size)}
                  {item.source && ` · ${item.source}`}
                </span>
              </div>
              {item.used_by.length > 0 && (
                <span className="text-xs bg-blue-600/20 text-blue-300 px-2 py-0.5 rounded flex items-center gap-1">
                  <Link size={10} />
                  {item.used_by.length} instance{item.used_by.length > 1 ? "s" : ""}
                </span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
