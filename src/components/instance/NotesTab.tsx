import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileText, Save, Loader2 } from "lucide-react";

interface Props {
  instanceId: string;
  initialNotes?: string;
}

export default function NotesTab({ instanceId, initialNotes }: Props) {
  const [notes, setNotes] = useState(initialNotes || "");
  const [saving, setSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<string | null>(null);

  const save = useCallback(
    async (text: string) => {
      setSaving(true);
      try {
        await invoke("update_instance", {
          id: instanceId,
          name: null,
          javaArgs: null,
          allocatedMemoryMb: null,
          notes: text,
        });
        setLastSaved(new Date().toLocaleTimeString());
      } catch (err) {
        console.error("Failed to save notes:", err);
      } finally {
        setSaving(false);
      }
    },
    [instanceId]
  );

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const text = e.target.value;
    setNotes(text);
    // Debounced save
    const timeout = setTimeout(() => save(text), 500);
    return () => clearTimeout(timeout);
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white flex items-center gap-2">
          <FileText size={20} />
          Notes
        </h3>
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          {saving ? (
            <span className="flex items-center gap-1">
              <Loader2 size={12} className="animate-spin" />
              Saving...
            </span>
          ) : lastSaved ? (
            <span className="flex items-center gap-1">
              <Save size={12} />
              Saved at {lastSaved}
            </span>
          ) : null}
          <span>{notes.length} chars</span>
        </div>
      </div>
      <textarea
        value={notes}
        onChange={handleChange}
        placeholder="Add notes about this instance... (mods to try, settings, bugs, etc.)"
        className="w-full h-80 bg-zinc-800 text-zinc-100 rounded-lg p-4 text-sm resize-y border border-zinc-700/50 focus:border-blue-500/50 focus:outline-none font-mono"
      />
    </div>
  );
}
