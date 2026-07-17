import { useState } from "react";
import PageContainer from "../components/layout/PageContainer";
import Button from "../components/common/Button";
import { useSettingsStore } from "../stores/settings";
import { useAuthStore } from "../stores/auth";
import { User, LogOut } from "lucide-react";
import { loginStart, loginPoll } from "../lib/tauri";

export default function Settings() {
  const settings = useSettingsStore();
  const auth = useAuthStore();
  const [loginPending, setLoginPending] = useState(false);
  const [deviceCode, setDeviceCode] = useState<string | null>(null);
  const [userCode, setUserCode] = useState<string | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);

  const handleLogin = async () => {
    setLoginError(null);
    setLoginPending(true);
    try {
      const response = await loginStart();
      setDeviceCode(response.device_code);
      setUserCode(response.user_code);
      // Open verification URL in browser
      window.open(response.verification_uri, "_blank");

      // Poll for completion
      const poll = async () => {
        try {
          const account = await loginPoll(response.device_code);
          auth.fetchAccounts();
          setDeviceCode(null);
          setUserCode(null);
          setLoginPending(false);
        } catch (err) {
          // Keep polling if authorization_pending
          setTimeout(poll, response.interval * 1000);
        }
      };
      setTimeout(poll, response.interval * 1000);
    } catch (err) {
      setLoginError(String(err));
      setLoginPending(false);
    }
  };

  return (
    <PageContainer title="Settings">
      <div className="space-y-8">
        {/* Account Section */}
        <section>
          <h2 className="mb-4 text-lg font-semibold">Account</h2>
          <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            {auth.activeAccount ? (
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <img
                    src={`https://mc-heads.net/avatar/${auth.activeAccount.uuid}/64`}
                    alt=""
                    className="h-12 w-12 rounded-lg"
                  />
                  <div>
                    <p className="font-semibold">
                      {auth.activeAccount.username}
                    </p>
                    <p className="text-xs text-zinc-500">
                      UUID: {auth.activeAccount.uuid}
                    </p>
                  </div>
                </div>
                <Button variant="ghost" size="sm">
                  <LogOut className="h-4 w-4" />
                  Logout
                </Button>
              </div>
            ) : deviceCode ? (
              <div className="text-center">
                <p className="mb-2 text-sm text-zinc-400">
                  Enter this code at Microsoft:
                </p>
                <p className="mb-4 text-3xl font-bold tracking-widest text-white">
                  {userCode}
                </p>
                <p className="text-xs text-zinc-500">
                  Waiting for authentication...
                </p>
              </div>
            ) : (
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3 text-zinc-400">
                  <User className="h-8 w-8" />
                  <div>
                    <p className="font-medium text-zinc-200">
                      Not signed in
                    </p>
                    <p className="text-xs">
                      Sign in with your Microsoft account to play
                    </p>
                  </div>
                </div>
                <Button
                  onClick={handleLogin}
                  disabled={loginPending}
                >
                  Sign in with Microsoft
                </Button>
              </div>
            )}
            {loginError && (
              <p className="mt-3 text-sm text-red-400">{loginError}</p>
            )}
          </div>
        </section>

        {/* Java Section */}
        <section>
          <h2 className="mb-4 text-lg font-semibold">Java</h2>
          <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 space-y-4">
            <div>
              <label className="mb-1.5 block text-sm font-medium text-zinc-300">
                Java Path
              </label>
              <input
                type="text"
                value={settings.javaPath ?? "Auto-detect"}
                readOnly
                className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-400"
              />
            </div>
            <div>
              <label className="mb-1.5 block text-sm font-medium text-zinc-300">
                Max Memory: {settings.maxMemoryMb} MB
              </label>
              <input
                type="range"
                min={1024}
                max={16384}
                step={512}
                value={settings.maxMemoryMb}
                onChange={(e) =>
                  settings.setMaxMemoryMb(Number(e.target.value))
                }
                className="w-full accent-green-500"
              />
              <div className="flex justify-between text-xs text-zinc-500">
                <span>1 GB</span>
                <span>16 GB</span>
              </div>
            </div>
          </div>
        </section>

        {/* CurseForge API Key */}
        <section>
          <h2 className="mb-4 text-lg font-semibold">API Keys</h2>
          <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
            <label className="mb-1.5 block text-sm font-medium text-zinc-300">
              CurseForge API Key
            </label>
            <p className="mb-3 text-xs text-zinc-500">
              Required to browse CurseForge mods. Get your key at{" "}
              <a
                href="https://console.curseforge.com"
                target="_blank"
                rel="noreferrer"
                className="text-green-400 underline"
              >
                console.curseforge.com
              </a>
            </p>
            <input
              type="password"
              value={settings.curseforgeApiKey ?? ""}
              onChange={(e) =>
                settings.setCurseforgeApiKey(e.target.value || null)
              }
              placeholder="Enter your API key"
              className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2.5 text-sm text-zinc-100 placeholder:text-zinc-500 focus:border-green-500 focus:outline-none"
            />
          </div>
        </section>
      </div>
    </PageContainer>
  );
}
