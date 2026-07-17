import { useState } from "react";
import {
  Share2,
  Copy,
  Check,
  Download,
  Loader2,
} from "lucide-react";
import Modal from "./Modal";
import type { InstanceListItem } from "../../lib/tauri";
import {
  exportInstanceShare,
  importInstanceShare,
} from "../../lib/tauri";
import { useInstancesStore } from "../../stores/instances";

// ── Export Dialog ─────────────────────────────────────────────

interface ExportProps {
  instance: InstanceListItem;
  isOpen: boolean;
  onClose: () => void;
}

export function ShareExportDialog({ instance, isOpen, onClose }: ExportProps) {
  const [code, setCode] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await exportInstanceShare(instance.id);
      setCode(result.code);
    } catch (err) {
      setError(String(err));
    }
    setLoading(false);
  };

  const handleCopy = async () => {
    if (!code) return;
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for non-secure contexts
      const textarea = document.createElement("textarea");
      textarea.value = code;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleClose = () => {
    setCode(null);
    setError(null);
    setCopied(false);
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Share Instance">
      <div className="space-y-4">
        <div className="flex items-center gap-3 bg-slate-900 rounded-lg p-3">
          <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center text-white font-bold">
            {instance.name.charAt(0).toUpperCase()}
          </div>
          <div>
            <p className="text-white font-medium">{instance.name}</p>
            <p className="text-xs text-slate-400">
              {instance.game_version} &middot; {instance.loader}
              {instance.loader_version ? ` ${instance.loader_version}` : ""}
            </p>
          </div>
        </div>

        {!code && !loading && (
          <p className="text-sm text-slate-400">
            Generate a share code that your friends can import to get the exact
            same instance configuration.
          </p>
        )}

        {loading && (
          <div className="flex items-center justify-center gap-2 py-8 text-slate-400">
            <Loader2 size={16} className="animate-spin" />
            <span className="text-sm">Generating share code...</span>
          </div>
        )}

        {code && (
          <div className="space-y-3">
            <div className="bg-slate-900 border border-slate-700 rounded-lg p-3">
              <p className="text-xs text-slate-500 mb-1">Share Code</p>
              <code className="block text-xs text-slate-300 break-all max-h-32 overflow-y-auto font-mono">
                {code}
              </code>
            </div>

            <button
              onClick={handleCopy}
              className={`w-full flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                copied
                  ? "bg-emerald-600 text-white"
                  : "bg-blue-600 hover:bg-blue-500 text-white"
              }`}
            >
              {copied ? (
                <>
                  <Check size={16} />
                  Copied to clipboard!
                </>
              ) : (
                <>
                  <Copy size={16} />
                  Copy Share Code
                </>
              )}
            </button>

            <p className="text-xs text-slate-500 text-center">
              Anyone with this code can import your exact instance setup.
            </p>
          </div>
        )}

        {error && <p className="text-red-400 text-sm">{error}</p>}

        {!code && !loading && (
          <button
            onClick={handleExport}
            className="w-full flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-500 text-white py-2.5 rounded-lg text-sm font-medium transition-colors"
          >
            <Share2 size={16} />
            Generate Share Code
          </button>
        )}
      </div>
    </Modal>
  );
}

// ── Import Dialog ─────────────────────────────────────────────

interface ImportProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ShareImportDialog({ isOpen, onClose }: ImportProps) {
  const [code, setCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [importedName, setImportedName] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fetchInstances = useInstancesStore((s) => s.fetchInstances);

  const handleImport = async () => {
    if (!code.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const result = await importInstanceShare(code.trim());
      setImportedName(result.name);
      setSuccess(true);
      await fetchInstances();
    } catch (err) {
      setError(String(err));
    }
    setLoading(false);
  };

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      setCode(text);
    } catch {
      // Clipboard access denied — user can paste manually
    }
  };

  const handleClose = () => {
    setCode("");
    setError(null);
    setSuccess(false);
    setImportedName(null);
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Import Instance">
      <div className="space-y-4">
        {success ? (
          <div className="text-center py-4">
            <Check size={48} className="mx-auto text-emerald-400 mb-3" />
            <p className="text-white font-medium text-lg">
              Instance Imported!
            </p>
            <p className="text-sm text-slate-400 mt-1">
              "{importedName}" has been added to your instances.
            </p>
            <button
              onClick={handleClose}
              className="mt-4 bg-emerald-600 hover:bg-emerald-500 text-white px-6 py-2 rounded-lg text-sm font-medium transition-colors"
            >
              Done
            </button>
          </div>
        ) : (
          <>
            <p className="text-sm text-slate-400">
              Paste a share code from a friend to import their exact instance
              configuration.
            </p>

            <div className="relative">
              <textarea
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="OMC:H4sIAAAA..."
                rows={4}
                className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-xs font-mono focus:outline-none focus:border-blue-500 resize-none"
              />
              <button
                onClick={handlePaste}
                className="absolute top-2 right-2 text-slate-500 hover:text-slate-300 p-1"
                title="Paste from clipboard"
              >
                <Copy size={14} />
              </button>
            </div>

            {error && (
              <div className="bg-red-900/30 border border-red-800 rounded-lg p-3 text-red-300 text-sm">
                {error}
              </div>
            )}

            <button
              onClick={handleImport}
              disabled={!code.trim() || loading}
              className="w-full flex items-center justify-center gap-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white py-2.5 rounded-lg text-sm font-medium transition-colors"
            >
              {loading ? (
                <>
                  <Loader2 size={16} className="animate-spin" />
                  Importing...
                </>
              ) : (
                <>
                  <Download size={16} />
                  Import Instance
                </>
              )}
            </button>
          </>
        )}
      </div>
    </Modal>
  );
}
