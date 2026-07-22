import { useState, useCallback, useEffect } from "react";
import {
  Search,
  Download,
  Loader2,
  Package,
  X,
  CheckCircle,
  AlertCircle,
  Compass,
} from "lucide-react";
import type { ModSearchResult, InstanceListItem, ModpackSearchResult } from "../lib/tauri";
import {
  modrinthSearch,
  curseforgeSearch,
  getInstances,
  installMod,
  searchModpacksModrinth,
  searchModpacksCurseforge,
  getModpackVersionsModrinth,
  downloadAndInstallModpack,
} from "../lib/tauri";
import { useI18nStore } from "../stores/i18n";

type Source = "modrinth" | "curseforge" | "all";
type Tab = "mods" | "modpacks";

interface InstallState {
  mod: ModSearchResult;
  status: "picking" | "installing" | "done" | "error";
  message?: string;
}

interface ModpackInstallState {
  modpack: ModpackSearchResult;
  status: "confirm" | "installing" | "done" | "error";
  message?: string;
}

export function Discover() {
  const [tab, setTab] = useState<Tab>("mods");
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ModSearchResult[]>([]);
  const [modpackResults, setModpackResults] = useState<ModpackSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [source, setSource] = useState<Source>("all");
  const [error, setError] = useState<string | null>(null);
  const [hasSearched, setHasSearched] = useState(false);
  const [instances, setInstances] = useState<InstanceListItem[]>([]);
  const [installState, setInstallState] = useState<InstallState | null>(null);
  const [modpackInstallState, setModpackInstallState] = useState<ModpackInstallState | null>(null);
  const [trendingMods, setTrendingMods] = useState<ModSearchResult[]>([]);
  const [trendingModpacks, setTrendingModpacks] = useState<ModpackSearchResult[]>([]);
  const [trendingLoading, setTrendingLoading] = useState(true);
  const t = useI18nStore((s) => s.t);

  // Load instances and trending content on mount
  useEffect(() => {
    getInstances().then(setInstances).catch(console.error);

    // Fetch popular/trending mods and modpacks
    (async () => {
      setTrendingLoading(true);
      try {
        const [mrMods, mrPacks] = await Promise.allSettled([
          modrinthSearch("", 0, 20),
          searchModpacksModrinth("", 0, 20),
        ]);
        if (mrMods.status === "fulfilled") setTrendingMods(mrMods.value);
        if (mrPacks.status === "fulfilled") setTrendingModpacks(mrPacks.value);
      } catch (err) {
        console.error("Failed to load trending:", err);
      } finally {
        setTrendingLoading(false);
      }
    })();
  }, []);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setLoading(true);
    setError(null);
    setHasSearched(true);

    try {
      if (tab === "mods") {
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
      } else {
        // Modpack search
        let allModpacks: ModpackSearchResult[] = [];

        if (source === "all" || source === "modrinth") {
          try {
            const mr = await searchModpacksModrinth(query);
            allModpacks = [...allModpacks, ...mr];
          } catch (err) {
            console.error("Modrinth modpack search error:", err);
          }
        }

        if (source === "all" || source === "curseforge") {
          try {
            const cf = await searchModpacksCurseforge(query);
            allModpacks = [...allModpacks, ...cf];
          } catch (err) {
            const msg = String(err);
            if (msg.includes("API key")) {
              // Silently skip CurseForge if no API key
            } else {
              console.error("CurseForge modpack search error:", err);
            }
          }
        }

        allModpacks.sort((a, b) => b.downloads - a.downloads);
        setModpackResults(allModpacks);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [query, source, tab]);

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

  const handleModpackInstall = (modpack: ModpackSearchResult) => {
    setModpackInstallState({ modpack, status: "confirm" });
  };

  const handleModpackInstallConfirm = async () => {
    if (!modpackInstallState) return;
    const { modpack } = modpackInstallState;
    setModpackInstallState({ modpack, status: "installing" });

    try {
      // Get the download URL — for Modrinth, fetch versions and pick first; for CurseForge, use the modpack info
      let downloadUrl: string;

      if (modpack.source === "modrinth") {
        const versions = await getModpackVersionsModrinth(modpack.project_id);
        const fileUrl = versions[0]?.file_url;
        if (!fileUrl) throw new Error("No downloadable file found for this modpack");
        downloadUrl = fileUrl;
      } else {
        // CurseForge — for now we require the CF file download URL
        throw new Error(
          "CurseForge modpack direct install is not yet supported. Try Modrinth modpacks!"
        );
      }

      await downloadAndInstallModpack(downloadUrl, modpack.source, modpack.title);
      setModpackInstallState({
        modpack,
        status: "done",
        message: `${modpack.title} installed as a new instance!`,
      });
      // Refresh instances
      getInstances().then(setInstances).catch(console.error);
      setTimeout(() => setModpackInstallState(null), 4000);
    } catch (err) {
      setModpackInstallState({
        modpack,
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

  const currentResults = tab === "mods" ? results : modpackResults;
  const noResults = hasSearched && currentResults.length === 0 && !loading;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">{t("discover.title")}</h1>
        <p className="text-slate-400 mt-1">
          {t("discover.subtitle")}
        </p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-slate-800 rounded-lg p-1 w-fit">
        <button
          onClick={() => {
            setTab("mods");
            setHasSearched(false);
            setResults([]);
            setModpackResults([]);
          }}
          className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
            tab === "mods"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white"
          }`}
        >
          <Package size={14} className="inline mr-1.5 -mt-0.5" />
          {t("discover.mods")}
        </button>
        <button
          onClick={() => {
            setTab("modpacks");
            setHasSearched(false);
            setResults([]);
            setModpackResults([]);
          }}
          className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
            tab === "modpacks"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white"
          }`}
        >
          <Compass size={14} className="inline mr-1.5 -mt-0.5" />
          {t("discover.modpacks")}
        </button>
      </div>

      {/* Search bar */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder={
              tab === "mods"
                ? "Search for mods..."
                : "Search for modpacks..."
            }
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

      {/* Empty state — show trending content */}
      {!hasSearched && currentResults.length === 0 ? (
        <div className="space-y-8">
          {/* Popular Mods */}
          <div>
            <h2 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
              <Package size={18} className="text-blue-400" />
              Popular Mods
            </h2>
            {trendingLoading ? (
              <div className="flex items-center justify-center py-12 text-slate-500">
                <Loader2 size={24} className="animate-spin mr-2" /> Loading popular mods...
              </div>
            ) : trendingMods.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {trendingMods.slice(0, 8).map((mod) => (
                  <div
                    key={`trending-${mod.source}-${mod.project_id}`}
                    className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors cursor-pointer"
                    onClick={() => handleInstallClick(mod)}
                  >
                    <div className="flex items-center gap-3">
                      {mod.icon_url ? (
                        <img src={mod.icon_url} alt={mod.title} className="w-10 h-10 rounded-lg object-cover bg-slate-700" />
                      ) : (
                        <div className="w-10 h-10 rounded-lg bg-slate-700 flex items-center justify-center">
                          <Package size={20} className="text-slate-500" />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <h3 className="text-white font-medium text-sm truncate">{mod.title}</h3>
                          {sourceBadge(mod.source)}
                        </div>
                        <p className="text-xs text-slate-400 truncate">{mod.description}</p>
                      </div>
                      <span className="flex items-center gap-1 text-xs text-slate-500 shrink-0">
                        <Download size={11} />
                        {formatDownloads(mod.downloads)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-slate-500 text-sm py-4">Could not load popular mods.</p>
            )}
          </div>

          {/* Popular Modpacks */}
          <div>
            <h2 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
              <Compass size={18} className="text-emerald-400" />
              Popular Modpacks
            </h2>
            {trendingLoading ? (
              <div className="flex items-center justify-center py-12 text-slate-500">
                <Loader2 size={24} className="animate-spin mr-2" /> Loading popular modpacks...
              </div>
            ) : trendingModpacks.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {trendingModpacks.slice(0, 8).map((mp) => (
                  <div
                    key={`trending-pack-${mp.source}-${mp.project_id}`}
                    className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors cursor-pointer"
                    onClick={() => handleModpackInstall(mp)}
                  >
                    <div className="flex items-center gap-3">
                      {mp.icon_url ? (
                        <img src={mp.icon_url} alt={mp.title} className="w-10 h-10 rounded-lg object-cover bg-slate-700" />
                      ) : (
                        <div className="w-10 h-10 rounded-lg bg-slate-700 flex items-center justify-center">
                          <Compass size={20} className="text-slate-500" />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <h3 className="text-white font-medium text-sm truncate">{mp.title}</h3>
                          {sourceBadge(mp.source)}
                        </div>
                        <p className="text-xs text-slate-400 truncate">{mp.description}</p>
                      </div>
                      <span className="flex items-center gap-1 text-xs text-slate-500 shrink-0">
                        <Download size={11} />
                        {formatDownloads(mp.downloads)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-slate-500 text-sm py-4">Could not load popular modpacks.</p>
            )}
          </div>
        </div>
      ) : noResults ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-500">
          <Package size={48} className="mb-4 text-slate-600" />
          <p className="text-lg font-medium text-slate-400">{t("discover.noResults")}</p>
          <p className="text-sm mt-1">Try different keywords or switch sources</p>
        </div>
      ) : tab === "mods" ? (
        /* ── Mods list ── */
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
      ) : (
        /* ── Modpacks list ── */
        <div className="space-y-3">
          {modpackResults.map((mp) => (
            <div
              key={`${mp.source}-${mp.project_id}`}
              className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors"
            >
              <div className="flex items-start gap-4">
                {mp.icon_url ? (
                  <img
                    src={mp.icon_url}
                    alt={mp.title}
                    className="w-14 h-14 rounded-lg object-cover bg-slate-700"
                  />
                ) : (
                  <div className="w-14 h-14 rounded-lg bg-slate-700 flex items-center justify-center">
                    <Compass size={28} className="text-slate-500" />
                  </div>
                )}

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <h3 className="text-white font-semibold">{mp.title}</h3>
                    {sourceBadge(mp.source)}
                  </div>
                  <p className="text-sm text-slate-400 mt-1 line-clamp-2">{mp.description}</p>
                  <div className="flex items-center gap-4 mt-2 flex-wrap">
                    <span className="flex items-center gap-1 text-xs text-slate-500">
                      <Download size={12} />
                      {formatDownloads(mp.downloads)}
                    </span>
                    {mp.categories.length > 0 && (
                      <div className="flex gap-1 flex-wrap">
                        {mp.categories.slice(0, 4).map((cat: string) => (
                          <span
                            key={cat}
                            className="px-1.5 py-0.5 bg-slate-700 text-slate-400 rounded text-xs"
                          >
                            {cat}
                          </span>
                        ))}
                      </div>
                    )}
                    {mp.game_versions.length > 0 && (
                      <span className="text-xs text-slate-500">
                        MC {mp.game_versions[0]}
                        {mp.game_versions.length > 1 && ` +${mp.game_versions.length - 1}`}
                      </span>
                    )}
                  </div>
                </div>

                <button
                  onClick={() => handleModpackInstall(mp)}
                  className="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 shrink-0"
                >
                  <Download size={14} />
                  Install
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Instance picker modal (for mods) */}
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

      {/* Modpack install confirmation modal */}
      {modpackInstallState && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-slate-800 rounded-xl border border-slate-700 w-full max-w-md mx-4 p-6">
            {modpackInstallState.status === "confirm" && (
              <>
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-lg font-semibold text-white">
                    Install Modpack
                  </h2>
                  <button
                    onClick={() => setModpackInstallState(null)}
                    className="text-slate-400 hover:text-white"
                  >
                    <X size={18} />
                  </button>
                </div>
                <div className="flex items-center gap-3 mb-4">
                  {modpackInstallState.modpack.icon_url ? (
                    <img
                      src={modpackInstallState.modpack.icon_url}
                      alt=""
                      className="w-12 h-12 rounded-lg object-cover"
                    />
                  ) : (
                    <div className="w-12 h-12 rounded-lg bg-slate-700 flex items-center justify-center">
                      <Compass size={24} className="text-slate-500" />
                    </div>
                  )}
                  <div>
                    <p className="text-white font-medium">{modpackInstallState.modpack.title}</p>
                    <p className="text-xs text-slate-400">
                      {formatDownloads(modpackInstallState.modpack.downloads)} downloads ·{" "}
                      {sourceBadge(modpackInstallState.modpack.source)}
                    </p>
                  </div>
                </div>
                <p className="text-sm text-slate-400 mb-6">
                  This will create a new instance with all the mods and configuration from this
                  modpack. Continue?
                </p>
                <div className="flex gap-3">
                  <button
                    onClick={() => setModpackInstallState(null)}
                    className="flex-1 bg-slate-700 hover:bg-slate-600 text-white px-4 py-2.5 rounded-lg text-sm font-medium transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleModpackInstallConfirm}
                    className="flex-1 bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2.5 rounded-lg text-sm font-medium transition-colors flex items-center justify-center gap-2"
                  >
                    <Download size={14} />
                    Install
                  </button>
                </div>
              </>
            )}

            {modpackInstallState.status === "installing" && (
              <div className="flex flex-col items-center py-8">
                <Loader2 size={32} className="animate-spin text-emerald-400 mb-4" />
                <p className="text-white font-medium">
                  Installing {modpackInstallState.modpack.title}...
                </p>
                <p className="text-sm text-slate-400 mt-1">
                  Downloading, parsing, and setting up the modpack
                </p>
              </div>
            )}

            {modpackInstallState.status === "done" && (
              <div className="flex flex-col items-center py-8">
                <CheckCircle size={32} className="text-green-400 mb-4" />
                <p className="text-white font-medium">{modpackInstallState.message}</p>
                <p className="text-sm text-slate-400 mt-1">Ready to play!</p>
              </div>
            )}

            {modpackInstallState.status === "error" && (
              <div className="flex flex-col items-center py-8">
                <AlertCircle size={32} className="text-red-400 mb-4" />
                <p className="text-white font-medium">Modpack install failed</p>
                <p className="text-sm text-red-400 mt-1 text-center max-w-xs">
                  {modpackInstallState.message}
                </p>
                <button
                  onClick={() => setModpackInstallState(null)}
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
