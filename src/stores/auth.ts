import { create } from "zustand";
import type { Account } from "../lib/tauri";
import { listAccounts } from "../lib/tauri";

interface AuthState {
  accounts: Account[];
  activeAccount: Account | null;
  isLoading: boolean;
  loginDeviceCode: string | null;
  loginUrl: string | null;

  fetchAccounts: () => Promise<void>;
  setActiveAccount: (account: Account | null) => void;
  setLoginPending: (userCode: string, url: string) => void;
  clearLoginPending: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  accounts: [],
  activeAccount: null,
  isLoading: false,
  loginDeviceCode: null,
  loginUrl: null,

  fetchAccounts: async () => {
    set({ isLoading: true });
    try {
      const accounts = await listAccounts();
      set({
        accounts,
        activeAccount: accounts[0] ?? null,
        isLoading: false,
      });
    } catch {
      set({ isLoading: false });
    }
  },

  setActiveAccount: (account) => set({ activeAccount: account }),

  setLoginPending: (userCode, url) =>
    set({ loginDeviceCode: userCode, loginUrl: url }),

  clearLoginPending: () =>
    set({ loginDeviceCode: null, loginUrl: null }),
}));
