import { useAuthStore } from "../stores/auth";
import type { AccountInfo } from "../lib/tauri";

interface UseActiveAccountResult {
  account: AccountInfo | null;
  hasAccount: boolean;
}

export function useActiveAccount(): UseActiveAccountResult {
  const account = useAuthStore((s) => s.activeAccount);
  return {
    account,
    hasAccount: account !== null,
  };
}
