import { useNotificationStore, type ActiveTask } from "../../stores/notificationStore";

const phaseIcons: Record<string, string> = {
  starting: "⏳",
  version_json: "📄",
  client_jar: "📦",
  libraries: "📚",
  assets: "🎨",
  loader: "⚙️",
  mod: "🧩",
  modpack: "📦",
  java: "☕",
  complete: "✅",
  launching: "🚀",
};

export function DownloadProgress() {
  const activeTasks = useNotificationStore((s) => s.activeTasks);
  const tasks: ActiveTask[] = Object.values(activeTasks);

  if (tasks.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-80">
      {tasks.map((task) => (
        <div
          key={task.id}
          className="bg-dark-800 border border-dark-600 rounded-lg p-3 shadow-xl"
        >
          <div className="flex items-center gap-2 mb-2">
            <span className="text-sm">{phaseIcons[task.phase] || "⏳"}</span>
            <span className="text-sm text-dark-200 font-medium truncate flex-1">
              {task.message}
            </span>
          </div>

          {task.percent >= 0 ? (
            <div className="h-2 bg-dark-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-primary-500 transition-all duration-300 ease-out rounded-full"
                style={{ width: `${Math.min(task.percent, 100)}%` }}
              />
            </div>
          ) : (
            <div className="h-2 bg-dark-600 rounded-full overflow-hidden">
              <div className="h-full bg-primary-500 animate-pulse rounded-full w-2/3" />
            </div>
          )}

          {task.total > 0 && (
            <div className="flex justify-between mt-1">
              <span className="text-xs text-dark-400">
                {task.current} / {task.total}
              </span>
              {task.percent >= 0 && (
                <span className="text-xs text-dark-400">{task.percent}%</span>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
