import { useState, useEffect, useCallback } from "react";
import {
  FolderOpen,
  ChevronDown,
  ChevronRight,
  Layers,
  Settings2,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

// ── Types ───────────────────────────────────────────────────────────────────

interface GroupInfo {
  name: string;
  color: string;
  instance_count: number;
  created_at: string;
}

// ── Props ───────────────────────────────────────────────────────────────────

interface GroupSidebarProps {
  activeGroup: string | null;
  onSelectGroup: (group: string | null) => void;
  onManageGroups?: () => void;
  /** Pass a refresh trigger to re-fetch groups */
  refreshKey?: number;
}

export default function GroupSidebar({
  activeGroup,
  onSelectGroup,
  onManageGroups,
  refreshKey,
}: GroupSidebarProps) {
  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [collapsed, setCollapsed] = useState(false);
  const [loading, setLoading] = useState(false);

  const fetchGroups = useCallback(async () => {
    try {
      setLoading(true);
      const result: GroupInfo[] = await invoke("list_groups");
      setGroups(result);
    } catch {
      // silently ignore — groups table might not exist yet
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchGroups();
  }, [fetchGroups, refreshKey]);

  const totalInstances = groups.reduce((sum, g) => sum + g.instance_count, 0);

  return (
    <div className="w-full border-b border-zinc-800 bg-zinc-900/50">
      {/* Section header */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex w-full items-center justify-between px-4 py-2.5 text-xs font-semibold uppercase tracking-wider text-zinc-500 transition-colors hover:text-zinc-300"
      >
        <span className="flex items-center gap-2">
          <FolderOpen className="h-3.5 w-3.5" />
          Groups
        </span>
        <div className="flex items-center gap-1">
          {onManageGroups && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onManageGroups();
              }}
              className="rounded p-0.5 text-zinc-600 transition-colors hover:bg-zinc-800 hover:text-zinc-400"
              title="Manage groups"
            >
              <Settings2 className="h-3 w-3" />
            </button>
          )}
          {collapsed ? (
            <ChevronRight className="h-3.5 w-3.5" />
          ) : (
            <ChevronDown className="h-3.5 w-3.5" />
          )}
        </div>
      </button>

      {/* Group list */}
      {!collapsed && (
        <div className="space-y-0.5 px-2 pb-3">
          {/* All Instances */}
          <button
            onClick={() => onSelectGroup(null)}
            className={`flex w-full items-center gap-3 rounded-lg px-3 py-1.5 text-sm transition-colors ${
              activeGroup === null
                ? "bg-zinc-800 text-white"
                : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200"
            }`}
          >
            <Layers className="h-3.5 w-3.5 flex-shrink-0" />
            <span className="flex-1 text-left">All Instances</span>
            {!loading && (
              <span className="min-w-[1.5rem] rounded-full bg-zinc-800 px-1.5 py-0.5 text-center text-xs text-zinc-500">
                {totalInstances}
              </span>
            )}
          </button>

          {/* Individual groups */}
          {loading ? (
            <p className="px-3 py-2 text-xs text-zinc-600">Loading…</p>
          ) : (
            groups.map((group) => (
              <button
                key={group.name}
                onClick={() => onSelectGroup(group.name)}
                className={`flex w-full items-center gap-3 rounded-lg px-3 py-1.5 text-sm transition-colors ${
                  activeGroup === group.name
                    ? "bg-zinc-800 text-white"
                    : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200"
                }`}
              >
                <div
                  className="h-3 w-3 flex-shrink-0 rounded-full"
                  style={{ backgroundColor: group.color }}
                />
                <span className="flex-1 truncate text-left">{group.name}</span>
                <span className="min-w-[1.5rem] rounded-full bg-zinc-800 px-1.5 py-0.5 text-center text-xs text-zinc-500">
                  {group.instance_count}
                </span>
              </button>
            ))
          )}

          {/* Empty state */}
          {!loading && groups.length === 0 && (
            <p className="px-3 py-2 text-xs text-zinc-600">
              No groups yet.
              {onManageGroups && (
                <button
                  onClick={onManageGroups}
                  className="ml-1 text-indigo-400 hover:underline"
                >
                  Create one
                </button>
              )}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
