import { useState, useEffect } from "react";
import { Loader2 } from "lucide-react";
import {
  getFabricLoaderVersions,
  getQuiltLoaderVersions,
  getForgeVersions,
  getNeoForgeVersions,
} from "../../lib/tauri";

interface Props {
  loader: string;
  gameVersion: string;
  value: string | null;
  onChange: (version: string) => void;
}

export default function LoaderVersionSelector({
  loader,
  gameVersion,
  value,
  onChange,
}: Props) {
  const [versions, setVersions] = useState<{ version: string; stable: boolean }[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (loader === "vanilla" || !gameVersion) {
      setVersions([]);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    const fetchVersions = async () => {
      try {
        let result: { version: string; stable: boolean }[] = [];
        switch (loader) {
          case "fabric": {
            const v = await getFabricLoaderVersions(gameVersion);
            result = v;
            break;
          }
          case "quilt": {
            const v = await getQuiltLoaderVersions(gameVersion);
            result = v;
            break;
          }
          case "forge": {
            const v = await getForgeVersions(gameVersion);
            result = v.map((ver) => ({ version: ver, stable: true }));
            break;
          }
          case "neoforge": {
            const v = await getNeoForgeVersions(gameVersion);
            result = v.map((ver) => ({ version: ver, stable: true }));
            break;
          }
        }

        if (!cancelled) {
          setVersions(result);
          // Auto-select first stable version if nothing selected
          if (!value && result.length > 0) {
            const stable = result.find((x) => x.stable) || result[0];
            onChange(stable.version);
          }
        }
      } catch (err) {
        if (!cancelled) setError(String(err));
      }
      if (!cancelled) setLoading(false);
    };

    fetchVersions();
    return () => { cancelled = true; };
  }, [loader, gameVersion]);

  if (loader === "vanilla") return null;

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-slate-400">
        <Loader2 size={14} className="animate-spin" />
        Fetching {loader} versions for {gameVersion}...
      </div>
    );
  }

  if (error) {
    return (
      <p className="text-sm text-red-400">
        Failed to load {loader} versions: {error}
      </p>
    );
  }

  if (versions.length === 0) {
    return (
      <p className="text-sm text-slate-500">
        No {loader} versions found for {gameVersion}
      </p>
    );
  }

  return (
    <select
      value={value || ""}
      onChange={(e) => onChange(e.target.value)}
      className="w-full bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
    >
      {versions.map((v) => (
        <option key={v.version} value={v.version}>
          {v.version}
          {v.stable ? " (stable)" : " (beta)"}
        </option>
      ))}
    </select>
  );
}
