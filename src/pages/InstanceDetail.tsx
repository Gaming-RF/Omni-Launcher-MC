import { useState, useEffect, useCallback } from "react";
import {
  ArrowLeft,
  Play,
  Package,
  Puzzle,
  Settings,
  Trash2,
  Power,
  PowerOff,
  Download,
  Loader2,
  Search,
  Check,
  Terminal,
} from "lucide-react";
import type {
  InstanceListItem,
  InstalledModInfo,
  ModSearchResult,
  LoaderVersionInfo,
  ModUpdateInfo,
} from "../lib/tauri";
import {
  getInstanceMods,
  installModFromModrinth,
  toggleModEnabled,
  removeMod,
  modrinthSearch,
  getFabricLoaderVersions,
  getQuiltLoaderVersions,
  getForgeVersions,
  getNeoForgeVersions,
  installFabricLoader,
  installQuiltLoader,
  installForgeLoader,
  installNeoForgeLoader,
  isInstanceRunning,
  checkModUpdates,
} from "../lib/tauri";
import { listen } from "@tauri-apps/api/event";
import { useNavigate } from "react-router-dom";
import { useInstancesStore } from "../stores/instances";
import { useI18nStore } from "../stores/i18n";
import { useActiveAccount } from "../hooks/useActiveAccount";
import { GameConsole } from "../components/instance/GameConsole";
import { PacksTab } from "../components/instance/PacksTab";

type Tab = "mods" | "resourcepacks" | "shaders" | "loader" | "settings" | "console";

interface Props {
  instance: InstanceListItem;
  onBack: () => void;
}

export function InstanceDetail({ instance, onBack }: Props) {
  const [tab, setTab] = useState<Tab>("mods");
  const launchGame = useInstancesStore((s) => s.launchGame);
  const t = useI18nStore((s) => s.t);
  const [isRunning, setIsRunning] = useState(false);

  // Check initial running state and subscribe to game events
  useEffect(() => {
    isInstanceRunning(instance.id).then(setIsRunning).catch(() => {});

    const unlistenStart = listen<{ instance_id: string }>("game-log", (event) => {
      if (event.payload.instance_id === instance.id) setIsRunning(true);
    });

    const unlistenExit = listen<{ instance_id: string; exit_code: number | null }>(
      "game-exit",
      (event) => {
        if (event.payload.instance_id === instance.id) setIsRunning(false);
      }
    );

    return () => {
      unlistenStart.then((fn) => fn());
      unlistenExit.then((fn) => fn());
    };
  }, [instance.id]);
  const { hasAccount } = useActiveAccount();
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center gap-4">
        <button
          onClick={onBack}
          className="text-slate-400 hover:text-white p-1 transition-colors"
        >
          <ArrowLeft size={20} />
        </button>
        <div className="flex-1">
          <h1 className="text-xl font-bold text-white">{instance.name}</h1>
          <p className="text-sm text-slate-400">
            {instance.game_version} &middot; {instance.loader}
            {instance.loader_version ? ` ${instance.loader_version}` : ""}
          </p>
        </div>
        {hasAccount ? (
          <button
            onClick={() => launchGame(instance.id)}
            className="bg-emerald-600 hover:bg-emerald-500 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
          >
            <Play size={16} />
            Play
          </button>
        ) : (
          <button
            onClick={() => navigate("/settings")}
            className="bg-blue-600 hover:bg-blue-500 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors"
          >
            Sign in to launch
          </button>
        )}
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b border-slate-700 overflow-x-auto">
        {(["mods", "resourcepacks", "shaders", "loader", "settings", "console"] as Tab[]).map((tabId) => (
          <button
            key={tabId}
            onClick={() => setTab(tabId)}
            className={`px-4 py-2 text-sm font-medium capitalize transition-colors relative whitespace-nowrap ${
              tab === tabId
                ? "text-white border-b-2 border-blue-500"
                : "text-slate-400 hover:text-slate-200"
            }`}
          >
            {tabId === "mods" && <Puzzle size={14} className="inline mr-1.5" />}
            {tabId === "resourcepacks" && <Package size={14} className="inline mr-1.5" />}
            {tabId === "shaders" && <Package size={14} className="inline mr-1.5" />}
            {tabId === "loader" && <Package size={14} className="inline mr-1.5" />}
            {tabId === "settings" && <Settings size={14} className="inline mr-1.5" />}
            {tabId === "console" && <Terminal size={14} className="inline mr-1.5" />}
            {tabId === "mods" ? t("instance.mods")
              : tabId === "resourcepacks" ? t("instance.resources")
              : tabId === "shaders" ? t("instance.shaders")
              : tabId === "loader" ? t("instance.loader")
              : tabId === "settings" ? t("instance.settings")
              : tabId === "console" ? t("instance.console")
              : tabId}
            {tabId === "console" && isRunning && (
              <span className="ml-1.5 inline-flex items-center gap-1 text-[10px] text-emerald-400">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse inline-block" />
                Running
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {tab === "mods" && <ModsTab instance={instance} />}
      {tab === "resourcepacks" && <PacksTab instanceId={instance.id} packType="resourcepacks" />}
      {tab === "shaders" && <PacksTab instanceId={instance.id} packType="shaderpacks" />}
      {tab === "loader" && <LoaderTab instance={instance} />}
      {tab === "settings" && <SettingsTab instance={instance} />}
      {tab === "console" && <GameConsole instanceId={instance.id} />}
    </div>
  );
}

// ── Mods Tab ───────────────────────────────────────────────────

function ModsTab({ instance }: { instance: InstanceListItem }) {
  const [mods, setMods] = useState<InstalledModInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ModSearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [updates, setUpdates] = useState<ModUpdateInfo[]>([]);
  const [checkingUpdates, setCheckingUpdates] = useState(false);

  const fetchMods = useCallback(async () => {
    setLoading(true);
    try {
      const m = await getInstanceMods(instance.id);
      setMods(m);
    } catch (err) {
      console.error("Failed to fetch mods:", err);
    }
    setLoading(false);
  }, [instance.id]);

  useEffect(() => {
    fetchMods();
  }, [fetchMods]);

  const handleCheckUpdates = async () => {
    setCheckingUpdates(true);
    try {
      const result = await checkModUpdates(instance.id);
      setUpdates(result.filter((u) => u.update_available));
    } catch (err) {
      console.error("Update check failed:", err);
    }
    setCheckingUpdates(false);
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;
    setSearching(true);
    try {
      const results = await modrinthSearch(searchQuery);
      setSearchResults(results);
    } catch (err) {
      console.error("Search failed:", err);
    }
    setSearching(false);
  };

  const handleInstall = async (projectId: string) => {
    setInstalling(projectId);
    try {
      await installModFromModrinth(
        instance.id,
        projectId,
        instance.game_version,
        instance.loader === "vanilla" ? "fabric" : instance.loader
      );
      await fetchMods();
      setSearchResults((prev) => prev.filter((r) => r.project_id !== projectId));
    } catch (err) {
      console.error("Install failed:", err);
    }
    setInstalling(null);
  };

  const handleToggle = async (modId: number) => {
    try {
      await toggleModEnabled(modId, instance.id);
      await fetchMods();
    } catch (err) {
      console.error("Toggle failed:", err);
    }
  };

  const handleRemove = async (modId: number) => {
    try {
      await removeMod(modId, instance.id);
      await fetchMods();
    } catch (err) {
      console.error("Remove failed:", err);
    }
  };

  return (
    <div className="space-y-4">
      {/* Check for updates button */}
      {mods.length > 0 && (
        <div className="flex items-center gap-3">
          <button
            onClick={handleCheckUpdates}
            disabled={checkingUpdates}
            className="bg-emerald-700 hover:bg-emerald-600 disabled:opacity-50 text-white px-3 py-1.5 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
          >
            {checkingUpdates ? (
              <Loader2 size={12} className="animate-spin" />
            ) : (
              <Download size={12} />
            )}
            Check for updates
          </button>
          {updates.length > 0 && (
            <span className="text-xs text-amber-400">
              {updates.length} update{updates.length !== 1 ? "s" : ""} available
            </span>
          )}
        </div>
      )}

      {/* Update banner */}
      {updates.length > 0 && (
        <div className="bg-amber-900/20 border border-amber-800 rounded-lg p-3 space-y-2">
          <p className="text-xs font-medium text-amber-400 mb-2">Available Updates</p>
          {updates.map((u) => (
            <div
              key={u.mod_id}
              className="flex items-center justify-between bg-slate-800/50 rounded-lg px-3 py-2"
            >
              <div className="min-w-0">
                <p className="text-sm text-white font-medium truncate">{u.mod_name}</p>
                <p className="text-xs text-slate-400">
                  {u.installed_version} → {u.latest_version}
                </p>
              </div>
              <a
                href={u.latest_file_url || "#"}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-blue-400 hover:text-blue-300 shrink-0 ml-3"
              >
                Download
              </a>
            </div>
          ))}
        </div>
      )}

      {/* Search to install */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500"
          />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder="Search Modrinth to install mods..."
            className="w-full bg-slate-800 border border-slate-600 rounded-lg pl-9 pr-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={searching}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm"
        >
          {searching ? <Loader2 size={14} className="animate-spin" /> : "Search"}
        </button>
      </div>

      {/* Search results */}
      {searchResults.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-slate-400">
            Search Results ({searchResults.length})
          </h3>
          {searchResults.map((mod) => (
            <div
              key={mod.project_id}
              className="flex items-center gap-3 bg-slate-800 rounded-lg p-3 border border-slate-700"
            >
              {mod.icon_url ? (
                <img
                  src={mod.icon_url}
                  alt=""
                  className="w-8 h-8 rounded bg-slate-700"
                />
              ) : (
                <div className="w-8 h-8 rounded bg-slate-700 flex items-center justify-center">
                  <Package size={14} className="text-slate-500" />
                </div>
              )}
              <div className="flex-1 min-w-0">
                <p className="text-white text-sm font-medium truncate">
                  {mod.title}
                </p>
                <p className="text-xs text-slate-400 truncate">
                  {mod.description}
                </p>
              </div>
              <button
                onClick={() => handleInstall(mod.project_id)}
                disabled={installing === mod.project_id}
                className="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white px-3 py-1.5 rounded text-xs font-medium flex items-center gap-1"
              >
                {installing === mod.project_id ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <Download size={12} />
                )}
                Install
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Installed mods */}
      <div>
        <h3 className="text-sm font-medium text-slate-400 mb-2">
          Installed Mods ({mods.length})
        </h3>
        {loading ? (
          <p className="text-slate-500 text-sm">Loading...</p>
        ) : mods.length === 0 ? (
          <p className="text-slate-500 text-sm">
            No mods installed. Search above to install from Modrinth.
          </p>
        ) : (
          <div className="space-y-1">
            {mods.map((mod) => (
              <div
                key={mod.id}
                className={`flex items-center gap-3 bg-slate-800 rounded-lg p-3 border border-slate-700 ${
                  !mod.enabled ? "opacity-50" : ""
                }`}
              >
                <button
                  onClick={() => handleToggle(mod.id)}
                  className={`p-1 rounded transition-colors ${
                    mod.enabled
                      ? "text-emerald-400 hover:text-emerald-300"
                      : "text-slate-600 hover:text-slate-400"
                  }`}
                  title={mod.enabled ? "Disable mod" : "Enable mod"}
                >
                  {mod.enabled ? <Power size={16} /> : <PowerOff size={16} />}
                </button>
                <div className="flex-1 min-w-0">
                  <p className="text-white text-sm font-medium truncate">
                    {mod.name}
                  </p>
                  <p className="text-xs text-slate-400">
                    {mod.version} &middot; {mod.file_name}
                  </p>
                </div>
                <span className="text-xs text-slate-500 capitalize px-2 py-0.5 bg-slate-900 rounded">
                  {mod.source}
                </span>
                <button
                  onClick={() => handleRemove(mod.id)}
                  className="text-slate-500 hover:text-red-400 p-1 transition-colors"
                  title="Remove mod"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ── Loader Tab ─────────────────────────────────────────────────

function LoaderTab({ instance }: { instance: InstanceListItem }) {
  const [selectedLoader, setSelectedLoader] = useState(instance.loader || "fabric");
  const [versions, setVersions] = useState<(LoaderVersionInfo | string)[]>([]);
  const [selectedVersion, setSelectedVersion] = useState("");
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchVersions = useCallback(async () => {
    setLoading(true);
    setVersions([]);
    setSelectedVersion("");
    try {
      let v: (LoaderVersionInfo | string)[] = [];
      switch (selectedLoader) {
        case "fabric":
          v = await getFabricLoaderVersions(instance.game_version);
          break;
        case "quilt":
          v = await getQuiltLoaderVersions(instance.game_version);
          break;
        case "forge":
          v = await getForgeVersions(instance.game_version);
          break;
        case "neoforge":
          v = await getNeoForgeVersions(instance.game_version);
          break;
      }
      setVersions(v);
      if (v.length > 0) {
        const first = typeof v[0] === "string" ? v[0] : v[0].version;
        setSelectedVersion(first as string);
      }
    } catch (err) {
      console.error("Failed to fetch loader versions:", err);
    }
    setLoading(false);
  }, [selectedLoader, instance.game_version]);

  useEffect(() => {
    fetchVersions();
  }, [fetchVersions]);

  const handleInstall = async () => {
    if (!selectedVersion) return;
    setInstalling(true);
    setError(null);
    setResult(null);
    try {
      let res: string;
      switch (selectedLoader) {
        case "fabric":
          res = await installFabricLoader(instance.id, selectedVersion);
          break;
        case "quilt":
          res = await installQuiltLoader(instance.id, selectedVersion);
          break;
        case "forge":
          res = await installForgeLoader(instance.id, selectedVersion);
          break;
        case "neoforge":
          res = await installNeoForgeLoader(instance.id, selectedVersion);
          break;
        default:
          throw new Error("Unknown loader");
      }
      setResult(res);
    } catch (err) {
      setError(String(err));
    }
    setInstalling(false);
  };

  const loaderOptions = [
    { id: "fabric", name: "Fabric", color: "text-yellow-400" },
    { id: "quilt", name: "Quilt", color: "text-purple-400" },
    { id: "forge", name: "Forge", color: "text-orange-400" },
    { id: "neoforge", name: "NeoForge", color: "text-red-400" },
  ];

  return (
    <div className="space-y-4">
      <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
        <h3 className="text-white font-medium mb-3">Install Mod Loader</h3>
        <p className="text-sm text-slate-400 mb-4">
          Install a mod loader for MC {instance.game_version}. This will download the
          necessary files and update the instance profile.
        </p>

        {/* Loader selector */}
        <div className="flex gap-2 mb-4">
          {loaderOptions.map((opt) => (
            <button
              key={opt.id}
              onClick={() => setSelectedLoader(opt.id)}
              className={`px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                selectedLoader === opt.id
                  ? "bg-blue-600 text-white"
                  : "bg-slate-700 text-slate-300 hover:bg-slate-600"
              }`}
            >
              {opt.name}
            </button>
          ))}
        </div>

        {/* Version selector */}
        <div className="flex gap-2">
          <select
            value={selectedVersion}
            onChange={(e) => setSelectedVersion(e.target.value)}
            disabled={loading || versions.length === 0}
            className="flex-1 bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 disabled:opacity-50"
          >
            {loading ? (
              <option>Loading versions...</option>
            ) : versions.length === 0 ? (
              <option>No versions available</option>
            ) : (
              versions.map((v) => {
                const ver = typeof v === "string" ? v : v.version;
                const stable = typeof v === "string" ? true : v.stable;
                return (
                  <option key={ver} value={ver}>
                    {ver}
                    {stable ? "" : " (snapshot)"}
                  </option>
                );
              })
            )}
          </select>
          <button
            onClick={handleInstall}
            disabled={installing || !selectedVersion}
            className="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2"
          >
            {installing ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Download size={14} />
            )}
            Install
          </button>
        </div>

        {result && (
          <p className="text-emerald-400 text-sm mt-3 flex items-center gap-1">
            <Check size={14} /> {result}
          </p>
        )}
        {error && (
          <p className="text-red-400 text-sm mt-3">{error}</p>
        )}

        {/* Current loader info */}
        {instance.loader !== "vanilla" && (
          <div className="mt-4 pt-4 border-t border-slate-700">
            <p className="text-sm text-slate-400">
              Current: <span className="text-white capitalize">{instance.loader}</span>
              {instance.loader_version && (
                <span className="text-slate-300"> {instance.loader_version}</span>
              )}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

// ── Settings Tab ───────────────────────────────────────────────

function SettingsTab({ instance }: { instance: InstanceListItem }) {
  const deleteInstance = useInstancesStore((s) => s.deleteInstance);
  const [name, setName] = useState(instance.name);
  const [memory, setMemory] = useState(instance.allocated_memory_mb);

  return (
    <div className="space-y-4">
      <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
        <h3 className="text-white font-medium mb-3">Instance Settings</h3>
        <div className="space-y-3">
          <div>
            <label className="block text-sm text-slate-300 mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>
          <div>
            <label className="block text-sm text-slate-300 mb-1">
              Allocated Memory: {memory}MB
            </label>
            <input
              type="range"
              min={1024}
              max={16384}
              step={512}
              value={memory}
              onChange={(e) => setMemory(Number(e.target.value))}
              className="w-full accent-blue-500"
            />
          </div>
          <div>
            <label className="block text-sm text-slate-300 mb-1">Game Version</label>
            <p className="text-white text-sm">{instance.game_version}</p>
          </div>
        </div>
      </div>

      <div className="bg-slate-800 rounded-xl p-4 border border-red-900/50">
        <h3 className="text-red-400 font-medium mb-2">Danger Zone</h3>
        <p className="text-sm text-slate-400 mb-3">
          Delete this instance. This will remove the database entry but keep files
          on disk.
        </p>
        <button
          onClick={() => deleteInstance(instance.id)}
          className="bg-red-600 hover:bg-red-500 text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2"
        >
          <Trash2 size={14} />
          Delete Instance
        </button>
      </div>
    </div>
  );
}
