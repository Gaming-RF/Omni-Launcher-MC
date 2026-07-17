import { Play, Settings, Trash2, Clock } from "lucide-react";
import { InstanceCreator } from "../components/instance/InstanceCreator";
import { useInstancesStore } from "../stores/instances";
import { useAuthStore } from "../stores/auth";

export function Home() {
  const instances = useInstancesStore((s) => s.instances);
  const deleteInstance = useInstancesStore((s) => s.deleteInstance);
  const launchGame = useInstancesStore((s) => s.launchGame);
  const activeAccount = useAuthStore((s) => s.activeAccount);

  const formatPlayTime = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m`;
    return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">
            Welcome{activeAccount ? `, ${activeAccount.username}` : ""}
          </h1>
          <p className="text-slate-400 mt-1">
            {instances.length === 0
              ? "Create your first instance to get started"
              : `${instances.length} instance${instances.length !== 1 ? "s" : ""}`}
          </p>
        </div>
        <InstanceCreator />
      </div>

      {instances.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-slate-500">
          <div className="w-16 h-16 rounded-full bg-slate-800 flex items-center justify-center mb-4">
            <Play size={28} className="text-slate-600" />
          </div>
          <p className="text-lg font-medium text-slate-400">No instances yet</p>
          <p className="text-sm mt-1">
            Create one to start playing Minecraft
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {instances.map((instance) => (
            <div
              key={instance.id}
              className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors group"
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
                <button
                  onClick={() => deleteInstance(instance.id)}
                  className="opacity-0 group-hover:opacity-100 text-slate-500 hover:text-red-400 transition-all p-1"
                  title="Delete instance"
                >
                  <Trash2 size={16} />
                </button>
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-1 text-xs text-slate-500">
                  <Clock size={12} />
                  <span>{formatPlayTime(instance.play_time_secs)}</span>
                  <span className="mx-1">&middot;</span>
                  <span>{instance.allocated_memory_mb}MB RAM</span>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    className="text-slate-500 hover:text-slate-300 p-1"
                    title="Settings"
                  >
                    <Settings size={14} />
                  </button>
                  <button
                    onClick={() => launchGame(instance.id)}
                    className="bg-emerald-600 hover:bg-emerald-500 text-white px-3 py-1.5 rounded-lg text-sm font-medium transition-colors"
                  >
                    Play
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
