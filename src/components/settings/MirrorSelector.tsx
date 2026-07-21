import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Globe, Zap, Check, Loader2, RefreshCw } from "lucide-react";

// ── Types ───────────────────────────────────────────────────────────────────

interface MirrorInfo {
  id: string;
  name: string;
  base_url: string;
  is_active: boolean;
  latency_ms: number | null;
}

// ── Latency badge helper ────────────────────────────────────────────────────

function latencyBadge(ms: number | null, loading: boolean) {
  if (loading) {
    return (
      <span className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-slate-700 text-slate-400">
        <Loader2 size={10} className="animate-spin" />
        Testing...
      </span>
    );
  }
  if (ms === null) {
    return (
      <span className="text-xs px-2 py-0.5 rounded-full bg-slate-700 text-slate-500">
        Untested
      </span>
    );
  }
  if (ms < 100) {
    return (
      <span className="text-xs px-2 py-0.5 rounded-full bg-emerald-900/40 text-emerald-400">
        {ms}ms
      </span>
    );
  }
  if (ms < 500) {
    return (
      <span className="text-xs px-2 py-0.5 rounded-full bg-amber-900/40 text-amber-400">
        {ms}ms
      </span>
    );
  }
  return (
    <span className="text-xs px-2 py-0.5 rounded-full bg-red-900/40 text-red-400">
      {ms}ms
    </span>
  );
}

// ── Component ───────────────────────────────────────────────────────────────

export default function MirrorSelector() {
  const [mirrors, setMirrors] = useState<MirrorInfo[]>([]);
  const [testingIds] = useState<Set<string>>(new Set());
  const [testingAll, setTestingAll] = useState(false);
  const [selecting, setSelecting] = useState<string | null>(null);

  const fetchMirrors = useCallback(async () => {
    try {
      const list: MirrorInfo[] = await invoke("list_mirrors");
      setMirrors(list);
    } catch (err) {
      console.error("Failed to fetch mirrors:", err);
    }
  }, []);

  useEffect(() => {
    fetchMirrors();
  }, [fetchMirrors]);

  const handleSelect = async (mirrorId: string) => {
    setSelecting(mirrorId);
    try {
      await invoke("set_mirror", { mirrorId });
      // Refresh list to update active states
      await fetchMirrors();
    } catch (err) {
      console.error("Failed to set mirror:", err);
    } finally {
      setSelecting(null);
    }
  };

  const handleTestAll = async () => {
    setTestingAll(true);
    try {
      const results: MirrorInfo[] = await invoke("test_all_mirrors");
      setMirrors(results);
    } catch (err) {
      console.error("Failed to test mirrors:", err);
    } finally {
      setTestingAll(false);
    }
  };

  const handleAutoSelect = async () => {
    setTestingAll(true);
    try {
      const results: MirrorInfo[] = await invoke("test_all_mirrors");
      setMirrors(results);

      // Pick the mirror with the lowest latency (exclude nulls)
      const tested = results.filter((m) => m.latency_ms !== null) as (MirrorInfo & {
        latency_ms: number;
      })[];
      if (tested.length > 0) {
        const best = tested.reduce((a, b) => (a.latency_ms <= b.latency_ms ? a : b));
        await invoke("set_mirror", { mirrorId: best.id });
        await fetchMirrors();
      }
    } catch (err) {
      console.error("Auto-select failed:", err);
    } finally {
      setTestingAll(false);
    }
  };

  const activeMirror = mirrors.find((m) => m.is_active);

  return (
    <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
      <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-1">
        <Globe size={20} />
        Download Mirror / CDN
      </h2>
      <p className="text-sm text-slate-400 mb-4">
        Choose a download source for Minecraft assets. Mirrors in your region may be
        significantly faster.
      </p>

      {/* Current mirror summary */}
      {activeMirror && (
        <div className="bg-slate-900 rounded-lg px-4 py-3 mb-4 flex items-center gap-3">
          <Zap size={16} className="text-blue-400 shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-white text-sm font-medium truncate">
              {activeMirror.name}
            </p>
            {activeMirror.base_url ? (
              <p className="text-xs text-slate-500 truncate font-mono">
                {activeMirror.base_url}
              </p>
            ) : (
              <p className="text-xs text-slate-500">Default Mojang servers</p>
            )}
          </div>
          {activeMirror.latency_ms !== null && latencyBadge(activeMirror.latency_ms, false)}
        </div>
      )}

      {/* Mirror list */}
      <div className="space-y-2 mb-4">
        {mirrors.map((mirror) => {
          const isTesting = testingIds.has(mirror.id) || testingAll;
          return (
            <label
              key={mirror.id}
              className={`flex items-center gap-3 rounded-lg p-3 cursor-pointer transition-colors ${
                mirror.is_active
                  ? "bg-blue-900/20 border border-blue-800"
                  : "bg-slate-900 border border-transparent hover:border-slate-600"
              }`}
            >
              {/* Radio */}
              <input
                type="radio"
                name="mirror"
                value={mirror.id}
                checked={mirror.is_active}
                disabled={selecting !== null}
                onChange={() => handleSelect(mirror.id)}
                className="accent-blue-500 shrink-0"
              />

              {/* Info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-white text-sm font-medium">{mirror.name}</span>
                  {mirror.is_active && (
                    <Check size={14} className="text-blue-400 shrink-0" />
                  )}
                </div>
                {mirror.base_url ? (
                  <p className="text-xs text-slate-500 truncate font-mono">
                    {mirror.base_url}
                  </p>
                ) : (
                  <p className="text-xs text-slate-500">Default Mojang servers</p>
                )}
              </div>

              {/* Latency badge */}
              {latencyBadge(mirror.latency_ms, isTesting)}
            </label>
          );
        })}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 flex-wrap">
        <button
          onClick={handleTestAll}
          disabled={testingAll}
          className="bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
        >
          {testingAll ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
          Test All
        </button>
        <button
          onClick={handleAutoSelect}
          disabled={testingAll}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
        >
          {testingAll ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <Zap size={14} />
          )}
          Auto-Select Best
        </button>
      </div>
    </section>
  );
}
