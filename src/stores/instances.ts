import { create } from "zustand";
import type { Instance } from "../lib/tauri";
import { listInstances } from "../lib/tauri";

interface InstancesState {
  instances: Instance[];
  isLoading: boolean;
  selectedInstance: Instance | null;

  fetchInstances: () => Promise<void>;
  selectInstance: (instance: Instance | null) => void;
}

export const useInstancesStore = create<InstancesState>((set) => ({
  instances: [],
  isLoading: false,
  selectedInstance: null,

  fetchInstances: async () => {
    set({ isLoading: true });
    try {
      const instances = await listInstances();
      set({ instances, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  selectInstance: (instance) => set({ selectedInstance: instance }),
}));
