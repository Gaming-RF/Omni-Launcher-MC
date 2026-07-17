import { Routes, Route } from "react-router-dom";
import Sidebar from "./components/layout/Sidebar";
import Home from "./pages/Home";
import Discover from "./pages/Discover";
import Settings from "./pages/Settings";

export default function App() {
  return (
    <div className="flex h-full bg-zinc-950 text-zinc-100">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-6">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/discover" element={<Discover />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
