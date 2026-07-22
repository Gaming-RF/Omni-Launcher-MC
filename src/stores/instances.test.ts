import { describe, it, expect, vi, beforeEach } from "vitest";
import { useInstancesStore } from "../stores/instances";

// Mock the tauri module
vi.mock("../lib/tauri", () => ({
  getInstances: vi.fn().mockResolvedValue([
    {
      id: "test-1",
      name: "Test Instance",
      game_version: "1.20.1",
      loader: "fabric",
      loader_version: "0.14.0",
      icon: null,
      created_at: "2026-01-01T00:00:00Z",
      last_played: null,
      play_time_secs: 0,
      allocated_memory_mb: 2048,
    },
  ]),
  createInstance: vi.fn().mockResolvedValue({
    id: "new-1",
    name: "New Instance",
    game_version: "1.20.1",
    loader: "vanilla",
    loader_version: null,
    icon: null,
    created_at: "2026-01-02T00:00:00Z",
    last_played: null,
    play_time_secs: 0,
    allocated_memory_mb: 2048,
  }),
  deleteInstance: vi.fn().mockResolvedValue(undefined),
  prepareInstance: vi.fn().mockResolvedValue("ready"),
  launchGame: vi.fn().mockResolvedValue(1234),
  launchGameOffline: vi.fn().mockResolvedValue(1234),
}));

describe("instances store", () => {
  beforeEach(() => {
    // Reset store state
    useInstancesStore.setState({
      instances: [],
      loading: false,
      error: null,
    });
  });

  it("starts with empty state", () => {
    const state = useInstancesStore.getState();
    expect(state.instances).toEqual([]);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("fetches instances successfully", async () => {
    await useInstancesStore.getState().fetchInstances();
    const state = useInstancesStore.getState();
    expect(state.instances).toHaveLength(1);
    expect(state.instances[0].name).toBe("Test Instance");
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("creates an instance and prepends to list", async () => {
    await useInstancesStore.getState().fetchInstances();
    await useInstancesStore.getState().createInstance({
      name: "New Instance",
      game_version: "1.20.1",
      loader: "vanilla",
      loader_version: null,
      icon: null,
      java_args: null,
      allocated_memory_mb: 2048,
    });
    const state = useInstancesStore.getState();
    expect(state.instances).toHaveLength(2);
    expect(state.instances[0].name).toBe("New Instance");
  });

  it("deletes an instance", async () => {
    await useInstancesStore.getState().fetchInstances();
    await useInstancesStore.getState().deleteInstance("test-1");
    const state = useInstancesStore.getState();
    expect(state.instances).toHaveLength(0);
  });

  it("clears error", () => {
    useInstancesStore.setState({ error: "some error" });
    useInstancesStore.getState().clearError();
    expect(useInstancesStore.getState().error).toBeNull();
  });
});
