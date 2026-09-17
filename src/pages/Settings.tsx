import { useState, useEffect, useRef, useCallback } from "react";
import { useSettingsStore } from "../stores/settings";
import {
  startLogin,
  pollLogin,
  cancelLogin,
  startDeviceLogin,
  pollDeviceLogin,
  extractErrorMessage,
} from "../lib/tauri";
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
  Download,
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
  const t = useI18nStore((s) => s.t);

  const [userCode, setUserCode] = useState<string | null>(null);
  const [verificationUri, setVerificationUri] = useState<string | null>(null);
  const [authorizationUrl, setAuthorizationUrl] = useState<string | null>(null);
  const [loginMethod, setLoginMethod] = useState<"browser" | "device" | null>(null);
  const [loginStatus, setLoginStatus] = useState<"idle" | "pending" | "success" | "error">("idle");
  const [loginError, setLoginError] = useState<string | null>(null);
  const mountedRef = useRef(true);
  const cancelRequestedRef = useRef(false);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const finishLogin = useCallback(async () => {
    await fetchAccounts();
    if (!mountedRef.current) return;
    setLoginStatus("success");
    setLoginMethod(null);
    setUserCode(null);
    setVerificationUri(null);
    setAuthorizationUrl(null);
  }, [fetchAccounts]);

  const handleBrowserLogin = useCallback(async () => {
    cancelRequestedRef.current = false;
    setLoginStatus("pending");
    setLoginMethod("browser");
    setLoginError(null);
    setAuthorizationUrl(null);
    try {
      const response = await startLogin();
      if (!mountedRef.current) return;
      setAuthorizationUrl(response.authorization_url);
      await pollLogin();
      await finishLogin();
    } catch (err) {
      if (!mountedRef.current || cancelRequestedRef.current) return;
      setLoginStatus("error");
      setLoginError(extractErrorMessage(err));
    }
  }, [finishLogin]);

  const handleDeviceLogin = useCallback(async () => {
    cancelRequestedRef.current = false;
    setLoginStatus("pending");
    setLoginMethod("device");
    setLoginError(null);
    try {
      const response = await startDeviceLogin();
      if (!mountedRef.current) return;
      setUserCode(response.user_code);
      setVerificationUri(response.verification_uri);

      const deadline = Date.now() + response.expires_in * 1000;
      let interval = response.interval;
      while (mountedRef.current && Date.now() < deadline) {
        try {
          await pollDeviceLogin();
          await finishLogin();
          return;
        } catch (err) {
          const msg = extractErrorMessage(err);
          if (!msg.includes("authorization_pending") && !msg.includes("slow_down")) {
            throw err;
          }
          if (msg.includes("slow_down")) interval += 5;
          await new Promise((resolve) => setTimeout(resolve, interval * 1000));
        }
      }
      throw new Error(t("settings.loginExpired"));
    } catch (err) {
      if (!mountedRef.current || cancelRequestedRef.current) return;
      setLoginStatus("error");
      setLoginError(extractErrorMessage(err));
    }
  }, [finishLogin, t]);

  const handleCancelLogin = useCallback(async () => {
    cancelRequestedRef.current = true;
    await cancelLogin();
    if (!mountedRef.current) return;
    setLoginStatus("idle");
    setLoginMethod(null);
    setUserCode(null);
    setVerificationUri(null);
    setAuthorizationUrl(null);
  }, []);

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
      <h1 className="text-2xl font-bold text-white">{t("settings.title")}</h1>

      {/* Account Section */}
      <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
        <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
          <User size={20} />
          {t("settings.account")}
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
            <p className="text-white font-medium mb-1">{t("settings.noAccount")}</p>
            <p className="text-slate-400 text-sm mb-4">
              {t("settings.accountDescription")}
            </p>
          </div>
        ) : null}

        {loginStatus === "pending" && loginMethod === "browser" ? (
          <div className="bg-slate-900 rounded-lg p-4 space-y-3">
            <div className="flex items-center gap-2 text-slate-300 text-sm">
              <Loader2 size={14} className="animate-spin" />
              {t("settings.browserWaiting")}
            </div>
            {authorizationUrl && (
              <a
                href={authorizationUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 text-blue-400 hover:text-blue-300 text-sm"
              >
                <ExternalLink size={14} />
                {t("settings.openMicrosoft")}
              </a>
            )}
            <button
              onClick={handleCancelLogin}
              className="text-slate-400 hover:text-white text-sm underline"
            >
              {t("common.cancel")}
            </button>
          </div>
        ) : loginStatus === "pending" && loginMethod === "device" && userCode ? (
          <div className="bg-slate-900 rounded-lg p-4 space-y-3">
            <p className="text-slate-300 text-sm">{t("settings.deviceInstructions")}</p>
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
              {t("settings.waitingForAuthorization")}
            </div>
            <button
              onClick={handleCancelLogin}
              className="text-slate-400 hover:text-white text-sm underline"
            >
              {t("common.cancel")}
            </button>
          </div>
        ) : (
          <div className="space-y-2">
            <button
              onClick={handleBrowserLogin}
              disabled={loginStatus === "pending"}
              className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            >
              {activeAccount ? t("settings.addAccount") : t("settings.signIn")}
            </button>
            <button
              onClick={handleDeviceLogin}
              disabled={loginStatus === "pending"}
              className="text-slate-300 hover:text-white disabled:opacity-50 text-sm underline"
            >
              {t("settings.useDeviceCode")}
            </button>
            {loginStatus === "success" && (
              <p className="text-emerald-400 text-sm flex items-center gap-1">
                <CheckCircle size={14} /> {t("settings.signedIn")}
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
    "idle" | "checking" | "available" | "none" | "error" | "installing"
  >("idle");
  const [latestVersion, setLatestVersion] = useState<string | null>(null);

  const currentVersion = __APP_VERSION__;

  const checkForUpdate = async () => {
    setUpdateStatus("checking");
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        setLatestVersion(update.version);
        setUpdateStatus("available");
      } else {
        setUpdateStatus("none");
      }
    } catch (err) {
      console.error("Update check failed:", err);
      setUpdateStatus("error");
    }
  };

  const installUpdate = async () => {
    setUpdateStatus("installing");
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const { relaunch } = await import("@tauri-apps/plugin-process");
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (err) {
      console.error("Update install failed:", err);
      setUpdateStatus("error");
    }
  };

  return (
    <section className="bg-slate-800 rounded-xl p-5 border border-slate-700">
      <h2 className="flex items-center gap-2 text-lg font-semibold text-white mb-4">
        <RefreshCw size={20} />
        App Updates
      </h2>
      <p className="text-sm text-slate-400 mb-4">
        OmniLauncherMC v{currentVersion} — Auto-updates via Tauri updater.
      </p>

      <div className="flex items-center gap-3">
        <button
          onClick={checkForUpdate}
          disabled={updateStatus === "checking" || updateStatus === "installing"}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
        >
          {updateStatus === "checking" ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
          Check for Updates
        </button>

        {(updateStatus === "available" || updateStatus === "installing") && (
          <button
            onClick={installUpdate}
            disabled={updateStatus === "installing"}
            className="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
          >
            {updateStatus === "installing" ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Download size={14} />
            )}
            {updateStatus === "installing" ? "Installing..." : `Install v${latestVersion}`}
          </button>
        )}
      </div>

      {updateStatus === "available" && latestVersion && (
        <p className="text-sm text-emerald-400 mt-3">
          Update available: v{latestVersion} (current: v{currentVersion})
        </p>
      )}
      {updateStatus === "none" && (
        <p className="text-sm text-slate-500 mt-3">You're on the latest version (v{currentVersion}).</p>
      )}
      {updateStatus === "error" && (
        <p className="text-sm text-red-400 mt-3">
          Unable to check for updates. Try{" "}
          <a href="https://github.com/Gaming-RF/Omni-Launcher-MC/releases" target="_blank" rel="noreferrer" className="underline">
            GitHub Releases
          </a>.
        </p>
      )}
      {updateStatus === "installing" && (
        <p className="text-sm text-blue-400 mt-3">
          Downloading update... the app will restart automatically.
        </p>
      )}
    </section>
  );
}
