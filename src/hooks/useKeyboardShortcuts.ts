import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

/**
 * Registers global keyboard shortcuts.
 * - Ctrl/Cmd + N: open instance creator
 * - Ctrl/Cmd + K: focus search input on Home page
 * - Ctrl/Cmd + L: launch the first/selected instance
 * - Ctrl/Cmd + ,: navigate to settings
 */
export function useKeyboardShortcuts() {
  const navigate = useNavigate();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Only respond to Ctrl or Cmd held
      if (!(e.ctrlKey || e.metaKey)) return;

      // Don't intercept if user is typing in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") {
        // Allow Ctrl+K to work even in inputs (blur then focus search)
        if (e.key !== "k" && e.key !== "K") return;
      }

      switch (e.key) {
        case "n":
        case "N":
          e.preventDefault();
          window.dispatchEvent(new CustomEvent("open-instance-creator"));
          break;

        case "k":
        case "K":
          e.preventDefault();
          window.dispatchEvent(new CustomEvent("focus-search"));
          break;

        case "l":
        case "L":
          e.preventDefault();
          window.dispatchEvent(new CustomEvent("launch-instance"));
          break;

        case ",":
          e.preventDefault();
          navigate("/settings");
          break;

        default:
          break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);
}
