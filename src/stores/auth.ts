import { create } from "zustand";
import type { AccountInfo } from "../lib/tauri";
import * as tauri from "../lib/tauri";

interface AuthState {
  accounts: AccountInfo[];
  activeAccount: AccountInfo | null;
  loading: boolean;
  error: string | null;

  fetchAccounts: () => Promise<void>;
  removeAccount: (uuid: string) => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  accounts: [],
  activeAccount: null,
  loading: false,
  error: null,

  fetchAccounts: async () => {
    set({ loading: true, error: null });
    try {
      const accounts = await tauri.getAccounts();
      set({
        accounts,
        activeAccount: accounts[0] ?? null,
        loading: false,
      });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  removeAccount: async (uuid: string) => {
    try {
      await tauri.removeAccount(uuid);
      const { accounts } = get();
      const remaining = accounts.filter((a) => a.uuid !== uuid);
      set({
        accounts: remaining,
        activeAccount: remaining[0] ?? null,
      });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clearError: () => set({ error: null }),
}));
