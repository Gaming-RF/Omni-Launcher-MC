import { Play, Settings, Trash2 } from "lucide-react";
import type { Instance } from "../../lib/tauri";
import Button from "../common/Button";

interface Props {
  instance: Instance;
  onPlay: (id: string) => void;
  onSettings: (id: string) => void;
  onDelete: (id: string) => void;
}

export default function InstanceCard({
  instance,
  onPlay,
  onSettings,
  onDelete,
}: Props) {
  const formatPlayTime = (seconds: number) => {
    if (seconds < 60) return "< 1 min";
    if (seconds < 3600) return `${Math.floor(seconds / 60)} min`;
    return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;
  };

  const loaderLabel = instance.mod_loader
    ? `${instance.mod_loader} ${instance.mod_loader_version ?? ""}`
    : "Vanilla";

  return (
    <div className="group relative overflow-hidden rounded-xl border border-zinc-800 bg-zinc-900 transition-all hover:border-zinc-700 hover:shadow-lg">
      {/* Icon / Header */}
      <div className="flex h-32 items-center justify-center bg-gradient-to-br from-zinc-800 to-zinc-900">
        {instance.icon ? (
          <img
            src={instance.icon}
            alt=""
            className="h-16 w-16 rounded-lg object-cover"
          />
        ) : (
          <div className="flex h-16 w-16 items-center justify-center rounded-lg bg-zinc-700 text-2xl font-bold text-zinc-400">
            {instance.name.charAt(0).toUpperCase()}
          </div>
        )}
      </div>

      {/* Info */}
      <div className="p-4">
        <h3 className="truncate font-semibold">{instance.name}</h3>
        <div className="mt-1 flex items-center gap-2 text-xs text-zinc-400">
          <span className="rounded bg-zinc-800 px-1.5 py-0.5">
            MC {instance.game_version}
          </span>
          <span className="rounded bg-zinc-800 px-1.5 py-0.5">
            {loaderLabel}
          </span>
        </div>
        {instance.play_time_seconds > 0 && (
          <p className="mt-2 text-xs text-zinc-500">
            Played {formatPlayTime(instance.play_time_seconds)}
          </p>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 border-t border-zinc-800 px-4 py-3">
        <Button
          variant="primary"
          size="sm"
          className="flex-1"
          onClick={() => onPlay(instance.id)}
        >
          <Play className="h-3.5 w-3.5" />
          Play
        </Button>
        <button
          onClick={() => onSettings(instance.id)}
          className="rounded-lg p-2 text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300"
        >
          <Settings className="h-4 w-4" />
        </button>
        <button
          onClick={() => onDelete(instance.id)}
          className="rounded-lg p-2 text-zinc-500 hover:bg-red-900/50 hover:text-red-400"
        >
          <Trash2 className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}
