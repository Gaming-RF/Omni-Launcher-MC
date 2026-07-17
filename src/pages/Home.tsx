import { useEffect, useState } from "react";
import { Plus } from "lucide-react";
import { useInstancesStore } from "../stores/instances";
import PageContainer from "../components/layout/PageContainer";
import InstanceCard from "../components/instances/InstanceCard";
import InstanceCreator from "../components/instances/InstanceCreator";
import Button from "../components/common/Button";
import { createInstance, deleteInstance } from "../lib/tauri";

// Placeholder version list until API is wired up
const PLACEHOLDER_VERSIONS = [
  "1.21.4",
  "1.21.3",
  "1.21.2",
  "1.21.1",
  "1.20.6",
  "1.20.4",
  "1.20.2",
  "1.20.1",
];

export default function Home() {
  const { instances, isLoading, fetchInstances } = useInstancesStore();
  const [showCreator, setShowCreator] = useState(false);

  useEffect(() => {
    fetchInstances();
  }, [fetchInstances]);

  const handleCreate = async (params: {
    name: string;
    game_version: string;
    mod_loader?: string;
  }) => {
    try {
      await createInstance(params);
      await fetchInstances();
    } catch (err) {
      console.error("Failed to create instance:", err);
    }
  };

  const handlePlay = (id: string) => {
    console.log("Play instance:", id);
    // TODO: Trigger game download + launch
  };

  const handleSettings = (id: string) => {
    console.log("Settings for instance:", id);
    // TODO: Navigate to instance detail/settings page
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteInstance(id);
      await fetchInstances();
    } catch (err) {
      console.error("Failed to delete instance:", err);
    }
  };

  return (
    <PageContainer title="Home">
      <div className="mb-6 flex items-center justify-between">
        <p className="text-sm text-zinc-400">
          {instances.length} instance{instances.length !== 1 ? "s" : ""}
        </p>
        <Button onClick={() => setShowCreator(true)}>
          <Plus className="h-4 w-4" />
          New Instance
        </Button>
      </div>

      {isLoading ? (
        <div className="flex h-64 items-center justify-center text-zinc-500">
          Loading instances...
        </div>
      ) : instances.length === 0 ? (
        <div className="flex h-64 flex-col items-center justify-center gap-3 text-zinc-500">
          <p className="text-lg">No instances yet</p>
          <p className="text-sm">
            Create your first instance to start playing
          </p>
          <Button onClick={() => setShowCreator(true)}>
            <Plus className="h-4 w-4" />
            Create Instance
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {instances.map((instance) => (
            <InstanceCard
              key={instance.id}
              instance={instance}
              onPlay={handlePlay}
              onSettings={handleSettings}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}

      <InstanceCreator
        isOpen={showCreator}
        onClose={() => setShowCreator(false)}
        onSubmit={handleCreate}
        versions={PLACEHOLDER_VERSIONS}
      />
    </PageContainer>
  );
}
