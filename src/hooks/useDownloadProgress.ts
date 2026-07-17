import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export interface ProgressEvent {
  task_id: string;
  phase: string;
  current: number;
  total: number;
  message: string;
}

type ProgressCallback = (event: ProgressEvent) => void;

/**
 * Subscribe to download-progress events from the Rust backend.
 * Returns an unsubscribe function.
 */
export function onDownloadProgress(callback: ProgressCallback): () => void {
  let unlisten: (() => void) | null = null;

  listen<ProgressEvent>("download-progress", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return () => {
    unlisten?.();
  };
}

/**
 * React hook that subscribes to download-progress events.
 */
export function useDownloadProgress(callback: ProgressCallback) {
  useEffect(() => {
    return onDownloadProgress(callback);
  }, [callback]);
}
