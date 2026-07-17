import { useEffect } from "react";
import { useNotificationStore } from "../../stores/notificationStore";

const typeStyles: Record<string, string> = {
  success: "border-green-500 bg-green-500/10",
  error: "border-red-500 bg-red-500/10",
  info: "border-blue-500 bg-blue-500/10",
};

const typeIcons: Record<string, string> = {
  success: "✅",
  error: "❌",
  info: "ℹ️",
};

export function ToastContainer() {
  const toasts = useNotificationStore((s) => s.toasts);
  const dismissToast = useNotificationStore((s) => s.dismissToast);

  // Auto-dismiss after 5 seconds
  useEffect(() => {
    if (toasts.length === 0) return;
    const latest = toasts[toasts.length - 1];
    const age = Date.now() - latest.createdAt;
    if (age >= 5000) {
      dismissToast(latest.id);
      return;
    }
    const timer = setTimeout(() => dismissToast(latest.id), 5000 - age);
    return () => clearTimeout(timer);
  }, [toasts, dismissToast]);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 w-96">
      {toasts.slice(-5).map((toast) => (
        <div
          key={toast.id}
          className={`border rounded-lg px-4 py-3 shadow-lg flex items-start gap-3 animate-slide-in ${typeStyles[toast.type]}`}
        >
          <span className="text-sm flex-shrink-0 mt-0.5">{typeIcons[toast.type]}</span>
          <p className="text-sm text-dark-100 flex-1">{toast.message}</p>
          <button
            onClick={() => dismissToast(toast.id)}
            className="text-dark-400 hover:text-white text-xs flex-shrink-0"
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  );
}
