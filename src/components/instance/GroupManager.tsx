import { useState, useEffect, useCallback } from "react";
import {
  FolderOpen,
  Plus,
  Trash2,
  X,
  Palette,
  Check,
} from "lucide-react";
import Modal from "../common/Modal";
import { invoke } from "@tauri-apps/api/core";

// ── Types ───────────────────────────────────────────────────────────────────

export interface GroupInfo {
  name: string;
  color: string;
  instance_count: number;
  created_at: string;
}

// ── Preset Colors ───────────────────────────────────────────────────────────

const PRESET_COLORS = [
  "#6366f1", // indigo
  "#8b5cf6", // violet
  "#a855f7", // purple
  "#ec4899", // pink
  "#f43f5e", // rose
  "#ef4444", // red
  "#f97316", // orange
  "#eab308", // yellow
  "#22c55e", // green
  "#14b8a6", // teal
  "#06b6d4", // cyan
  "#3b82f6", // blue
];

// ── Props ───────────────────────────────────────────────────────────────────

interface GroupManagerProps {
  isOpen: boolean;
  onClose: () => void;
  /** If set, show assign/remove controls for this instance */
  instanceId?: string;
  instanceGroups?: string[];
  onGroupsChanged?: () => void;
}

export default function GroupManager({
  isOpen,
  onClose,
  instanceId,
  instanceGroups = [],
  onGroupsChanged,
}: GroupManagerProps) {
  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [newName, setNewName] = useState("");
  const [newColor, setNewColor] = useState(PRESET_COLORS[0]);
  const [showColorPicker, setShowColorPicker] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // ── IPC wrappers using raw invoke ─────────────────────────────────────

  const fetchGroups = useCallback(async () => {
    try {
      setLoading(true);
      const result: GroupInfo[] = await invoke("list_groups");
      setGroups(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) fetchGroups();
  }, [isOpen, fetchGroups]);

  // ── Handlers ──────────────────────────────────────────────────────────

  const handleCreate = async () => {
    if (!newName.trim()) return;
    try {
      await invoke("create_group", {
        name: newName.trim(),
        color: newColor,
      });
      setNewName("");
      fetchGroups();
      onGroupsChanged?.();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDelete = async (name: string) => {
    try {
      await invoke("delete_group", { name });
      setDeleteConfirm(null);
      fetchGroups();
      onGroupsChanged?.();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleToggleAssign = async (groupName: string, currentlyAssigned: boolean) => {
    if (!instanceId) return;
    try {
      if (currentlyAssigned) {
        await invoke("remove_instance_from_group", {
          instanceId,
          groupName,
        });
      } else {
        await invoke("assign_instance_to_group", {
          instanceId,
          groupName,
        });
      }
      fetchGroups();
      onGroupsChanged?.();
    } catch (err) {
      setError(String(err));
    }
  };

  // ── Render ────────────────────────────────────────────────────────────

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Manage Groups" maxWidth="max-w-md">
      <div className="space-y-4">
        {/* Error */}
        {error && (
          <div className="flex items-center justify-between rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-400">
            <span>{error}</span>
            <button onClick={() => setError(null)} className="p-0.5">
              <X className="h-3.5 w-3.5" />
            </button>
          </div>
        )}

        {/* Create new group */}
        <div className="flex gap-2">
          <div className="relative flex-1">
            <FolderOpen className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" />
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleCreate()}
              placeholder="New group name…"
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 py-2 pl-9 pr-3 text-sm text-zinc-100 placeholder:text-zinc-500 focus:border-indigo-500 focus:outline-none"
            />
          </div>

          {/* Color picker trigger */}
          <div className="relative">
            <button
              onClick={() => setShowColorPicker(!showColorPicker)}
              className="flex h-10 w-10 items-center justify-center rounded-lg border border-zinc-700 bg-zinc-800 hover:border-zinc-600"
              title="Pick color"
            >
              <div
                className="h-5 w-5 rounded-full"
                style={{ backgroundColor: newColor }}
              />
            </button>

            {/* Color picker dropdown */}
            {showColorPicker && (
              <div className="absolute right-0 top-12 z-10 w-48 rounded-lg border border-zinc-700 bg-zinc-800 p-3 shadow-xl">
                <div className="mb-2 flex items-center gap-2 text-xs text-zinc-400">
                  <Palette className="h-3.5 w-3.5" />
                  <span>Color</span>
                </div>
                <div className="grid grid-cols-6 gap-1.5">
                  {PRESET_COLORS.map((c) => (
                    <button
                      key={c}
                      onClick={() => {
                        setNewColor(c);
                        setShowColorPicker(false);
                      }}
                      className="flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110"
                      style={{ backgroundColor: c }}
                    >
                      {c === newColor && <Check className="h-3.5 w-3.5 text-white" />}
                    </button>
                  ))}
                </div>
                <input
                  type="color"
                  value={newColor}
                  onChange={(e) => setNewColor(e.target.value)}
                  className="mt-2 h-8 w-full cursor-pointer rounded border-0 bg-transparent"
                />
              </div>
            )}
          </div>

          <button
            onClick={handleCreate}
            disabled={!newName.trim()}
            className="flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-40"
          >
            <Plus className="h-4 w-4" />
            Add
          </button>
        </div>

        {/* Groups list */}
        <div className="max-h-64 space-y-1 overflow-y-auto">
          {loading ? (
            <p className="py-4 text-center text-sm text-zinc-500">Loading…</p>
          ) : groups.length === 0 ? (
            <p className="py-4 text-center text-sm text-zinc-500">
              No groups yet. Create one above.
            </p>
          ) : (
            groups.map((group) => {
              const isAssigned = instanceGroups.includes(group.name);

              return (
                <div
                  key={group.name}
                  className="group flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-zinc-800/60"
                >
                  {/* Color dot */}
                  <div
                    className="h-3 w-3 flex-shrink-0 rounded-full"
                    style={{ backgroundColor: group.color }}
                  />

                  {/* Name + count */}
                  <div className="min-w-0 flex-1">
                    <span className="text-sm font-medium text-zinc-200">
                      {group.name}
                    </span>
                    <span className="ml-2 text-xs text-zinc-500">
                      {group.instance_count} instance
                      {group.instance_count !== 1 ? "s" : ""}
                    </span>
                  </div>

                  {/* Assign / remove toggle (only when instanceId is set) */}
                  {instanceId && (
                    <button
                      onClick={() => handleToggleAssign(group.name, isAssigned)}
                      className={`rounded-md px-2 py-1 text-xs font-medium transition-colors ${
                        isAssigned
                          ? "bg-green-500/20 text-green-400 hover:bg-green-500/30"
                          : "bg-zinc-700 text-zinc-400 hover:bg-zinc-600 hover:text-zinc-200"
                      }`}
                    >
                      {isAssigned ? "Assigned" : "Assign"}
                    </button>
                  )}

                  {/* Delete */}
                  {deleteConfirm === group.name ? (
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => handleDelete(group.name)}
                        className="rounded-md bg-red-500/20 px-2 py-1 text-xs font-medium text-red-400 hover:bg-red-500/30"
                      >
                        Confirm
                      </button>
                      <button
                        onClick={() => setDeleteConfirm(null)}
                        className="rounded-md px-2 py-1 text-xs text-zinc-500 hover:text-zinc-300"
                      >
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <button
                      onClick={() => setDeleteConfirm(group.name)}
                      className="rounded-md p-1.5 text-zinc-600 opacity-0 transition-opacity hover:bg-red-500/10 hover:text-red-400 group-hover:opacity-100"
                      title="Delete group"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>
    </Modal>
  );
}
