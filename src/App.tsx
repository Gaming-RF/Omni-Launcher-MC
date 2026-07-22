import { useEffect, useCallback } from "react";
import { Routes, Route } from "react-router-dom";
import Sidebar from "./components/layout/Sidebar";
import { Home } from "./pages/Home";
import { Discover } from "./pages/Discover";
import { Settings } from "./pages/Settings";
import { InstanceDetailPage } from "./pages/InstanceDetailPage";
import Import from "./pages/Import";
import Library from "./pages/Library";
import RunningInstancesBar from "./components/layout/RunningInstancesBar";
import { useAppInit } from "./hooks/useAppInit";
import { useDownloadProgress } from "./hooks/useDownloadProgress";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useNotificationStore } from "./stores/notificationStore";
import { DownloadProgress } from "./components/common/DownloadProgress";
import { ToastContainer } from "./components/common/Toast";
import { PageTransition } from "./components/common/PageTransition";

export default function App() {
  const { init } = useAppInit();
  const handleProgress = useNotificationStore((s) => s.handleProgress);
  useKeyboardShortcuts();

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
          <Route path="/" element={<PageTransition><Home /></PageTransition>} />
          <Route path="/instance/:id" element={<PageTransition><InstanceDetailPage /></PageTransition>} />
          <Route path="/discover" element={<PageTransition><Discover /></PageTransition>} />
          <Route path="/import" element={<PageTransition><Import /></PageTransition>} />
          <Route path="/settings" element={<PageTransition><Settings /></PageTransition>} />
          <Route path="/library" element={<PageTransition><Library /></PageTransition>} />
        </Routes>
      </main>
      <RunningInstancesBar />

      {/* Global overlay components */}
      <DownloadProgress />
      <ToastContainer />
    </div>
  );
}
