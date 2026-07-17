import { useState, useEffect, useRef, useCallback } from "react";
import { useSettingsStore } from "../stores/settings";
import { startLogin, pollLogin } from "../lib/tauri";
import { useAuthStore } from "../stores/auth";
import {
  User,
  Key,
  Monitor,
  Cpu,
  ExternalLink,
  CheckCircle,
  AlertCircle,
  Loader2,
  LogIn,
} from "lucide-react";
import JavaPicker from "../components/common/JavaPicker";

export function Settings() {
  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const activeAccount = useAuthStore((s) => s.activeAccount);
  const fetchAccounts = useAuthStore((s) => s.fetchAccounts);

  const [userCode, setUserCode] = useState<string | null>(null);
  const [verificationUri, setVerificationUri] = useState<string | null>(null);
  const [loginStatus, setLoginStatus] = useState<"idle" | "pending" | "success" | "error">("idle");
  const [loginError, setLoginError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const handleLogin = useCallback(async () => {
    setLoginStatus("pending");
    setLoginError(null);
    try {
      const response = await startLogin();
      if (!mountedRef.current) return;
      setUserCode(response.user_code);
      setVerificationUri(response.verification_uri);

      // Start polling with unmount-safe scheduling
      const poll = async () => {
        if (!mountedRef.current) return;
        try {
          await pollLogin();
          if (!mountedRef.current) return;
          fetchAccounts();
          setLoginStatus("success");
          setUserCode(null);
          setVerificationUri(null);
        } catch (err) {
          if (!mountedRef.current) return;
          const msg = String(err);
          if (msg.includes("authorization_pending") || msg.includes("slow_down")) {
            setTimeout(poll, 5000);
          } else {
            setLoginStatus("error");
            setLoginError(msg);
          }
        }
      };
      setTimeout(poll, 5000);
    } catch (err) {
      if (!mountedRef.current) return;
      setLoginStatus("error");
      setLoginError(String(err));
    }
  }, [fetchAccounts]);

  const [javaPath, setJavaPath] = useState(settings?.java_path ?? "");
  const [cfKey, setCfKey] = useState(settings?.curseforge_api_key ?? "");
  const [memory, setMemory] = useState(settings?.default_memory_mb ?? "4096");

  useEffect(() => {
    if (settings) {
      setJavaPath(settings.java_path ?? "");
      setCfKey(settings.curseforge_api_key ?? "");
      setMemory(settings.default_memory_mb);
    }
  }, [settings]);

  return (
    <div className="max-w-2xl space-y-8">
      <h1 className="text-2xl font-bold text-white">Settings</h1>

      {/* Account Section */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          <User size={20} />
          Account
        </h2>

        {activeAccount ? (
          <div className="flex items-center gap-3 bg-slate-900 rounded-lg p-3 mb-4">
            <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center text-white font-bold">
              {activeAccount.username.charAt(0)}
            </div>
            <div>
              <p className="text-white font-medium">{activeAccount.username}</p>
              <p className="text-xs text-slate-400 font-mono">{activeAccount.uuid}</p>
            </div>
            <CheckCircle size={16} className="ml-auto text-emerald-400" />
          </div>
        ) : loginStatus === "idle" ? (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <div className="w-14 h-14 rounded-full bg-blue-600/15 flex items-center justify-center mb-4">
              <LogIn size={24} className="text-blue-400" />
            </div>
            <p className="text-white font-medium mb-1">No account signed in</p>
            <p className="text-slate-400 text-sm mb-4">
              Sign in with your Microsoft account to launch Minecraft
            </p>
          </div>
        ) : null}

        {loginStatus === "pending" && userCode ? (
          <div className="bg-slate-900 rounded-lg p-4 space-y-3">
            <p className="text-slate-300 text-sm">Enter this code in your browser:</p>
            <p className="text-2xl font-mono font-bold text-white tracking-wider bg-slate-800 px-4 py-2 rounded text-center">
              {userCode}
            </p>
            <a
              href={verificationUri ?? "https://www.microsoft.com/link"}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-blue-400 hover:text-blue-300 text-sm"
            >
              <ExternalLink size={14} />
              {verificationUri}
            </a>
            <div className="flex items-center gap-2 text-slate-400 text-sm">
              <Loader2 size={14} className="animate-spin" />
              Waiting for you to authorize...
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            <button
              onClick={handleLogin}
              disabled={loginStatus === "pending"}
              className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            >
              {activeAccount ? "Add Another Account" : "Sign in with Microsoft"}
            </button>
            {loginStatus === "success" && (
              <p className="text-emerald-400 text-sm flex items-center gap-1">
                <CheckCircle size={14} /> Signed in successfully!
              </p>
            )}
            {loginStatus === "error" && (
              <p className="text-red-400 text-sm flex items-center gap-1">
                <AlertCircle size={14} /> {loginError}
              </p>
            )}
          </div>
        )}
      </section>

      {/* Java Section */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          <Cpu size={20} />
          Java
        </h2>
        <JavaPicker
          value={javaPath || null}
          onChange={(path) => {
            setJavaPath(path ?? "");
            updateSetting("java_path", path ?? "");
          }}
        />
      </section>

      {/* Memory Section */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          <Monitor size={20} />
          Performance
        </h2>
        <label className="block text-sm text-slate-300 mb-1">
          Default Allocated Memory (MB)
        </label>
        <div className="flex gap-2">
          <input
            type="number"
            min={512}
            max={16384}
            step={512}
            value={memory}
            onChange={(e) => setMemory(e.target.value)}
            className="w-32 bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
          />
          <button
            onClick={() => updateSetting("default_memory_mb", memory)}
            className="bg-slate-700 hover:bg-slate-600 text-white px-4 py-2 rounded-lg text-sm"
          >
            Save
          </button>
        </div>
      </section>

      {/* CurseForge API Key */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          <Key size={20} />
          CurseForge API Key
        </h2>
        <p className="text-sm text-slate-400 mb-3">
          Required for CurseForge mod browsing. Get a free key at{" "}
          <a
            href="https://console.curseforge.com/"
            target="_blank"
            className="text-blue-400 hover:underline"
          >
            console.curseforge.com
          </a>
        </p>
        <div className="flex gap-2">
          <input
            type="password"
            value={cfKey}
            onChange={(e) => setCfKey(e.target.value)}
            placeholder="$2a$..."
            className="flex-1 bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500"
          />
          <button
            onClick={() => updateSetting("curseforge_api_key", cfKey)}
            className="bg-slate-700 hover:bg-slate-600 text-white px-4 py-2 rounded-lg text-sm"
          >
            Save
          </button>
        </div>
      </section>
    </div>
  );
}
