import { useState, useEffect, useRef, useMemo } from "react";
import {
  Trash2,
  Clock,
  ChevronRight,
  Share2,
  Download,
  X,
  Info,
  Search,
  SlidersHorizontal,
  ArrowUpDown,
  Gamepad2,
} from "lucide-react";
import { InstanceCreator } from "../components/instance/InstanceCreator";
import { InstanceDetail } from "./InstanceDetail";
import { ShareExportDialog, ShareImportDialog } from "../components/common/ShareDialog";
import { useNavigate } from "react-router-dom";
import { useInstancesStore } from "../stores/instances";
import { useActiveAccount } from "../hooks/useActiveAccount";
import type { InstanceListItem } from "../lib/tauri";

type SortOption = "last-played" | "name-asc" | "name-desc" | "newest" | "oldest";

const LOADER_FILTERS = ["All", "Vanilla", "Fabric", "Quilt", "Forge", "NeoForge"] as const;

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
  { value: "last-played", label: "Last Played" },
  { value: "name-asc", label: "Name A–Z" },
  { value: "name-desc", label: "Name Z–A" },
  { value: "newest", label: "Newest" },
  { value: "oldest", label: "Oldest" },
];

export function Home() {
  const instances = useInstancesStore((s) => s.instances);
  const deleteInstance = useInstancesStore((s) => s.deleteInstance);
  const launchGame = useInstancesStore((s) => s.launchGame);
  const { hasAccount } = useActiveAccount();
  const navigate = useNavigate();
  const [showSignInBanner, setShowSignInBanner] = useState(true);
  const [selectedInstance, setSelectedInstance] = useState<InstanceListItem | null>(null);
  const [shareInstance, setShareInstance] = useState<InstanceListItem | null>(null);
  const [showImport, setShowImport] = useState(false);

  // Search / filter / sort state
  const [searchQuery, setSearchQuery] = useState("");
  const [loaderFilter, setLoaderFilter] = useState<string>("All");
  const [sortBy, setSortBy] = useState<SortOption>("last-played");
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Listen for Ctrl+K focus-search custom event
  useEffect(() => {
    const onFocus = () => {
      searchInputRef.current?.focus();
      searchInputRef.current?.select();
    };
    window.addEventListener("focus-search", onFocus);
    return () => window.removeEventListener("focus-search", onFocus);
  }, []);

  // Listen for launch-instance custom event (launch first visible instance)
  useEffect(() => {
    const onLaunch = () => {
      if (filteredSorted.length > 0) {
        launchGame(filteredSorted[0].id);
      }
    };
    window.addEventListener("launch-instance", onLaunch);
    return () => window.removeEventListener("launch-instance", onLaunch);
  });

  // Filter and sort
  const filteredSorted = useMemo(() => {
    let result = [...instances];

    // Search filter
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter((i) => i.name.toLowerCase().includes(q));
    }

    // Loader filter
    if (loaderFilter !== "All") {
      const lf = loaderFilter.toLowerCase();
      result = result.filter((i) => i.loader === lf);
    }

    // Sort
    switch (sortBy) {
      case "last-played":
        result.sort((a, b) => {
          if (!a.last_played && !b.last_played) return 0;
          if (!a.last_played) return 1;
          if (!b.last_played) return -1;
          return new Date(b.last_played).getTime() - new Date(a.last_played).getTime();
        });
        break;
      case "name-asc":
        result.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case "name-desc":
        result.sort((a, b) => b.name.localeCompare(a.name));
        break;
      case "newest":
        result.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
        break;
      case "oldest":
        result.sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
        break;
    }

    return result;
  }, [instances, searchQuery, loaderFilter, sortBy]);

  // If an instance is selected, show the detail view
  if (selectedInstance) {
    // Find the latest version from the store (in case it was updated)
    const latest = instances.find((i) => i.id === selectedInstance.id);
    return (
      <InstanceDetail
        instance={latest || selectedInstance}
        onBack={() => setSelectedInstance(null)}
      />
    );
  }

  const formatPlayTime = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m`;
    return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
  };

  const isFiltered =
    searchQuery.trim() !== "" || loaderFilter !== "All";

  return (
    <>
    <div className="space-y-6">
      {/* Offline / no-account banner */}
      {!hasAccount && showSignInBanner && (
        <div className="flex items-center gap-3 bg-blue-600/15 border border-blue-500/30 rounded-lg px-4 py-3 text-sm">
          <Info size={18} className="text-blue-400 flex-shrink-0" />
          <span className="text-blue-200">
            Sign in with Microsoft to play Minecraft
          </span>
          <button
            onClick={() => navigate("/settings")}
            className="ml-auto bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded-md text-xs font-medium transition-colors"
          >
            Sign in
          </button>
          <button
            onClick={() => setShowSignInBanner(false)}
            className="text-blue-400 hover:text-blue-300 p-0.5 transition-colors"
          >
            <X size={14} />
          </button>
        </div>
      )}

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">
            Welcome{hasAccount ? " back" : ""}
          </h1>
          <p className="text-slate-400 mt-1">
            {instances.length === 0
              ? "Create your first instance to get started"
              : isFiltered
                ? `${filteredSorted.length} of ${instances.length} instance${instances.length !== 1 ? "s" : ""}`
                : `${instances.length} instance${instances.length !== 1 ? "s" : ""}`}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowImport(true)}
            className="bg-slate-700 hover:bg-slate-600 text-white px-3 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-1.5"
          >
            <Download size={14} />
            Import
          </button>
          <InstanceCreator />
        </div>
      </div>

      {/* Empty state: no instances at all */}
      {instances.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-500">
          <div className="w-16 h-16 rounded-full bg-slate-800 flex items-center justify-center mb-4">
            <Gamepad2 size={28} className="text-slate-600" />
          </div>
          <p className="text-lg font-medium text-slate-400">No instances yet</p>
          <p className="text-sm mt-1 mb-4">
            Create your first instance to start playing Minecraft
          </p>
          <InstanceCreator />
        </div>
      ) : (
        <>
          {/* Search / Filter / Sort bar */}
          <div className="flex flex-wrap items-center gap-2">
            {/* Search */}
            <div className="relative flex-1 min-w-[200px]">
              <Search
                size={14}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none"
              />
              <input
                ref={searchInputRef}
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search instances… (Ctrl+K)"
                className="w-full bg-slate-800 border border-slate-700 rounded-lg pl-9 pr-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 placeholder:text-slate-500"
              />
            </div>

            {/* Loader filter */}
            <div className="relative">
              <SlidersHorizontal
                size={14}
                className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none"
              />
              <select
                value={loaderFilter}
                onChange={(e) => setLoaderFilter(e.target.value)}
                className="bg-slate-800 border border-slate-700 rounded-lg pl-8 pr-8 py-2 text-white text-sm focus:outline-none focus:border-blue-500 appearance-none cursor-pointer"
              >
                {LOADER_FILTERS.map((f) => (
                  <option key={f} value={f}>
                    {f === "All" ? "All Loaders" : f}
                  </option>
                ))}
              </select>
            </div>

            {/* Sort */}
            <div className="relative">
              <ArrowUpDown
                size={14}
                className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none"
              />
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as SortOption)}
                className="bg-slate-800 border border-slate-700 rounded-lg pl-8 pr-8 py-2 text-white text-sm focus:outline-none focus:border-blue-500 appearance-none cursor-pointer"
              >
                {SORT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Instance grid or "no results" state */}
          {filteredSorted.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-slate-500">
              <Search size={24} className="mb-2 text-slate-600" />
              <p className="text-sm font-medium text-slate-400">
                No instances match your filters
              </p>
              <p className="text-xs mt-1">
                Try adjusting your search or filter criteria
              </p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredSorted.map((instance) => (
                <div
                  key={instance.id}
                  onClick={() => setSelectedInstance(instance)}
                  className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors group cursor-pointer"
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 bg-gradient-to-br from-emerald-500 to-emerald-700 rounded-lg flex items-center justify-center text-white font-bold text-lg">
                        {instance.name.charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <h3 className="text-white font-semibold">
                          {instance.name}
                        </h3>
                        <p className="text-xs text-slate-400">
                          {instance.game_version} &middot; {instance.loader}
                          {instance.loader_version
                            ? ` ${instance.loader_version}`
                            : ""}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setShareInstance(instance);
                        }}
                        className="opacity-0 group-hover:opacity-100 text-slate-500 hover:text-blue-400 transition-all p-1"
                        title="Share instance"
                      >
                        <Share2 size={16} />
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteInstance(instance.id);
                        }}
                        className="opacity-0 group-hover:opacity-100 text-slate-500 hover:text-red-400 transition-all p-1"
                        title="Delete instance"
                      >
                        <Trash2 size={16} />
                      </button>
                      <ChevronRight
                        size={16}
                        className="text-slate-600 group-hover:text-slate-400 transition-colors"
                      />
                    </div>
                  </div>

                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1 text-xs text-slate-500">
                      <Clock size={12} />
                      <span>{formatPlayTime(instance.play_time_secs)}</span>
                      <span className="mx-1">&middot;</span>
                      <span>{instance.allocated_memory_mb}MB RAM</span>
                    </div>
                    <button
                      onClick={(e) => {
                        if (!hasAccount) {
                          e.stopPropagation();
                          return;
                        }
                        e.stopPropagation();
                        launchGame(instance.id);
                      }}
                      disabled={!hasAccount}
                      title={!hasAccount ? "Sign in to play" : undefined}
                      className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                        hasAccount
                          ? "bg-emerald-600 hover:bg-emerald-500 text-white"
                          : "bg-slate-700 text-slate-500 cursor-not-allowed"
                      }`}
                    >
                      Play
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
    {/* Share dialogs */}
    {shareInstance && (
      <ShareExportDialog
        instance={shareInstance}
        isOpen={!!shareInstance}
        onClose={() => setShareInstance(null)}
      />
    )}
    <ShareImportDialog
      isOpen={showImport}
      onClose={() => setShowImport(false)}
    />
    </>
  );
}
