import { useState } from "react";
import Modal from "../common/Modal";
import Button from "../common/Button";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (params: {
    name: string;
    game_version: string;
    mod_loader?: string;
  }) => void;
  versions: string[];
}

const MOD_LOADERS = [
  { value: "", label: "Vanilla" },
  { value: "fabric", label: "Fabric" },
  { value: "forge", label: "Forge" },
  { value: "neoforge", label: "NeoForge" },
  { value: "quilt", label: "Quilt" },
];

export default function InstanceCreator({
  isOpen,
  onClose,
  onSubmit,
  versions,
}: Props) {
  const [name, setName] = useState("");
  const [gameVersion, setGameVersion] = useState(versions[0] ?? "");
  const [modLoader, setModLoader] = useState("");

  const handleSubmit = () => {
    if (!name.trim() || !gameVersion) return;
    onSubmit({
      name: name.trim(),
      game_version: gameVersion,
      mod_loader: modLoader || undefined,
    });
    setName("");
    setGameVersion(versions[0] ?? "");
    setModLoader("");
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Create Instance">
      <div className="space-y-4">
        {/* Name */}
        <div>
          <label className="mb-1.5 block text-sm font-medium text-zinc-300">
            Instance Name
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Modded World"
            className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-500 focus:border-green-500 focus:outline-none focus:ring-1 focus:ring-green-500"
          />
        </div>

        {/* Game Version */}
        <div>
          <label className="mb-1.5 block text-sm font-medium text-zinc-300">
            Minecraft Version
          </label>
          <select
            value={gameVersion}
            onChange={(e) => setGameVersion(e.target.value)}
            className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 focus:border-green-500 focus:outline-none"
          >
            {versions.map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        </div>

        {/* Mod Loader */}
        <div>
          <label className="mb-1.5 block text-sm font-medium text-zinc-300">
            Mod Loader
          </label>
          <select
            value={modLoader}
            onChange={(e) => setModLoader(e.target.value)}
            className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 focus:border-green-500 focus:outline-none"
          >
            {MOD_LOADERS.map(({ value, label }) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-2">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!name.trim()}>
            Create
          </Button>
        </div>
      </div>
    </Modal>
  );
}
