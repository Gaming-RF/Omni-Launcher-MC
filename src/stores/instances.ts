import { create } from "zustand";
import type { InstanceListItem } from "../lib/tauri";
import * as tauri from "../lib/tauri";

interface InstancesState {
  instances: InstanceListItem[];
  loading: boolean;
  error: string | null;

  fetchInstances: () => Promise<void>;
  createInstance: (payload: tauri.CreateInstancePayload) => Promise<InstanceListItem>;
  deleteInstance: (id: string) => Promise<void>;
  launchGame: (instanceId: string) => Promise<void>;
  launchGameOffline: (instanceId: string, username: string) => Promise<void>;
  clearError: () => void;
}

export const useInstancesStore = create<InstancesState>((set, get) => ({
  instances: [],
  loading: false,
  error: null,

  fetchInstances: async () => {
    set({ loading: true, error: null });
    try {
      const instances = await tauri.getInstances();
      set({ instances, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  createInstance: async (payload) => {
    set({ error: null });
    try {
      const instance = await tauri.createInstance(payload);
      set({ instances: [instance, ...get().instances] });
      return instance;
    } catch (err) {
      set({ error: String(err) });
      throw err;
    }
  },

  deleteInstance: async (id: string) => {
    try {
      await tauri.deleteInstance(id);
      set({ instances: get().instances.filter((i) => i.id !== id) });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  launchGame: async (instanceId: string) => {
    set({ error: null });
    try {
      // Prepare (download assets etc.) then launch
      await tauri.prepareInstance(instanceId);
      const pid = await tauri.launchGame(instanceId);
      console.log(`Game launched with PID ${pid}`);
    } catch (err) {
      set({ error: String(err) });
    }
  },

  launchGameOffline: async (instanceId: string, username: string) => {
    set({ error: null });
    try {
      await tauri.prepareInstance(instanceId);
      const pid = await tauri.launchGameOffline(instanceId, username);
      console.log(`Game launched offline as ${username} (PID ${pid})`);
    } catch (err) {
      set({ error: String(err) });
    }
  },

  clearError: () => set({ error: null }),
}));
