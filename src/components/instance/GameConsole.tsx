import { useState, useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { Terminal, Trash2, Clock, XCircle, StopCircle } from "lucide-react";
import { getGameLogs, killGame } from "../../lib/tauri";

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

  const containerRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);
  const hasLoadedHistory = useRef(false);

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

      {/* Log output */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg overflow-y-auto font-mono text-xs leading-relaxed p-3 min-h-[300px] max-h-[500px]"
      >
        {lines.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-slate-600">
            <Terminal size={24} className="mb-2" />
            <p>No console output yet</p>
            <p className="text-[11px] mt-1">
              Launch the game to see logs here
            </p>
          </div>
        ) : (
          lines.map((line, i) => (
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
