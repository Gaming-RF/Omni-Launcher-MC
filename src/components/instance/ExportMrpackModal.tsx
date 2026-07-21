import { useState } from "react";
import {
  Package,
  Download,
  FolderOpen,
  Check,
  X,
  Loader2,
} from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { exportMrpackToPath, type MrpackExportResult } from "../../lib/tauri";

interface Props {
  instanceId: string;
  instanceName: string;
  onClose: () => void;
  onSuccess?: (result: MrpackExportResult) => void;
}

type ExportState = "idle" | "exporting" | "success" | "error";

export function ExportMrpackModal({
  instanceId,
  instanceName,
  onClose,
  onSuccess,
}: Props) {
  const [includeConfigs, setIncludeConfigs] = useState(true);
  const [includeResourcePacks, setIncludeResourcePacks] = useState(false);
  const [includeSaves, setIncludeSaves] = useState(false);
  const [exportState, setExportState] = useState<ExportState>("idle");
  const [result, setResult] = useState<MrpackExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    setExportState("exporting");
    setError(null);

    try {
      const filePath = await save({
        defaultPath: `${instanceName}.mrpack`,
        filters: [{ name: "Modrinth Modpack", extensions: ["mrpack"] }],
      });

      if (!filePath) {
        // User cancelled the save dialog
        setExportState("idle");
        return;
      }

      const res = await exportMrpackToPath(
        instanceId,
        filePath,
        includeConfigs,
        includeResourcePacks,
        includeSaves
      );

      setResult(res);
      setExportState("success");
      onSuccess?.(res);
    } catch (err) {
      setError(String(err));
      setExportState("error");
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="bg-zinc-900 border border-zinc-700 rounded-xl p-6 w-full max-w-md shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <Package className="w-5 h-5 text-amber-400" />
            <h2 className="text-lg font-semibold text-zinc-100">
              Export as .mrpack
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {exportState === "success" && result ? (
          /* ── Success state ──────────────────────────────────── */
          <div className="space-y-4">
            <div className="flex items-start gap-3 p-3 bg-emerald-950/50 border border-emerald-800 rounded-lg">
              <Check className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
              <div className="min-w-0">
                <p className="text-sm text-emerald-300 font-medium">
                  Export successful!
                </p>
                <p className="text-xs text-zinc-400 mt-1 break-all">
                  {result.path}
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3 text-sm">
              <div className="bg-zinc-800 rounded-lg p-3">
                <p className="text-zinc-500 text-xs">Files included</p>
                <p className="text-zinc-200 font-medium">{result.file_count}</p>
              </div>
              <div className="bg-zinc-800 rounded-lg p-3">
                <p className="text-zinc-500 text-xs">Total size</p>
                <p className="text-zinc-200 font-medium">
                  {formatBytes(result.total_size_bytes)}
                </p>
              </div>
            </div>

            <button
              onClick={onClose}
              className="w-full py-2 px-4 bg-zinc-700 hover:bg-zinc-600 text-zinc-200 rounded-lg transition-colors text-sm font-medium"
            >
              Done
            </button>
          </div>
        ) : (
          /* ── Options & export ───────────────────────────────── */
          <div className="space-y-4">
            <p className="text-sm text-zinc-400">
              Export{" "}
              <span className="text-zinc-200 font-medium">{instanceName}</span>{" "}
              as a Modrinth modpack file.
            </p>

            {/* Override options */}
            <div className="space-y-2">
              <p className="text-xs font-medium text-zinc-500 uppercase tracking-wider">
                Include in overrides
              </p>

              <label className="flex items-center gap-3 p-3 bg-zinc-800 rounded-lg cursor-pointer hover:bg-zinc-800/80 transition-colors">
                <input
                  type="checkbox"
                  checked={includeConfigs}
                  onChange={(e) => setIncludeConfigs(e.target.checked)}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-700 text-amber-500 focus:ring-amber-500 focus:ring-offset-0"
                />
                <FolderOpen className="w-4 h-4 text-zinc-400 flex-shrink-0" />
                <div>
                  <p className="text-sm text-zinc-200">Config files</p>
                  <p className="text-xs text-zinc-500">
                    mod configs, keybinds, options
                  </p>
                </div>
              </label>

              <label className="flex items-center gap-3 p-3 bg-zinc-800 rounded-lg cursor-pointer hover:bg-zinc-800/80 transition-colors">
                <input
                  type="checkbox"
                  checked={includeResourcePacks}
                  onChange={(e) => setIncludeResourcePacks(e.target.checked)}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-700 text-amber-500 focus:ring-amber-500 focus:ring-offset-0"
                />
                <FolderOpen className="w-4 h-4 text-zinc-400 flex-shrink-0" />
                <div>
                  <p className="text-sm text-zinc-200">
                    Resource packs &amp; shaders
                  </p>
                  <p className="text-xs text-zinc-500">
                    texture packs, shader packs
                  </p>
                </div>
              </label>

              <label className="flex items-center gap-3 p-3 bg-zinc-800 rounded-lg cursor-pointer hover:bg-zinc-800/80 transition-colors">
                <input
                  type="checkbox"
                  checked={includeSaves}
                  onChange={(e) => setIncludeSaves(e.target.checked)}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-700 text-amber-500 focus:ring-amber-500 focus:ring-offset-0"
                />
                <FolderOpen className="w-4 h-4 text-zinc-400 flex-shrink-0" />
                <div>
                  <p className="text-sm text-zinc-200">World saves</p>
                  <p className="text-xs text-zinc-500">singleplayer worlds</p>
                </div>
              </label>
            </div>

            {/* Error message */}
            {error && (
              <div className="p-3 bg-red-950/50 border border-red-800 rounded-lg">
                <p className="text-sm text-red-300">{error}</p>
              </div>
            )}

            {/* Export button */}
            <button
              onClick={handleExport}
              disabled={exportState === "exporting"}
              className="w-full flex items-center justify-center gap-2 py-2.5 px-4 bg-amber-600 hover:bg-amber-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white rounded-lg transition-colors text-sm font-medium"
            >
              {exportState === "exporting" ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Exporting...
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  Export .mrpack
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
