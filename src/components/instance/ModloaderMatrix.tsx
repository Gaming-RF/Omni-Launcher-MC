import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Download,
  Check,
  RefreshCw,
  Loader2,
  Package,
  ChevronDown,
  ChevronRight,
} from "lucide-react";

interface LoaderVersionInfo {
  version: string;
  stable: boolean;
}

interface ModloaderMatrixEntry {
  loader: string;
  versions: LoaderVersionInfo[];
  latest_version: string | null;
  recommended_version: string | null;
  installed_version: string | null;
}

interface Props {
  instanceId?: string;
  gameVersion?: string;
}

const LOADER_COLORS: Record<string, string> = {
  fabric: "bg-green-600/20 text-green-400 border-green-600/30",
  forge: "bg-orange-600/20 text-orange-400 border-orange-600/30",
  quilt: "bg-purple-600/20 text-purple-400 border-purple-600/30",
  neoforge: "bg-red-600/20 text-red-400 border-red-600/30",
};

const LOADER_ICONS: Record<string, string> = {
  fabric: "🐑",
  forge: "🔨",
  quilt: "🧵",
  neoforge: "🔥",
};

export default function ModloaderMatrix({ instanceId, gameVersion }: Props) {
  const [matrix, setMatrix] = useState<ModloaderMatrixEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      let result: ModloaderMatrixEntry[];
      if (instanceId) {
        result = await invoke("get_instance_modloader_matrix", { instanceId });
      } else if (gameVersion) {
        result = await invoke("get_modloader_matrix", { gameVersion });
      } else {
        return;
      }
      setMatrix(result);
    } catch (err) {
      console.error("Failed to load modloader matrix:", err);
    } finally {
      setLoading(false);
    }
  }, [instanceId, gameVersion]);

  useEffect(() => {
    load();
  }, [load]);

  const handleInstall = async (loader: string, version: string) => {
    if (!instanceId) return;
    setInstalling(`${loader}-${version}`);
    try {
      const cmdMap: Record<string, string> = {
        fabric: "install_fabric_loader",
        forge: "install_forge_loader",
        quilt: "install_quilt_loader",
        neoforge: "install_neoforge_loader",
      };
      const cmd = cmdMap[loader];
      if (cmd) {
        await invoke(cmd, {
          instanceId,
          loaderVersion: version,
        });
        await load();
      }
    } catch (err) {
      console.error("Failed to install loader:", err);
    } finally {
      setInstalling(null);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 size={24} className="animate-spin text-zinc-500" />
        <span className="ml-2 text-zinc-500 text-sm">
          Loading loader versions...
        </span>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <Package size={20} />
          Modloaders
        </h3>
        <button
          onClick={load}
          className="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 px-3 py-1.5 rounded-lg text-sm flex items-center gap-1"
        >
          <RefreshCw size={14} />
          Refresh
        </button>
      </div>

      {matrix.map((entry) => (
        <div
          key={entry.loader}
          className={`border rounded-lg overflow-hidden ${
            LOADER_COLORS[entry.loader]?.split(" ").pop() || "border-zinc-700"
          }`}
        >
          {/* Header */}
          <button
            onClick={() =>
              setExpanded(expanded === entry.loader ? null : entry.loader)
            }
            className={`w-full flex items-center gap-3 p-3 ${
              LOADER_COLORS[entry.loader]?.split(" ").slice(0, 2).join(" ") ||
              "bg-zinc-800/50"
            } hover:brightness-110 transition-all`}
          >
            <span className="text-xl">
              {LOADER_ICONS[entry.loader] || "📦"}
            </span>
            <span className="font-medium capitalize text-sm">
              {entry.loader}
            </span>

            {/* Badges */}
            <div className="flex items-center gap-2 ml-auto">
              {entry.installed_version && (
                <span className="bg-green-600/30 text-green-300 px-2 py-0.5 rounded text-xs flex items-center gap-1">
                  <Check size={12} />
                  {entry.installed_version}
                </span>
              )}
              {entry.latest_version && (
                <span className="bg-zinc-700/50 text-zinc-300 px-2 py-0.5 rounded text-xs">
                  Latest: {entry.latest_version}
                </span>
              )}
              {entry.recommended_version &&
                entry.recommended_version !== entry.latest_version && (
                  <span className="bg-blue-600/30 text-blue-300 px-2 py-0.5 rounded text-xs">
                    Rec: {entry.recommended_version}
                  </span>
                )}
              <span className="text-xs text-zinc-500">
                {entry.versions.length} versions
              </span>
              {expanded === entry.loader ? (
                <ChevronDown size={16} />
              ) : (
                <ChevronRight size={16} />
              )}
            </div>
          </button>

          {/* Version list */}
          {expanded === entry.loader && (
            <div className="border-t border-zinc-700/50 max-h-60 overflow-y-auto">
              {entry.versions.length === 0 ? (
                <p className="text-zinc-500 text-sm p-3">
                  No versions available
                </p>
              ) : (
                entry.versions.map((ver) => (
                  <div
                    key={ver.version}
                    className="flex items-center gap-2 px-3 py-2 hover:bg-zinc-800/50 border-b border-zinc-800/50 last:border-0"
                  >
                    <span className="text-sm text-zinc-300 font-mono flex-1">
                      {ver.version}
                    </span>
                    {!ver.stable && (
                      <span className="text-xs bg-yellow-600/30 text-yellow-300 px-1.5 py-0.5 rounded">
                        beta
                      </span>
                    )}
                    {ver.version === entry.installed_version && (
                      <span className="text-xs bg-green-600/30 text-green-300 px-1.5 py-0.5 rounded flex items-center gap-1">
                        <Check size={10} />
                        installed
                      </span>
                    )}
                    {instanceId &&
                      ver.version !== entry.installed_version && (
                        <button
                          onClick={() =>
                            handleInstall(entry.loader, ver.version)
                          }
                          disabled={
                            installing === `${entry.loader}-${ver.version}`
                          }
                          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-2 py-1 rounded text-xs flex items-center gap-1"
                        >
                          {installing ===
                          `${entry.loader}-${ver.version}` ? (
                            <Loader2 size={12} className="animate-spin" />
                          ) : (
                            <Download size={12} />
                          )}
                          Install
                        </button>
                      )}
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
