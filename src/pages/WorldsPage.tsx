import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import {
  getInstanceWorlds,
  addServer,
  removeServer,
  deleteWorld,
  backupWorld,
  type WorldsInfo,
  type ServerEntry,
  type SingleplayerWorld,
} from "../lib/tauri";

export default function WorldsPage() {
  const { id: instanceId } = useParams<{ id: string }>();
  const [worlds, setWorlds] = useState<WorldsInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<"servers" | "worlds">("servers");
  const [showAddServer, setShowAddServer] = useState(false);
  const [newServerName, setNewServerName] = useState("");
  const [newServerAddress, setNewServerAddress] = useState("");
  const [error, setError] = useState<string | null>(null);

  const loadWorlds = useCallback(async () => {
    if (!instanceId) return;
    try {
      const data = await getInstanceWorlds(instanceId);
      setWorlds(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [instanceId]);

  useEffect(() => {
    loadWorlds();
  }, [loadWorlds]);

  const handleAddServer = async () => {
    if (!instanceId || !newServerName || !newServerAddress) return;
    try {
      await addServer(instanceId, newServerName, newServerAddress);
      setShowAddServer(false);
      setNewServerName("");
      setNewServerAddress("");
      await loadWorlds();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRemoveServer = async (index: number) => {
    if (!instanceId) return;
    try {
      await removeServer(instanceId, index);
      await loadWorlds();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDeleteWorld = async (folderName: string) => {
    if (!instanceId || !confirm("Delete this world permanently?")) return;
    try {
      await deleteWorld(instanceId, folderName);
      await loadWorlds();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleBackupWorld = async (folderName: string) => {
    if (!instanceId) return;
    try {
      const backupName = await backupWorld(instanceId, folderName);
      alert(`World backed up as: ${backupName}`);
    } catch (e) {
      setError(String(e));
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  if (loading) {
    return (
      <div className="p-6 flex items-center justify-center h-64">
        <div className="text-gray-400">Loading worlds...</div>
      </div>
    );
  }

  return (
    <div className="p-6">
      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg p-3 mb-4 text-red-300 text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-2 underline">
            dismiss
          </button>
        </div>
      )}

      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setTab("servers")}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            tab === "servers"
              ? "bg-emerald-600 text-white"
              : "bg-gray-700 text-gray-300 hover:bg-gray-600"
          }`}
        >
          Servers ({worlds?.servers.length ?? 0})
        </button>
        <button
          onClick={() => setTab("worlds")}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
            tab === "worlds"
              ? "bg-emerald-600 text-white"
              : "bg-gray-700 text-gray-300 hover:bg-gray-600"
          }`}
        >
          Worlds ({worlds?.singleplayer.length ?? 0})
        </button>
      </div>

      {tab === "servers" && (
        <div>
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-lg font-semibold">Server List</h2>
            <button
              onClick={() => setShowAddServer(true)}
              className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 rounded-lg text-sm text-white"
            >
              + Add Server
            </button>
          </div>

          {showAddServer && (
            <div className="bg-gray-800/50 rounded-lg p-4 mb-4">
              <div className="flex gap-3">
                <input
                  type="text"
                  placeholder="Server Name"
                  value={newServerName}
                  onChange={(e) => setNewServerName(e.target.value)}
                  className="flex-1 bg-gray-700 text-white rounded-lg px-3 py-2 border border-gray-600 focus:border-emerald-500 focus:outline-none"
                />
                <input
                  type="text"
                  placeholder="Address (e.g. play.hypixel.net)"
                  value={newServerAddress}
                  onChange={(e) => setNewServerAddress(e.target.value)}
                  className="flex-1 bg-gray-700 text-white rounded-lg px-3 py-2 border border-gray-600 focus:border-emerald-500 focus:outline-none"
                />
                <button
                  onClick={handleAddServer}
                  className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 rounded-lg text-white text-sm"
                >
                  Add
                </button>
                <button
                  onClick={() => setShowAddServer(false)}
                  className="px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded-lg text-white text-sm"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {worlds?.servers.length === 0 ? (
            <div className="text-center text-gray-500 py-12">
              No servers added yet
            </div>
          ) : (
            <div className="space-y-2">
              {worlds?.servers.map((server, i) => (
                <ServerRow
                  key={i}
                  server={server}
                  onRemove={() => handleRemoveServer(server.index)}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {tab === "worlds" && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Singleplayer Worlds</h2>
          {worlds?.singleplayer.length === 0 ? (
            <div className="text-center text-gray-500 py-12">
              No worlds found
            </div>
          ) : (
            <div className="space-y-2">
              {worlds?.singleplayer.map((world) => (
                <WorldRow
                  key={world.folder_name}
                  world={world}
                  formatSize={formatSize}
                  onDelete={() => handleDeleteWorld(world.folder_name)}
                  onBackup={() => handleBackupWorld(world.folder_name)}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ServerRow({
  server,
  onRemove,
}: {
  server: ServerEntry;
  onRemove: () => void;
}) {
  return (
    <div className="bg-gray-800/50 rounded-lg p-4 flex items-center justify-between">
      <div>
        <div className="font-medium">{server.name}</div>
        <div className="text-sm text-gray-400 font-mono">{server.address}</div>
      </div>
      <div className="flex gap-2">
        <button
          onClick={onRemove}
          className="px-3 py-1.5 bg-red-600/20 hover:bg-red-600/40 text-red-400 rounded-lg text-sm"
        >
          Remove
        </button>
      </div>
    </div>
  );
}

function WorldRow({
  world,
  formatSize,
  onDelete,
  onBackup,
}: {
  world: SingleplayerWorld;
  formatSize: (b: number) => string;
  onDelete: () => void;
  onBackup: () => void;
}) {
  return (
    <div className="bg-gray-800/50 rounded-lg p-4 flex items-center justify-between">
      <div className="flex items-center gap-3">
        {world.icon ? (
          <img
            src={`asset://localhost/${world.icon}`}
            alt=""
            className="w-10 h-10 rounded"
          />
        ) : (
          <div className="w-10 h-10 rounded bg-gray-700 flex items-center justify-center text-lg">
            🌍
          </div>
        )}
        <div>
          <div className="font-medium">{world.name}</div>
          <div className="text-sm text-gray-400">
            {world.game_mode} • {formatSize(world.size_bytes)}
          </div>
        </div>
      </div>
      <div className="flex gap-2">
        <button
          onClick={onBackup}
          className="px-3 py-1.5 bg-blue-600/20 hover:bg-blue-600/40 text-blue-400 rounded-lg text-sm"
        >
          Backup
        </button>
        <button
          onClick={onDelete}
          className="px-3 py-1.5 bg-red-600/20 hover:bg-red-600/40 text-red-400 rounded-lg text-sm"
        >
          Delete
        </button>
      </div>
    </div>
  );
}
