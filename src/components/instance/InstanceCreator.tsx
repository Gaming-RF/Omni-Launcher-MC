import { useState, useEffect } from "react";
import Button from "../common/Button";
import Modal from "../common/Modal";
import { useInstancesStore } from "../../stores/instances";
import type { VersionEntry } from "../../lib/tauri";
import { getVersionManifest } from "../../lib/tauri";

export function InstanceCreator() {
  const [isOpen, setIsOpen] = useState(false);
  const [name, setName] = useState("");
  const [gameVersion, setGameVersion] = useState("");
  const [loader, setLoader] = useState("vanilla");
  const [memory, setMemory] = useState(4096);
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(false);

  const createInstance = useInstancesStore((s) => s.createInstance);

  useEffect(() => {
    if (isOpen && versions.length === 0) {
      setLoadingVersions(true);
      getVersionManifest()
        .then(setVersions)
        .catch(console.error)
        .finally(() => setLoadingVersions(false));
    }
  }, [isOpen, versions.length]);

  const handleCreate = async () => {
    if (!name || !gameVersion) return;
    try {
      await createInstance({
        name,
        game_version: gameVersion,
        loader,
        loader_version: null,
        icon: null,
        java_args: null,
        allocated_memory_mb: memory,
      });
      setIsOpen(false);
      setName("");
      setGameVersion("");
      setLoader("vanilla");
    } catch {
      // Error handled in store
    }
  };

  const releases = versions.filter((v) => v.version_type === "release");
  const snapshots = versions.filter((v) => v.version_type === "snapshot");

  return (
    <>
      <Button onClick={() => setIsOpen(true)}>+ New Instance</Button>
      <Modal isOpen={isOpen} onClose={() => setIsOpen(false)} title="Create Instance">
        <div className="space-y-4">
          {/* Name */}
          <div>
            <label className="block text-sm text-slate-300 mb-1">Instance Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Modded World"
              className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          {/* Game Version */}
          <div>
            <label className="block text-sm text-slate-300 mb-1">Game Version</label>
            {loadingVersions ? (
              <p className="text-sm text-slate-400">Loading versions...</p>
            ) : (
              <select
                value={gameVersion}
                onChange={(e) => setGameVersion(e.target.value)}
                className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
              >
                <option value="">Select a version</option>
                <optgroup label="Releases">
                  {releases.map((v) => (
                    <option key={v.id} value={v.id}>
                      {v.id}
                    </option>
                  ))}
                </optgroup>
                <optgroup label="Snapshots">
                  {snapshots.slice(0, 20).map((v) => (
                    <option key={v.id} value={v.id}>
                      {v.id}
                    </option>
                  ))}
                </optgroup>
              </select>
            )}
          </div>

          {/* Loader */}
          <div>
            <label className="block text-sm text-slate-300 mb-1">Mod Loader</label>
            <select
              value={loader}
              onChange={(e) => setLoader(e.target.value)}
              className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
            >
              <option value="vanilla">Vanilla</option>
              <option value="fabric">Fabric</option>
              <option value="forge">Forge</option>
              <option value="neoforge">NeoForge</option>
              <option value="quilt">Quilt</option>
            </select>
          </div>

          {/* Memory */}
          <div>
            <label className="block text-sm text-slate-300 mb-1">
              Allocated Memory: {memory}MB
            </label>
            <input
              type="range"
              min={1024}
              max={16384}
              step={512}
              value={memory}
              onChange={(e) => setMemory(Number(e.target.value))}
              className="w-full accent-blue-500"
            />
            <div className="flex justify-between text-xs text-slate-500">
              <span>1GB</span>
              <span>16GB</span>
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            <button
              onClick={() => setIsOpen(false)}
              className="flex-1 bg-slate-700 hover:bg-slate-600 text-white py-2 rounded-lg text-sm"
            >
              Cancel
            </button>
            <button
              onClick={handleCreate}
              disabled={!name || !gameVersion}
              className="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed text-white py-2 rounded-lg text-sm font-medium"
            >
              Create
            </button>
          </div>
        </div>
      </Modal>
    </>
  );
}
