import { useAuthStore } from "../stores/auth";
import { useInstancesStore } from "../stores/instances";
import { useSettingsStore } from "../stores/settings";

/**
 * Initialize app state on mount. Fetches accounts, instances, and settings
 * from the Rust backend via Tauri commands.
 */
export function useAppInit() {
  const fetchAccounts = useAuthStore((s) => s.fetchAccounts);
  const fetchInstances = useInstancesStore((s) => s.fetchInstances);
  const fetchSettings = useSettingsStore((s) => s.fetchSettings);

  const init = async () => {
    await Promise.allSettled([
      fetchAccounts(),
      fetchInstances(),
      fetchSettings(),
    ]);
  };

  return { init };
}
