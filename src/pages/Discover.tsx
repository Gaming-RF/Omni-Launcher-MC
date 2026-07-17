import { useState } from "react";
import PageContainer from "../components/layout/PageContainer";
import SearchInput from "../components/common/SearchInput";

export default function Discover() {
  const [search, setSearch] = useState("");

  return (
    <PageContainer title="Discover">
      <div className="mb-6">
        <SearchInput
          value={search}
          onChange={setSearch}
          placeholder="Search mods and modpacks from Modrinth and CurseForge..."
        />
      </div>

      {/* Filters */}
      <div className="mb-6 flex flex-wrap gap-3">
        {["All", "Mods", "Modpacks", "Resource Packs", "Shaders"].map(
          (filter) => (
            <button
              key={filter}
              className="rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-1.5 text-xs font-medium text-zinc-400 transition-colors hover:border-zinc-700 hover:text-zinc-200"
            >
              {filter}
            </button>
          ),
        )}
      </div>

      {/* Results */}
      <div className="flex h-96 items-center justify-center text-zinc-500">
        <div className="text-center">
          <p className="text-lg">Mod browsing coming in Phase 2</p>
          <p className="mt-2 text-sm">
            Will search both Modrinth and CurseForge simultaneously
          </p>
        </div>
      </div>
    </PageContainer>
  );
}
