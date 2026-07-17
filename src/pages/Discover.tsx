import { useState, useCallback, useEffect } from "react";
import {
  Search,
  Download,
  Loader2,
  Globe,
  Package,
  X,
  CheckCircle,
  AlertCircle,
} from "lucide-react";
import type { ModSearchResult, InstanceListItem } from "../lib/tauri";
import { modrinthSearch, curseforgeSearch, getInstances, installMod } from "../lib/tauri";

type Source = "modrinth" | "curseforge" | "all";

interface InstallState {
  mod: ModSearchResult;
  status: "picking" | "installing" | "done" | "error";
  message?: string;
}

export function Discover() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ModSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [source, setSource] = useState<Source>("all");
  const [error, setError] = useState<string | null>(null);
  const [hasSearched, setHasSearched] = useState(false);
  const [instances, setInstances] = useState<InstanceListItem[]>([]);
  const [installState, setInstallState] = useState<InstallState | null>(null);

  // Load instances on mount
  useEffect(() => {
    getInstances().then(setInstances).catch(console.error);
  }, []);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setLoading(true);
    setError(null);
    setHasSearched(true);

    try {
      let allResults: ModSearchResult[] = [];

      if (source === "all" || source === "modrinth") {
        try {
          const mr = await modrinthSearch(query);
          allResults = [...allResults, ...mr];
        } catch (err) {
          console.error("Modrinth search error:", err);
        }
      }

      if (source === "all" || source === "curseforge") {
        try {
          const cf = await curseforgeSearch(query);
          allResults = [...allResults, ...cf];
        } catch (err) {
          const msg = String(err);
          if (msg.includes("API key")) {
            setError("CurseForge requires an API key. Add one in Settings.");
          } else {
            console.error("CurseForge search error:", err);
          }
        }
      }

      allResults.sort((a, b) => b.downloads - a.downloads);
      setResults(allResults);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [query, source]);

  const handleInstallClick = (mod: ModSearchResult) => {
    setInstallState({ mod, status: "picking" });
  };

  const handleInstallToInstance = async (instance: InstanceListItem) => {
    if (!installState) return;
    const { mod } = installState;
    setInstallState({ mod, status: "installing" });

    try {
      const loader = instance.loader || "vanilla";
      await installMod(instance.id, mod.source, mod.project_id, instance.game_version, loader);
      setInstallState({
        mod,
        status: "done",
        message: `Installed ${mod.title} to ${instance.name}`,
      });
      setTimeout(() => setInstallState(null), 3000);
    } catch (err) {
      setInstallState({
        mod,
        status: "error",
        message: String(err),
      });
    }
  };

  const formatDownloads = (n: number) => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return n.toString();
  };

  const sourceBadge = (src: string) => {
    if (src === "modrinth") {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-emerald-900/50 text-emerald-400 border border-emerald-800">
          Modrinth
        </span>
      );
    }
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-orange-900/50 text-orange-400 border border-orange-800">
        CurseForge
      </span>
    );
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Discover Mods</h1>
        <p className="text-slate-400 mt-1">
          Search and install from Modrinth &amp; CurseForge — all in one place
        </p>
      </div>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder="Search for mods, modpacks, shaders..."
            className="w-full bg-slate-800 border border-slate-600 rounded-lg pl-10 pr-4 py-2.5 text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>
        <select
          value={source}
          onChange={(e) => setSource(e.target.value as Source)}
          className="bg-slate-800 border border-slate-600 rounded-lg px-3 py-2.5 text-white text-sm focus:outline-none focus:border-blue-500"
        >
          <option value="all">All Sources</option>
          <option value="modrinth">Modrinth</option>
          <option value="curseforge">CurseForge</option>
        </select>
        <button
          onClick={handleSearch}
          disabled={loading || !query.trim()}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-5 py-2.5 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
        >
          {loading ? <Loader2 size={16} className="animate-spin" /> : <Search size={16} />}
          Search
        </button>
      </div>

      {error && (
        <div className="bg-red-900/30 border border-red-800 rounded-lg p-3 text-red-300 text-sm">
          {error}
        </div>
      )}

      {!hasSearched && results.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-500">
          <Globe size={48} className="mb-4 text-slate-600" />
          <p className="text-lg font-medium text-slate-400">Search for mods</p>
          <p className="text-sm mt-1">Find mods from Modrinth and CurseForge in one place</p>
        </div>
      ) : results.length === 0 && !loading ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-500">
          <Package size={48} className="mb-4 text-slate-600" />
          <p className="text-lg font-medium text-slate-400">No results found</p>
          <p className="text-sm mt-1">Try different keywords or switch sources</p>
        </div>
      ) : (
        <div className="space-y-3">
          {results.map((mod) => (
            <div
              key={`${mod.source}-${mod.project_id}`}
              className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors"
            >
              <div className="flex items-start gap-4">
                {mod.icon_url ? (
                  <img
                    src={mod.icon_url}
                    alt={mod.title}
                    className="w-12 h-12 rounded-lg object-cover bg-slate-700"
                  />
                ) : (
                  <div className="w-12 h-12 rounded-lg bg-slate-700 flex items-center justify-center">
                    <Package size={24} className="text-slate-500" />
                  </div>
                )}

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <h3 className="text-white font-semibold">{mod.title}</h3>
                    {sourceBadge(mod.source)}
                  </div>
                  <p className="text-sm text-slate-400 mt-1 line-clamp-2">{mod.description}</p>
                  <div className="flex items-center gap-4 mt-2">
                    <span className="flex items-center gap-1 text-xs text-slate-500">
                      <Download size={12} />
                      {formatDownloads(mod.downloads)}
                    </span>
                    {mod.categories.length > 0 && (
                      <div className="flex gap-1">
                        {mod.categories.slice(0, 3).map((cat: string) => (
                          <span
                            key={cat}
                            className="px-1.5 py-0.5 bg-slate-700 text-slate-400 rounded text-xs"
                          >
                            {cat}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>

                <button
                  onClick={() => handleInstallClick(mod)}
                  className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 shrink-0"
                >
                  <Download size={14} />
                  Install
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Instance picker modal */}
      {installState && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-slate-800 rounded-xl border border-slate-700 w-full max-w-md mx-4 p-6">
            {installState.status === "picking" && (
              <>
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-lg font-semibold text-white">
                    Install {installState.mod.title}
                  </h2>
                  <button
                    onClick={() => setInstallState(null)}
                    className="text-slate-400 hover:text-white"
                  >
                    <X size={18} />
                  </button>
                </div>
                <p className="text-sm text-slate-400 mb-4">
                  Choose an instance to install to:
                </p>
                {instances.length === 0 ? (
                  <p className="text-slate-500 text-sm">
                    No instances found. Create one first.
                  </p>
                ) : (
                  <div className="space-y-2 max-h-60 overflow-y-auto">
                    {instances.map((inst) => (
                      <button
                        key={inst.id}
                        onClick={() => handleInstallToInstance(inst)}
                        className="w-full text-left bg-slate-700 hover:bg-slate-600 rounded-lg p-3 transition-colors"
                      >
                        <div className="font-medium text-white">{inst.name}</div>
                        <div className="text-xs text-slate-400 mt-0.5">
                          MC {inst.game_version} · {inst.loader || "vanilla"}
                        </div>
                      </button>
                    ))}
                  </div>
                )}
              </>
            )}

            {installState.status === "installing" && (
              <div className="flex flex-col items-center py-8">
                <Loader2 size={32} className="animate-spin text-blue-400 mb-4" />
                <p className="text-white font-medium">Installing {installState.mod.title}...</p>
                <p className="text-sm text-slate-400 mt-1">Downloading from {installState.mod.source}</p>
              </div>
            )}

            {installState.status === "done" && (
              <div className="flex flex-col items-center py-8">
                <CheckCircle size={32} className="text-green-400 mb-4" />
                <p className="text-white font-medium">{installState.message}</p>
              </div>
            )}

            {installState.status === "error" && (
              <div className="flex flex-col items-center py-8">
                <AlertCircle size={32} className="text-red-400 mb-4" />
                <p className="text-white font-medium">Install failed</p>
                <p className="text-sm text-red-400 mt-1 text-center max-w-xs">
                  {installState.message}
                </p>
                <button
                  onClick={() => setInstallState(null)}
                  className="mt-4 bg-slate-700 hover:bg-slate-600 text-white px-4 py-2 rounded-lg text-sm"
                >
                  Close
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
