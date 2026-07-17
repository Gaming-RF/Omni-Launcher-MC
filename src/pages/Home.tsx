import { useState } from "react";
import { Play, Trash2, Clock, ChevronRight, Share2, Download } from "lucide-react";
import { InstanceCreator } from "../components/instance/InstanceCreator";
import { InstanceDetail } from "./InstanceDetail";
import { ShareExportDialog, ShareImportDialog } from "../components/common/ShareDialog";
import { useInstancesStore } from "../stores/instances";
import { useAuthStore } from "../stores/auth";
import type { InstanceListItem } from "../lib/tauri";

export function Home() {
  const instances = useInstancesStore((s) => s.instances);
  const deleteInstance = useInstancesStore((s) => s.deleteInstance);
  const launchGame = useInstancesStore((s) => s.launchGame);
  const activeAccount = useAuthStore((s) => s.activeAccount);
  const [selectedInstance, setSelectedInstance] = useState<InstanceListItem | null>(null);
  const [shareInstance, setShareInstance] = useState<InstanceListItem | null>(null);
  const [showImport, setShowImport] = useState(false);

  // If an instance is selected, show the detail view
  if (selectedInstance) {
    // Find the latest version from the store (in case it was updated)
    const latest = instances.find((i) => i.id === selectedInstance.id);
    return (
      <InstanceDetail
        instance={latest || selectedInstance}
        onBack={() => setSelectedInstance(null)}
      />
    );
  }

  const formatPlayTime = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m`;
    return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
  };

  return (
    <>
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
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowImport(true)}
            className="bg-slate-700 hover:bg-slate-600 text-white px-3 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-1.5"
          >
            <Download size={14} />
            Import
          </button>
          <InstanceCreator />
        </div>
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
              onClick={() => setSelectedInstance(instance)}
              className="bg-slate-800 rounded-xl p-4 border border-slate-700 hover:border-slate-600 transition-colors group cursor-pointer"
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
                <div className="flex items-center gap-1">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setShareInstance(instance);
                    }}
                    className="opacity-0 group-hover:opacity-100 text-slate-500 hover:text-blue-400 transition-all p-1"
                    title="Share instance"
                  >
                    <Share2 size={16} />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteInstance(instance.id);
                    }}
                    className="opacity-0 group-hover:opacity-100 text-slate-500 hover:text-red-400 transition-all p-1"
                    title="Delete instance"
                  >
                    <Trash2 size={16} />
                  </button>
                  <ChevronRight
                    size={16}
                    className="text-slate-600 group-hover:text-slate-400 transition-colors"
                  />
                </div>
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-1 text-xs text-slate-500">
                  <Clock size={12} />
                  <span>{formatPlayTime(instance.play_time_secs)}</span>
                  <span className="mx-1">&middot;</span>
                  <span>{instance.allocated_memory_mb}MB RAM</span>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    launchGame(instance.id);
                  }}
                  className="bg-emerald-600 hover:bg-emerald-500 text-white px-3 py-1.5 rounded-lg text-sm font-medium transition-colors"
                >
                  Play
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
    {/* Share dialogs */}
    {shareInstance && (
      <ShareExportDialog
        instance={shareInstance}
        isOpen={!!shareInstance}
        onClose={() => setShareInstance(null)}
      />
    )}
    <ShareImportDialog
      isOpen={showImport}
      onClose={() => setShowImport(false)}
    />
    </>
  );
}
