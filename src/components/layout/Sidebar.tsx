import { NavLink } from "react-router-dom";
import {
  Home,
  Compass,
  Settings,
  User,
  Download,
} from "lucide-react";
import { useAuthStore } from "../../stores/auth";

const navItems = [
  { to: "/", icon: Home, label: "Home" },
  { to: "/discover", icon: Compass, label: "Discover" },
  { to: "/import", icon: Download, label: "Import" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Sidebar() {
  const activeAccount = useAuthStore((s) => s.activeAccount);

  return (
    <aside className="flex w-56 flex-col border-r border-zinc-800 bg-zinc-900">
      {/* Logo */}
      <div className="flex items-center gap-2 px-5 py-4">
        <div className="h-8 w-8 rounded-lg bg-green-500" />
        <span className="text-lg font-bold tracking-tight">OmniMC</span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 px-3">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-zinc-800 text-white"
                  : "text-zinc-400 hover:bg-zinc-800/50 hover:text-zinc-200"
              }`
            }
          >
            <Icon className="h-4 w-4" />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Account */}
      <div className="border-t border-zinc-800 px-4 py-3">
        {activeAccount ? (
          <div className="flex items-center gap-3">
            <img
              src={`https://mc-heads.net/avatar/${activeAccount.uuid}/32`}
              alt=""
              className="h-8 w-8 rounded"
            />
            <div className="min-w-0">
              <p className="truncate text-sm font-medium">
                {activeAccount.username}
              </p>
              <p className="text-xs text-zinc-500">Logged in</p>
            </div>
          </div>
        ) : (
          <button className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200">
            <User className="h-4 w-4" />
            Sign in
          </button>
        )}
      </div>
    </aside>
  );
}
