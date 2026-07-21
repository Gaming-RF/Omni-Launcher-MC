import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { Terminal, Trash2, Clock, XCircle, StopCircle, Search, AlertTriangle } from "lucide-react";
import { getGameLogs, killGame } from "../../lib/tauri";

// Common crash patterns and their suggested fixes
const CRASH_PATTERNS: { pattern: RegExp; title: string; fix: string }[] = [
  {
    pattern: /java\.lang\.OutOfMemoryError/i,
    title: "Out of Memory",
    fix: "Increase allocated RAM in instance settings (try 4096 MB or higher)",
  },
  {
    pattern: /UnsupportedClassVersionError/i,
    title: "Java Version Mismatch",
    fix: "Install a newer Java version compatible with this Minecraft version",
  },
  {
    pattern: /IncompatibleClassChangeError|NoSuchMethodError/i,
    title: "Mod Compatibility Issue",
    fix: "Remove recently added mods or check for mod updates",
  },
  {
    pattern: /ModResolutionException|Missing or unsupported mandatory dependencies/i,
    title: "Missing Mod Dependencies",
    fix: "Install the required dependency mods listed in the error",
  },
  {
    pattern: /java\.net\.(ConnectException|SocketTimeoutException)/i,
    title: "Network Connection Failed",
    fix: "Check your internet connection or try again later",
  },
  {
    pattern: /Failed to start the minecraft server/i,
    title: "Server Launch Failed",
    fix: "Check port availability and server configuration",
  },
  {
    pattern: /GLFW error.*65543|GLFW.*not initialized/i,
    title: "Graphics/GLFW Error",
    fix: "Update GPU drivers or try using a different Java version",
  },
  {
    pattern: /Exit code: -1\b/i,
    title: "Abnormal Exit",
    fix: "Check the console for specific error messages above",
  },
  {
    pattern: /Exit code: -80/i,
    title: "Insufficient Memory",
    fix: "Allocate more RAM or close other applications",
  },
  {
    pattern: /Exit code: 1\b/i,
    title: "Game Crash",
    fix: "Check the console log above for the specific error",
  },
];

function detectCrashPattern(lines: string[]): { title: string; fix: string } | null {
  for (const line of lines) {
    for (const { pattern, title, fix } of CRASH_PATTERNS) {
      if (pattern.test(line)) {
        return { title, fix };
      }
    }
  }
  return null;
}

interface GameLogEvent {
  instance_id: string;
  line: string;
  stream: string;
}

interface GameExitEvent {
  instance_id: string;
  exit_code: number | null;
}

interface Props {
  instanceId: string;
}

export function GameConsole({ instanceId }: Props) {
  const [lines, setLines] = useState<
    { text: string; stream: string; time: Date }[]
  >([]);
  const [showTimestamps, setShowTimestamps] = useState(true);
  const [exitInfo, setExitInfo] = useState<{
    code: number | null;
  } | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [searchFilter, setSearchFilter] = useState("");

  const containerRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);
  const hasLoadedHistory = useRef(false);

  // Crash detection based on log content
  const crashInfo = useMemo(() => {
    if (exitInfo && exitInfo.code !== 0) {
      return detectCrashPattern(lines.map((l) => l.text));
    }
    return null;
  }, [exitInfo, lines]);

  // Filter lines by search
  const filteredLines = useMemo(() => {
    if (!searchFilter.trim()) return lines;
    const q = searchFilter.toLowerCase();
    return lines.filter((l) => l.text.toLowerCase().includes(q));
  }, [lines, searchFilter]);

  // Detect if user scrolls away from bottom
  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
    autoScrollRef.current = atBottom;
  }, []);

  // Auto-scroll when new lines arrive
  useEffect(() => {
    if (autoScrollRef.current) {
      const el = containerRef.current;
      if (el) {
        el.scrollTop = el.scrollHeight;
      }
    }
  }, [lines]);

  // Load historical logs on mount
  useEffect(() => {
    if (hasLoadedHistory.current) return;
    hasLoadedHistory.current = true;

    getGameLogs(instanceId)
      .then((logs) => {
        if (logs.length > 0) {
          setLines(
            logs.map((text) => ({
              text,
              stream: "stdout",
              time: new Date(),
            }))
          );
        }
      })
      .catch(() => {
        // No historical logs available — that's fine
      });
  }, [instanceId]);

  // Subscribe to game-log events
  useEffect(() => {
    const unlisten = listen<GameLogEvent>("game-log", (event) => {
      if (event.payload.instance_id !== instanceId) return;
      setIsRunning(true);
      setExitInfo(null);
      setLines((prev) => [
        ...prev,
        {
          text: event.payload.line,
          stream: event.payload.stream,
          time: new Date(),
        },
      ]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [instanceId]);

  // Subscribe to game-exit events
  useEffect(() => {
    const unlisten = listen<GameExitEvent>("game-exit", (event) => {
      if (event.payload.instance_id !== instanceId) return;
      setIsRunning(false);
      setExitInfo({ code: event.payload.exit_code });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [instanceId]);

  const handleClear = () => {
    setLines([]);
    setExitInfo(null);
  };

  const handleKill = async () => {
    try {
      await killGame(instanceId);
    } catch (err) {
      console.error("Failed to kill game:", err);
    }
  };

  const formatTime = (d: Date) =>
    d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between gap-2 mb-2">
        <div className="flex items-center gap-2">
          <Terminal size={14} className="text-slate-400" />
          <span className="text-sm text-slate-400 font-medium">
            Game Console
            {lines.length > 0 && (
              <span className="text-slate-600 ml-1">({lines.length} lines)</span>
            )}
          </span>
          {isRunning && (
            <span className="flex items-center gap-1 text-xs text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded-full">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
              Running
            </span>
          )}
        </div>
        <div className="flex items-center gap-1.5">
          {isRunning && (
            <button
              onClick={handleKill}
              className="flex items-center gap-1 text-xs text-red-400 hover:text-red-300 bg-red-400/10 hover:bg-red-400/20 px-2 py-1 rounded transition-colors"
              title="Kill game process"
            >
              <StopCircle size={12} />
              Kill
            </button>
          )}
          <button
            onClick={() => setShowTimestamps((v) => !v)}
            className={`flex items-center gap-1 text-xs px-2 py-1 rounded transition-colors ${
              showTimestamps
                ? "text-blue-400 bg-blue-400/10"
                : "text-slate-500 hover:text-slate-400 bg-slate-800"
            }`}
            title="Toggle timestamps"
          >
            <Clock size={12} />
            Time
          </button>
          <button
            onClick={handleClear}
            className="flex items-center gap-1 text-xs text-slate-500 hover:text-slate-300 bg-slate-800 hover:bg-slate-700 px-2 py-1 rounded transition-colors"
            title="Clear console"
          >
            <Trash2 size={12} />
            Clear
          </button>
        </div>
      </div>

      {/* Exit code banner */}
      {exitInfo && (
        <div
          className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm mb-2 ${
            exitInfo.code === 0
              ? "bg-emerald-400/10 text-emerald-400 border border-emerald-400/20"
              : "bg-red-400/10 text-red-400 border border-red-400/20"
          }`}
        >
          <XCircle size={14} />
          Game exited with code{" "}
          <span className="font-mono font-bold">
            {exitInfo.code ?? "unknown"}
          </span>
          {exitInfo.code === 0 && (
            <span className="text-emerald-500 ml-1">(clean exit)</span>
          )}
        </div>
      )}

      {/* Crash analysis banner */}
      {crashInfo && (
        <div className="flex items-start gap-3 px-3 py-3 rounded-lg text-sm mb-2 bg-amber-400/10 text-amber-300 border border-amber-400/20">
          <AlertTriangle size={16} className="shrink-0 mt-0.5" />
          <div>
            <p className="font-semibold text-amber-200">Crash Detected: {crashInfo.title}</p>
            <p className="text-xs text-amber-400 mt-1">Suggested fix: {crashInfo.fix}</p>
          </div>
        </div>
      )}

      {/* Log search */}
      <div className="relative mb-2">
        <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500" />
        <input
          type="text"
          value={searchFilter}
          onChange={(e) => setSearchFilter(e.target.value)}
          placeholder="Filter logs..."
          className="w-full bg-zinc-900 border border-zinc-800 rounded-lg pl-8 pr-3 py-1.5 text-xs text-white font-mono focus:outline-none focus:border-zinc-600"
        />
        {searchFilter && (
          <span className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] text-slate-500">
            {filteredLines.length}/{lines.length}
          </span>
        )}
      </div>

      {/* Log output */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg overflow-y-auto font-mono text-xs leading-relaxed p-3 min-h-[300px] max-h-[500px]"
      >
        {filteredLines.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-slate-600">
            <Terminal size={24} className="mb-2" />
            <p>{searchFilter ? "No matching lines" : "No console output yet"}</p>
            {!searchFilter && (
              <p className="text-[11px] mt-1">Launch the game to see logs here</p>
            )}
          </div>
        ) : (
          filteredLines.map((line, i) => (
            <div
              key={i}
              className={`flex gap-2 ${
                line.stream === "stderr"
                  ? "text-orange-400"
                  : "text-zinc-300"
              }`}
            >
              {showTimestamps && (
                <span className="text-slate-600 select-none shrink-0">
                  {formatTime(line.time)}
                </span>
              )}
              <span className="whitespace-pre-wrap break-all">
                {line.text}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
