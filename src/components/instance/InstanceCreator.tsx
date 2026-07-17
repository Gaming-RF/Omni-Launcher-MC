import { useState, useEffect } from "react";
import Button from "../common/Button";
import Modal from "../common/Modal";
import { useInstancesStore } from "../../stores/instances";
import type { VersionEntry, ModpackInfo } from "../../lib/tauri";
import {
  getVersionManifest,
  parseMrpackFile,
  installMrpackModpack,
} from "../../lib/tauri";
import { Package, Upload, Loader2 } from "lucide-react";

type CreateMode = "manual" | "modpack";

export function InstanceCreator() {
  const [isOpen, setIsOpen] = useState(false);
  const [mode, setMode] = useState<CreateMode>("manual");
  const [name, setName] = useState("");
  const [gameVersion, setGameVersion] = useState("");
  const [loader, setLoader] = useState("vanilla");
  const [memory, setMemory] = useState(4096);
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(false);

  // Modpack state
  const [modpackPath, setModpackPath] = useState<string | null>(null);
  const [modpackInfo, setModpackInfo] = useState<ModpackInfo | null>(null);
  const [modpackLoading, setModpackLoading] = useState(false);
  const [modpackInstalling, setModpackInstalling] = useState(false);
  const [modpackError, setModpackError] = useState<string | null>(null);

  const createInstance = useInstancesStore((s) => s.createInstance);
  const fetchInstances = useInstancesStore((s) => s.fetchInstances);

  useEffect(() => {
    if (isOpen && versions.length === 0 && mode === "manual") {
      setLoadingVersions(true);
      getVersionManifest()
        .then(setVersions)
        .catch(console.error)
        .finally(() => setLoadingVersions(false));
    }
  }, [isOpen, versions.length, mode]);

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
      resetForm();
    } catch {
      // Error handled in store
    }
  };

  const handleModpackSelect = async () => {
    // Use Tauri dialog to select file
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Modpack", extensions: ["mrpack", "zip"] },
        ],
      });
      if (selected && typeof selected === "string") {
        setModpackPath(selected);
        setModpackLoading(true);
        setModpackError(null);
        try {
          const info = await parseMrpackFile(selected);
          setModpackInfo(info);
          if (!name) setName(info.name);
        } catch (err) {
          setModpackError(String(err));
        }
        setModpackLoading(false);
      }
    } catch (err) {
      setModpackError("File dialog not available. Path entry coming soon.");
    }
  };

  const handleModpackInstall = async () => {
    if (!modpackPath || !name) return;
    setModpackInstalling(true);
    setModpackError(null);
    try {
      await installMrpackModpack(modpackPath, name);
      await fetchInstances();
      setIsOpen(false);
      resetForm();
    } catch (err) {
      setModpackError(String(err));
    }
    setModpackInstalling(false);
  };

  const resetForm = () => {
    setName("");
    setGameVersion("");
    setLoader("vanilla");
    setMemory(4096);
    setModpackPath(null);
    setModpackInfo(null);
    setModpackError(null);
  };

  const releases = versions.filter((v) => v.version_type === "release");
  const snapshots = versions.filter((v) => v.version_type === "snapshot");

  return (
    <>
      <Button onClick={() => setIsOpen(true)}>+ New Instance</Button>
      <Modal isOpen={isOpen} onClose={() => { setIsOpen(false); resetForm(); }} title="Create Instance">
        <div className="space-y-4">
          {/* Mode tabs */}
          <div className="flex gap-1 bg-slate-900 rounded-lg p-1">
            <button
              onClick={() => setMode("manual")}
              className={`flex-1 px-3 py-1.5 rounded text-sm font-medium transition-colors ${
                mode === "manual" ? "bg-blue-600 text-white" : "text-slate-400"
              }`}
            >
              Manual
            </button>
            <button
              onClick={() => setMode("modpack")}
              className={`flex-1 px-3 py-1.5 rounded text-sm font-medium transition-colors flex items-center justify-center gap-1 ${
                mode === "modpack" ? "bg-blue-600 text-white" : "text-slate-400"
              }`}
            >
              <Package size={14} />
              From Modpack
            </button>
          </div>

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

          {mode === "manual" ? (
            <>
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
                        <option key={v.id} value={v.id}>{v.id}</option>
                      ))}
                    </optgroup>
                    <optgroup label="Snapshots">
                      {snapshots.slice(0, 20).map((v) => (
                        <option key={v.id} value={v.id}>{v.id}</option>
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
            </>
          ) : (
            <>
              {/* Modpack import */}
              <div>
                <button
                  onClick={handleModpackSelect}
                  disabled={modpackLoading}
                  className="w-full bg-slate-900 border-2 border-dashed border-slate-600 hover:border-blue-500 rounded-lg p-6 text-center transition-colors"
                >
                  {modpackLoading ? (
                    <Loader2 size={24} className="animate-spin mx-auto text-slate-400" />
                  ) : modpackPath ? (
                    <div>
                      <Package size={24} className="mx-auto text-emerald-400 mb-2" />
                      <p className="text-white text-sm font-medium">{modpackPath.split(/[/\\]/).pop()}</p>
                      <p className="text-xs text-slate-400 mt-1">Click to change</p>
                    </div>
                  ) : (
                    <div>
                      <Upload size={24} className="mx-auto text-slate-400 mb-2" />
                      <p className="text-white text-sm font-medium">Select .mrpack or .zip file</p>
                      <p className="text-xs text-slate-400 mt-1">Modrinth or CurseForge modpack</p>
                    </div>
                  )}
                </button>
              </div>

              {modpackInfo && (
                <div className="bg-slate-900 rounded-lg p-3 border border-slate-700">
                  <p className="text-white font-medium">{modpackInfo.name}</p>
                  <p className="text-xs text-slate-400 mt-1">
                    MC {modpackInfo.game_version} &middot; {modpackInfo.loader}
                    {modpackInfo.loader_version ? ` ${modpackInfo.loader_version}` : ""} &middot;{" "}
                    {modpackInfo.file_count} files
                  </p>
                  {modpackInfo.summary && (
                    <p className="text-xs text-slate-400 mt-2">{modpackInfo.summary}</p>
                  )}
                </div>
              )}
            </>
          )}

          {modpackError && (
            <p className="text-red-400 text-sm">{modpackError}</p>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            <button
              onClick={() => { setIsOpen(false); resetForm(); }}
              className="flex-1 bg-slate-700 hover:bg-slate-600 text-white py-2 rounded-lg text-sm"
            >
              Cancel
            </button>
            {mode === "manual" ? (
              <button
                onClick={handleCreate}
                disabled={!name || !gameVersion}
                className="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed text-white py-2 rounded-lg text-sm font-medium"
              >
                Create
              </button>
            ) : (
              <button
                onClick={handleModpackInstall}
                disabled={!name || !modpackPath || modpackInstalling}
                className="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed text-white py-2 rounded-lg text-sm font-medium flex items-center justify-center gap-2"
              >
                {modpackInstalling ? (
                  <>
                    <Loader2 size={14} className="animate-spin" />
                    Installing...
                  </>
                ) : (
                  "Install Modpack"
                )}
              </button>
            )}
          </div>
        </div>
      </Modal>
    </>
  );
}
