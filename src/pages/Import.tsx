import { useState } from "react";
import {
  scanLauncherInstances,
  importLauncherInstance,
  type ImportableInstance,
} from "../lib/tauri";
import { useI18nStore } from "../stores/i18n";

const LAUNCHER_TYPES = [
  { id: "multi_mc", label: "MultiMC" },
  { id: "prism_launcher", label: "Prism Launcher" },
  { id: "curseforge_app", label: "CurseForge App" },
  { id: "atlauncher", label: "ATLauncher" },
  { id: "gdlauncher", label: "GDLauncher" },
  { id: "vanilla", label: "Vanilla Launcher" },
] as const;

export default function Import() {
  const { t } = useI18nStore();
  const [selectedLauncher, setSelectedLauncher] = useState("multi_mc");
  const [instances, setInstances] = useState<ImportableInstance[]>([]);
  const [scanning, setScanning] = useState(false);
  const [importing, setImporting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string[]>([]);

  const handleScan = async () => {
    setScanning(true);
    setError(null);
    setInstances([]);
    try {
      const results = await scanLauncherInstances(selectedLauncher);
      setInstances(results);
      if (results.length === 0) {
        setError("No instances found. Make sure the launcher is installed.");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setScanning(false);
    }
  };

  const handleImport = async (inst: ImportableInstance) => {
    setImporting(inst.source_path);
    setError(null);
    try {
      await importLauncherInstance(
        inst.source_launcher.toLowerCase().replace(/\s/g, "_"),
        inst.source_path,
        inst.name,
        inst.game_version,
        inst.loader,
        inst.loader_version ?? undefined
      );
      setSuccess((prev) => [...prev, inst.name]);
    } catch (e) {
      setError(`Failed to import ${inst.name}: ${e}`);
    } finally {
      setImporting(null);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">{t("import.title")}</h1>

      <div className="bg-gray-800/50 rounded-lg p-4 mb-6">
        <p className="text-gray-300 text-sm mb-4">
          Import instances from other Minecraft launchers. Your mods, saves, and
          configurations will be copied automatically.
        </p>

        <div className="flex gap-3 items-end">
          <div className="flex-1">
            <label className="block text-sm text-gray-400 mb-1">
              Launcher
            </label>
            <select
              value={selectedLauncher}
              onChange={(e) => setSelectedLauncher(e.target.value)}
              className="w-full bg-gray-700 text-white rounded-lg px-3 py-2 border border-gray-600 focus:border-emerald-500 focus:outline-none"
            >
              {LAUNCHER_TYPES.map((lt) => (
                <option key={lt.id} value={lt.id}>
                  {lt.label}
                </option>
              ))}
            </select>
          </div>
          <button
            onClick={handleScan}
            disabled={scanning}
            className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 rounded-lg text-white font-medium transition-colors"
          >
            {scanning ? "Scanning..." : "Scan for Instances"}
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg p-3 mb-4 text-red-300 text-sm">
          {error}
        </div>
      )}

      {instances.length > 0 && (
        <div className="space-y-3">
          <h2 className="text-lg font-semibold">
            Found {instances.length} instance
            {instances.length !== 1 ? "s" : ""}
          </h2>
          {instances.map((inst) => {
            const isImported = success.includes(inst.name);
            const isImporting = importing === inst.source_path;

            return (
              <div
                key={inst.source_path}
                className="bg-gray-800/50 rounded-lg p-4 flex items-center justify-between"
              >
                <div>
                  <div className="font-medium">{inst.name}</div>
                  <div className="text-sm text-gray-400">
                    {inst.game_version} • {inst.loader}
                    {inst.loader_version ? ` ${inst.loader_version}` : ""}
                    <span className="ml-2 text-gray-500">
                      from {inst.source_launcher}
                    </span>
                  </div>
                </div>
                <button
                  onClick={() => handleImport(inst)}
                  disabled={isImporting || isImported}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                    isImported
                      ? "bg-green-800/50 text-green-400 cursor-default"
                      : isImporting
                        ? "bg-gray-600 text-gray-400 cursor-wait"
                        : "bg-emerald-600 hover:bg-emerald-700 text-white"
                  }`}
                >
                  {isImported ? "✓ Imported" : isImporting ? "Importing..." : "Import"}
                </button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
