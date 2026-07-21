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
  RefreshCw,
  Trash2,
} from "lucide-react";
import JavaPicker from "../components/common/JavaPicker";
import { LOCALES } from "../lib/i18n";
import { useI18nStore } from "../stores/i18n";

export function Settings() {
  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const activeAccount = useAuthStore((s) => s.activeAccount);
  const accounts = useAuthStore((s) => s.accounts);
  const fetchAccounts = useAuthStore((s) => s.fetchAccounts);
  const switchAccount = useAuthStore((s) => s.switchAccount);
  const removeAccount = useAuthStore((s) => s.removeAccount);
  const locale = useI18nStore((s) => s.locale);
  const setLocale = useI18nStore((s) => s.setLocale);

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
          <div className="space-y-3 mb-4">
            {/* All accounts list */}
            {accounts.map((account) => (
              <div
                key={account.uuid}
                className={`flex items-center gap-3 rounded-lg p-3 transition-colors ${
                  account.uuid === activeAccount.uuid
                    ? "bg-blue-900/20 border border-blue-800"
                    : "bg-slate-900 hover:bg-slate-850 cursor-pointer"
                }`}
                onClick={() => {
                  if (account.uuid !== activeAccount.uuid) {
                    switchAccount(account.uuid);
                  }
                }}
              >
                {/* Skin head render */}
                <img
                  src={
                    account.skin_url ||
                    `https://mc-heads.net/head/${account.uuid}/64`
                  }
                  alt={account.username}
                  className="w-10 h-10 rounded-lg bg-slate-700"
                  onError={(e) => {
                    // Fallback to initial letter
                    (e.target as HTMLImageElement).style.display = "none";
                  }}
                />
                <div className="flex-1 min-w-0">
                  <p className="text-white font-medium truncate">
                    {account.username}
                  </p>
                  <p className="text-xs text-slate-400 font-mono truncate">
                    {account.uuid}
                  </p>
                </div>
                {account.uuid === activeAccount.uuid ? (
                  <CheckCircle size={16} className="text-emerald-400 shrink-0" />
                ) : (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      removeAccount(account.uuid);
                    }}
                    className="text-slate-500 hover:text-red-400 transition-colors shrink-0"
                    title="Remove account"
                  >
                    <Trash2 size={14} />
                  </button>
                )}
              </div>
            ))}
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

      {/* Language */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          🌐 Language
        </h2>
        <select
          value={locale}
          onChange={(e) => setLocale(e.target.value as typeof locale)}
          className="bg-slate-900 border border-slate-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 w-full max-w-xs"
        >
          {LOCALES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
        <p className="text-xs text-slate-500 mt-2">
          Restart the app for full language effect
        </p>
      </section>

      {/* App Updates */}
      <UpdateChecker />
    </div>
  );
}

function UpdateChecker() {
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "available" | "downloading" | "ready" | "none" | "error"
  >("idle");
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const updateRef = useRef<ReturnType<typeof import("@tauri-apps/plugin-updater").check> extends Promise<infer T> ? T : never>(null);

  const checkForUpdate = async () => {
    setUpdateStatus("checking");
    setError(null);
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        updateRef.current = update as NonNullable<typeof update>;
        setUpdateVersion(update.version);
        setUpdateStatus("available");
      } else {
        setUpdateStatus("none");
      }
    } catch (err) {
      setUpdateStatus("error");
      setError(String(err));
    }
  };

  const installUpdate = async () => {
    if (!updateRef.current) return;
    setUpdateStatus("downloading");
    try {
      await updateRef.current.downloadAndInstall();
      setUpdateStatus("ready");
    } catch (err) {
      setUpdateStatus("error");
      setError(String(err));
    }
  };

  const restartApp = async () => {
    try {
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
      <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
        <RefreshCw size={20} />
        App Updates
      </h2>
      <p className="text-sm text-slate-400 mb-4">
        OmniLauncherMC v{__APP_VERSION__} — Check for the latest version on GitHub.
      </p>

      <div className="flex items-center gap-3">
        <button
          onClick={checkForUpdate}
          disabled={updateStatus === "checking" || updateStatus === "downloading"}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
        >
          {updateStatus === "checking" ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
          Check for Updates
        </button>

        {updateStatus === "available" && (
          <button
            onClick={installUpdate}
            className="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
          >
            <Cpu size={14} />
            Install Update
          </button>
        )}

        {updateStatus === "ready" && (
          <button
            onClick={restartApp}
            className="bg-amber-600 hover:bg-amber-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
          >
            <RefreshCw size={14} />
            Restart Now
          </button>
        )}
      </div>

      {updateStatus === "available" && updateVersion && (
        <p className="text-sm text-emerald-400 mt-3">
          Update available: v{updateVersion}
        </p>
      )}
      {updateStatus === "downloading" && (
        <p className="text-sm text-blue-400 mt-3 flex items-center gap-2">
          <Loader2 size={12} className="animate-spin" /> Downloading update...
        </p>
      )}
      {updateStatus === "ready" && (
        <p className="text-sm text-amber-400 mt-3">
          Update downloaded. Restart to apply.
        </p>
      )}
      {updateStatus === "none" && (
        <p className="text-sm text-slate-500 mt-3">You're on the latest version.</p>
      )}
      {updateStatus === "error" && (
        <p className="text-sm text-red-400 mt-3">
          {error?.includes("network") || error?.includes("fetch")
            ? "Unable to check for updates. Check your internet connection."
            : error}
        </p>
      )}
    </section>
  );
}
