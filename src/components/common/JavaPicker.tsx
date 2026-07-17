import { useState, useEffect } from "react";
import {
  Cpu,
  Download,
  CheckCircle,
  Loader2,
  ChevronDown,
} from "lucide-react";
import type { JavaCheckResult } from "../../lib/tauri";
import { ensureJavaForMc, downloadJavaVersion } from "../../lib/tauri";

interface Props {
  value: string | null;
  onChange: (path: string | null) => void;
  /** If set, auto-detect the best Java for this MC version. */
  requiredMcVersion?: string;
}

export default function JavaPicker({ value, onChange, requiredMcVersion }: Props) {
  const [javaCheck, setJavaCheck] = useState<JavaCheckResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [customPath, setCustomPath] = useState(value ?? "");

  const checkJava = async () => {
    if (!requiredMcVersion) return;
    setLoading(true);
    setError(null);
    try {
      const result = await ensureJavaForMc(requiredMcVersion, value ?? undefined);
      setJavaCheck(result);
    } catch (err) {
      setError(String(err));
    }
    setLoading(false);
  };

  useEffect(() => {
    if (requiredMcVersion) {
      checkJava();
    }
  }, [requiredMcVersion]);

  const handleAutoDownload = async () => {
    if (!javaCheck) return;
    setDownloading(true);
    setError(null);
    try {
      const path = await downloadJavaVersion(javaCheck.major_version);
      onChange(path);
      await checkJava();
    } catch (err) {
      setError(String(err));
    }
    setDownloading(false);
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <Cpu size={16} className="text-slate-400" />
        <span className="text-sm text-slate-300">Java Runtime</span>
      </div>

      {/* Auto-detect status */}
      {requiredMcVersion && (
        <div className="bg-slate-900 border border-slate-600 rounded-lg p-3">
          {loading ? (
            <div className="flex items-center gap-2 text-sm text-slate-400">
              <Loader2 size={14} className="animate-spin" />
              Checking Java for MC {requiredMcVersion}...
            </div>
          ) : javaCheck?.found ? (
            <div className="flex items-center gap-2">
              <CheckCircle size={16} className="text-emerald-400" />
              <div>
                <p className="text-sm text-white">
                  Java {javaCheck.major_version} found
                  {javaCheck.auto_downloaded && (
                    <span className="ml-2 text-xs bg-emerald-900/40 text-emerald-400 px-1.5 py-0.5 rounded">
                      auto-downloaded
                    </span>
                  )}
                </p>
                <p className="text-xs text-slate-500 font-mono">{javaCheck.path}</p>
              </div>
            </div>
          ) : javaCheck ? (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Cpu size={16} className="text-yellow-400" />
                <p className="text-sm text-yellow-300">
                  Java {javaCheck.major_version} not found
                </p>
              </div>
              <button
                onClick={handleAutoDownload}
                disabled={downloading}
                className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-3 py-2 rounded-lg text-sm font-medium transition-colors"
              >
                {downloading ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  <Download size={14} />
                )}
                Download Java {javaCheck.major_version}
              </button>
            </div>
          ) : null}
        </div>
      )}

      {/* Custom path override */}
      <div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex items-center gap-2 text-xs text-slate-500 hover:text-slate-300 transition-colors"
        >
          <ChevronDown
            size={12}
            className={`transition-transform ${expanded ? "rotate-180" : ""}`}
          />
          Custom Java path override
        </button>

        {expanded && (
          <div className="mt-2 flex gap-2">
            <input
              type="text"
              value={customPath}
              onChange={(e) => setCustomPath(e.target.value)}
              placeholder="/usr/bin/java"
              className="flex-1 bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
            />
            <button
              onClick={() => onChange(customPath || null)}
              className="bg-slate-700 hover:bg-slate-600 text-white px-3 py-2 rounded-lg text-sm"
            >
              Set
            </button>
          </div>
        )}
      </div>

      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  );
}
