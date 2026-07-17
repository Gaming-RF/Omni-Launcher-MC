import { create } from "zustand";

interface SettingsState {
  javaPath: string | null;
  maxMemoryMb: number;
  gameDir: string;
  theme: "dark" | "light";
  curseforgeApiKey: string | null;

  setJavaPath: (path: string | null) => void;
  setMaxMemoryMb: (mb: number) => void;
  setGameDir: (dir: string) => void;
  setTheme: (theme: "dark" | "light") => void;
  setCurseforgeApiKey: (key: string | null) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  javaPath: null,
  maxMemoryMb: 4096,
  gameDir: "",
  theme: "dark",
  curseforgeApiKey: null,

  setJavaPath: (path) => set({ javaPath: path }),
  setMaxMemoryMb: (mb) => set({ maxMemoryMb: mb }),
  setGameDir: (dir) => set({ gameDir: dir }),
  setTheme: (theme) => set({ theme }),
  setCurseforgeApiKey: (key) => set({ curseforgeApiKey: key }),
}));
