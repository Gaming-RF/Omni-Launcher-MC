import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { Download, CheckCircle2, AlertCircle, SkipForward, X } from "lucide-react";
import ProgressBar from "./ProgressBar";

interface DownloadProgressEvent {
  index: number;
  total: number;
  bytes_downloaded: number;
  total_bytes: number;
  display_name: string;
  status: DownloadStatus;
}

type DownloadStatus =
  | "Downloading"
  | "Verifying"
  | "Completed"
  | "Skipped"
  | { Failed: string };

interface ActiveDownload {
  name: string;
  bytesDownloaded: number;
  totalBytes: number;
  status: DownloadStatus;
}

export default function DownloadPanel() {
  const [active, setActive] = useState<Map<number, ActiveDownload>>(new Map());
  const [visible, setVisible] = useState(false);
  const [completed, setCompleted] = useState(0);
  const [total, setTotal] = useState(0);

  const handleEvent = useCallback((event: { payload: DownloadProgressEvent }) => {
    const p = event.payload;
    setVisible(true);
    setTotal(p.total);

    setActive((prev) => {
      const next = new Map(prev);
      next.set(p.index, {
        name: p.display_name || `File ${p.index + 1}`,
        bytesDownloaded: p.bytes_downloaded,
        totalBytes: p.total_bytes,
        status: p.status,
      });
      return next;
    });

    if (
      typeof p.status === "string" &&
      (p.status === "Completed" || p.status === "Skipped")
    ) {
      setCompleted((c) => c + 1);
    }
    if (typeof p.status === "object" && "Failed" in p.status) {
      setCompleted((c) => c + 1);
    }
  }, []);

  useEffect(() => {
    const unlisten = listen<DownloadProgressEvent>("download-progress", handleEvent);
    return () => { unlisten.then((fn) => fn()); };
  }, [handleEvent]);

  useEffect(() => {
    if (total > 0 && completed >= total) {
      const timer = setTimeout(() => {
        setVisible(false);
        setActive(new Map());
        setCompleted(0);
        setTotal(0);
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [completed, total]);

  if (!visible || active.size === 0) return null;

  const downloads = Array.from(active.entries()).sort(([a], [b]) => a - b);
  const overallPercent = total > 0 ? Math.round((completed / total) * 100) : 0;

  return (
    <div className="fixed bottom-4 right-4 z-50 w-96 rounded-xl border border-zinc-700 bg-zinc-900/95 p-4 shadow-2xl backdrop-blur">
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Download className="h-4 w-4 text-green-400" />
          <span className="text-sm font-medium text-zinc-200">
            Downloads ({completed}/{total})
          </span>
        </div>
        <button onClick={() => setVisible(false)} className="rounded p-1 text-zinc-500 hover:text-zinc-300">
          <X className="h-4 w-4" />
        </button>
      </div>

      <ProgressBar value={overallPercent} size="sm" label="Overall" />

      <div className="mt-3 max-h-64 space-y-2 overflow-y-auto">
        {downloads.map(([idx, dl]) => (
          <div key={idx} className="flex items-center gap-2 text-xs">
            <StatusIcon status={dl.status} />
            <span className="flex-1 truncate text-zinc-300">{dl.name}</span>
            <span className="w-12 text-right text-zinc-500">{formatStatus(dl)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function StatusIcon({ status }: { status: DownloadStatus }) {
  if (typeof status === "string") {
    switch (status) {
      case "Downloading": return <Download className="h-3 w-3 animate-pulse text-blue-400" />;
      case "Verifying": return <Download className="h-3 w-3 animate-spin text-yellow-400" />;
      case "Completed": return <CheckCircle2 className="h-3 w-3 text-green-400" />;
      case "Skipped": return <SkipForward className="h-3 w-3 text-zinc-500" />;
    }
  }
  return <AlertCircle className="h-3 w-3 text-red-400" />;
}

function formatStatus(dl: ActiveDownload): string {
  const { status, bytesDownloaded, totalBytes } = dl;
  if (typeof status === "object" && "Failed" in status) return "Failed";
  switch (status) {
    case "Downloading":
      if (totalBytes > 0) return `${formatBytes(bytesDownloaded)}/${formatBytes(totalBytes)}`;
      return formatBytes(bytesDownloaded);
    case "Verifying": return "Verifying";
    case "Completed": return formatBytes(bytesDownloaded);
    case "Skipped": return "Skipped";
    default: return "";
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}
