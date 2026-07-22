import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, Square, ChevronUp, ChevronDown, Loader2 } from "lucide-react";

interface RunningInstanceInfo {
  instance_id: string;
  instance_name: string;
  is_running: boolean;
}

export default function RunningInstancesBar() {
  const [instances, setInstances] = useState<RunningInstanceInfo[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [stopping, setStopping] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const result: RunningInstanceInfo[] = await invoke(
        "get_all_running_instances"
      );
      setInstances(result);
    } catch {
      // silently fail
    }
  }, []);

  useEffect(() => {
    load();
    const interval = setInterval(load, 3000);
    return () => clearInterval(interval);
  }, [load]);

  const handleStop = async (id: string) => {
    setStopping(id);
    try {
      await invoke("terminate_instance", { instanceId: id });
      await load();
    } catch (err) {
      console.error("Failed to stop:", err);
    } finally {
      setStopping(null);
    }
  };

  const handleStopAll = async () => {
    try {
      await invoke("terminate_all_instances");
      await load();
    } catch (err) {
      console.error("Failed to stop all:", err);
    }
  };

  if (instances.length === 0) return null;

  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 bg-zinc-900/95 border-t border-zinc-700/50 backdrop-blur">
      {/* Compact bar */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-4 py-2 hover:bg-zinc-800/50"
      >
        <div className="flex items-center gap-2">
          <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
          <span className="text-sm text-white font-medium">
            {instances.length} instance{instances.length > 1 ? "s" : ""} running
          </span>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleStopAll();
            }}
            className="text-xs text-red-400 hover:text-red-300 px-2 py-1 rounded hover:bg-red-900/30"
          >
            Stop All
          </button>
          {expanded ? <ChevronDown size={16} /> : <ChevronUp size={16} />}
        </div>
      </button>

      {/* Expanded list */}
      {expanded && (
        <div className="border-t border-zinc-800 max-h-40 overflow-y-auto">
          {instances.map((inst) => (
            <div
              key={inst.instance_id}
              className="flex items-center gap-3 px-4 py-2 hover:bg-zinc-800/30"
            >
              <Play size={14} className="text-green-400" />
              <span className="text-sm text-zinc-200 flex-1">
                {inst.instance_name}
              </span>
              <button
                onClick={() => handleStop(inst.instance_id)}
                disabled={stopping === inst.instance_id}
                className="text-red-400 hover:text-red-300 p-1 rounded hover:bg-red-900/30"
              >
                {stopping === inst.instance_id ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <Square size={14} />
                )}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
