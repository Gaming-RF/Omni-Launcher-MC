import { useParams, useNavigate } from "react-router-dom";
import { useInstancesStore } from "../stores/instances";
import { InstanceDetail } from "./InstanceDetail";
import { useEffect } from "react";
import { ArrowLeft } from "lucide-react";

export function InstanceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const instances = useInstancesStore((s) => s.instances);
  const loadInstances = useInstancesStore((s) => s.fetchInstances);

  useEffect(() => {
    if (instances.length === 0) {
      loadInstances();
    }
  }, [instances.length, loadInstances]);

  const instance = instances.find((i) => i.id === id);

  if (!instance) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-4">
        <p className="text-zinc-400">Instance not found.</p>
        <button
          onClick={() => navigate("/")}
          className="flex items-center gap-2 px-4 py-2 bg-zinc-800 rounded hover:bg-zinc-700"
        >
          <ArrowLeft size={16} />
          Back to Home
        </button>
      </div>
    );
  }

  return <InstanceDetail instance={instance} onBack={() => navigate("/")} />;
}
