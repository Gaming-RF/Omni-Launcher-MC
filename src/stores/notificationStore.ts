import { create } from "zustand";
import type { ProgressEvent } from "../hooks/useDownloadProgress";

export interface ActiveTask {
  id: string;
  phase: string;
  message: string;
  current: number;
  total: number;
  percent: number; // 0-100, -1 for indeterminate
  updatedAt: number;
}

export interface Toast {
  id: string;
  type: "success" | "error" | "info";
  message: string;
  createdAt: number;
}

interface NotificationState {
  /** Active download/preparation tasks. */
  activeTasks: Record<string, ActiveTask>;
  /** Completed or errored tasks (recent, for toast display). */
  toasts: Toast[];

  /** Update a task from a progress event. */
  handleProgress: (event: ProgressEvent) => void;
  /** Show a toast notification. */
  addToast: (type: Toast["type"], message: string) => void;
  /** Dismiss a toast by ID. */
  dismissToast: (id: string) => void;
  /** Clear all completed tasks. */
  clearTasks: () => void;
}

const phaseLabels: Record<string, string> = {
  starting: "Preparing",
  version_json: "Version JSON",
  client_jar: "Client JAR",
  libraries: "Libraries",
  assets: "Assets",
  loader: "Mod Loader",
  mod: "Downloading Mod",
  modpack: "Modpack",
  java: "Java Runtime",
  complete: "Complete",
  error: "Error",
};

export const useNotificationStore = create<NotificationState>((set) => ({
  activeTasks: {},
  toasts: [],

  handleProgress: (event) => {
    set((state) => {
      const percent =
        event.total > 0 ? Math.round((event.current / event.total) * 100) : -1;

      const task: ActiveTask = {
        id: event.task_id,
        phase: event.phase,
        message: event.message || phaseLabels[event.phase] || event.phase,
        current: event.current,
        total: event.total,
        percent,
        updatedAt: Date.now(),
      };

      const newTasks = { ...state.activeTasks, [event.task_id]: task };

      // If complete or error, remove from active tasks after a delay and show toast
      if (event.phase === "complete" || event.phase === "error") {
        setTimeout(() => {
          set((s) => {
            const t = { ...s.activeTasks };
            delete t[event.task_id];
            return { activeTasks: t };
          });
        }, 3000);

        return {
          activeTasks: newTasks,
          toasts: [
            ...state.toasts,
            {
              id: `${event.task_id}-${Date.now()}`,
              type: event.phase === "complete" ? "success" : "error",
              message: event.message,
              createdAt: Date.now(),
            },
          ],
        };
      }

      return { activeTasks: newTasks };
    });
  },

  addToast: (type, message) => {
    set((state) => ({
      toasts: [
        ...state.toasts,
        { id: `toast-${Date.now()}-${Math.random()}`, type, message, createdAt: Date.now() },
      ],
    }));
  },

  dismissToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },

  clearTasks: () => {
    set({ activeTasks: {} });
  },
}));
