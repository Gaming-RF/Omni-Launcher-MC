import { create } from "zustand";
import type { SettingsInfo } from "../lib/tauri";
import * as tauri from "../lib/tauri";

interface SettingsState {
  settings: SettingsInfo | null;
  loading: boolean;
  error: string | null;

  fetchSettings: () => Promise<void>;
  updateSetting: (key: string, value: string) => Promise<void>;
  clearError: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  loading: false,
  error: null,

  fetchSettings: async () => {
    set({ loading: true, error: null });
    try {
      const settings = await tauri.getSettings();
      set({ settings, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  updateSetting: async (key: string, value: string) => {
    try {
      await tauri.updateSetting(key, value);
      // Refresh settings after update
      const settings = await tauri.getSettings();
      set({ settings });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clearError: () => set({ error: null }),
}));
