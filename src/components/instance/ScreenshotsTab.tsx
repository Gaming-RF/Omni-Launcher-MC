import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Camera,
  Trash2,
  FolderOpen,
  X,
  ChevronLeft,
  ChevronRight,
  Loader2,
} from "lucide-react";

interface ScreenshotInfo {
  filename: string;
  path: string;
  size_bytes: number;
  created_at: string;
}

interface Props {
  instanceId: string;
}

export default function ScreenshotsTab({ instanceId }: Props) {
  const [screenshots, setScreenshots] = useState<ScreenshotInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [lightbox, setLightbox] = useState<number | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result: ScreenshotInfo[] = await invoke("list_screenshots", {
        instanceId,
      });
      setScreenshots(result);
    } catch (err) {
      console.error("Failed to load screenshots:", err);
    } finally {
      setLoading(false);
    }
  }, [instanceId]);

  useEffect(() => {
    load();
  }, [load]);

  const handleDelete = async (filename: string) => {
    if (!confirm(`Delete "${filename}"? This cannot be undone.`)) return;
    try {
      await invoke("delete_screenshot", { instanceId, filename });
      await load();
    } catch (err) {
      console.error("Failed to delete:", err);
    }
  };

  const handleOpenFolder = async () => {
    try {
      await invoke("open_screenshots_folder", { instanceId });
    } catch (err) {
      console.error("Failed to open folder:", err);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const navigateLightbox = (dir: number) => {
    if (lightbox === null) return;
    const next = lightbox + dir;
    if (next >= 0 && next < screenshots.length) {
      setLightbox(next);
    }
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Camera size={20} />
          Screenshots
          {screenshots.length > 0 && (
            <span className="text-sm text-zinc-500 font-normal">
              ({screenshots.length})
            </span>
          )}
        </h3>
        <button
          onClick={handleOpenFolder}
          className="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-2 rounded-lg text-sm flex items-center gap-1"
        >
          <FolderOpen size={14} />
          Open Folder
        </button>
      </div>

      {/* Gallery */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 size={24} className="animate-spin text-zinc-500" />
        </div>
      ) : screenshots.length === 0 ? (
        <div className="text-center py-12 text-zinc-500">
          <Camera size={48} className="mx-auto mb-3 opacity-30" />
          <p className="text-sm">No screenshots yet.</p>
          <p className="text-xs mt-1">
            Play the game and press F2 to take screenshots!
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
          {screenshots.map((ss, idx) => (
            <div
              key={ss.filename}
              className="group relative bg-zinc-800 rounded-lg overflow-hidden border border-zinc-700/50 cursor-pointer"
              onClick={() => setLightbox(idx)}
            >
              {/* Image */}
              <div className="aspect-video bg-zinc-900">
                <img
                  src={`asset://localhost/${ss.path}`}
                  alt={ss.filename}
                  className="w-full h-full object-cover"
                  loading="lazy"
                  onError={(e) => {
                    (e.target as HTMLImageElement).style.display = "none";
                  }}
                />
              </div>
              {/* Info overlay */}
              <div className="p-2">
                <p className="text-xs text-zinc-400 truncate">{ss.created_at}</p>
                <p className="text-xs text-zinc-600">
                  {formatBytes(ss.size_bytes)}
                </p>
              </div>
              {/* Delete button (hover) */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDelete(ss.filename);
                }}
                className="absolute top-2 right-2 p-1.5 bg-red-600/80 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-500"
              >
                <Trash2 size={14} className="text-white" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Lightbox */}
      {lightbox !== null && screenshots[lightbox] && (
        <div
          className="fixed inset-0 z-50 bg-black/90 flex items-center justify-center"
          onClick={() => setLightbox(null)}
        >
          <button
            onClick={() => setLightbox(null)}
            className="absolute top-4 right-4 p-2 text-white/70 hover:text-white"
          >
            <X size={24} />
          </button>

          {lightbox > 0 && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                navigateLightbox(-1);
              }}
              className="absolute left-4 p-2 text-white/70 hover:text-white"
            >
              <ChevronLeft size={32} />
            </button>
          )}

          <img
            src={`asset://localhost/${screenshots[lightbox].path}`}
            alt={screenshots[lightbox].filename}
            className="max-w-[90vw] max-h-[85vh] object-contain"
            onClick={(e) => e.stopPropagation()}
          />

          {lightbox < screenshots.length - 1 && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                navigateLightbox(1);
              }}
              className="absolute right-4 p-2 text-white/70 hover:text-white"
            >
              <ChevronRight size={32} />
            </button>
          )}

          <div className="absolute bottom-4 text-center">
            <p className="text-white text-sm">
              {screenshots[lightbox].created_at}
            </p>
            <p className="text-zinc-400 text-xs">
              {screenshots[lightbox].filename}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
