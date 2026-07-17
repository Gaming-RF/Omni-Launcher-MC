import { useEffect, useCallback } from "react";
import { Routes, Route } from "react-router-dom";
import Sidebar from "./components/layout/Sidebar";
import { Home } from "./pages/Home";
import { Discover } from "./pages/Discover";
import { Settings } from "./pages/Settings";
import { InstanceDetailPage } from "./pages/InstanceDetailPage";
import { useAppInit } from "./hooks/useAppInit";
import { useDownloadProgress } from "./hooks/useDownloadProgress";
import { useNotificationStore } from "./stores/notificationStore";
import { DownloadProgress } from "./components/common/DownloadProgress";
import { ToastContainer } from "./components/common/Toast";

export default function App() {
  const { init } = useAppInit();
  const handleProgress = useNotificationStore((s) => s.handleProgress);

  useEffect(() => {
    init();
  }, [init]);

  // Subscribe to download-progress events from Rust backend
  const onProgress = useCallback(
    (event: Parameters<typeof handleProgress>[0]) => {
      handleProgress(event);
    },
    [handleProgress]
  );
  useDownloadProgress(onProgress);

  return (
    <div className="flex h-full bg-zinc-950 text-zinc-100">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-6">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/instance/:id" element={<InstanceDetailPage />} />
          <Route path="/discover" element={<Discover />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>

      {/* Global overlay components */}
      <DownloadProgress />
      <ToastContainer />
    </div>
  );
}
