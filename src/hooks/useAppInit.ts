import { useEffect } from "react";
import { useInstancesStore } from "../stores/instances";
import { useAuthStore } from "../stores/auth";

/**
 * Initialize stores on app mount.
 * Fetches accounts and instances from the backend.
 */
export function useAppInit() {
  const fetchInstances = useInstancesStore((s) => s.fetchInstances);
  const fetchAccounts = useAuthStore((s) => s.fetchAccounts);

  useEffect(() => {
    fetchAccounts();
    fetchInstances();
  }, [fetchAccounts, fetchInstances]);
}
