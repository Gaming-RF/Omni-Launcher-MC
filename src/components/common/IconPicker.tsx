import { useState } from "react";

const ICONS = [
  "🎮", "⚡", "🔨", "🧵", "⚙️", "🗺️", "🎨", "🏰",
  "🐉", "🌊", "🏔️", "🌲", "🏠", "💎", "🔮", "🎭",
  "🚀", "🌟", "⭐", "🎯", "🔥", "💀", "👾", "🤖",
  "🎪", "🗡️", "🛡️", "⚔️", "🧪", "🌋", "🎃", "🎄",
];

interface Props {
  value: string;
  onChange: (icon: string) => void;
}

export default function IconPicker({ value, onChange }: Props) {
  const [custom, setCustom] = useState("");
  const [showCustom, setShowCustom] = useState(false);

  return (
    <div className="space-y-2">
      <div className="grid grid-cols-8 gap-1.5">
        {ICONS.map((icon) => (
          <button
            key={icon}
            onClick={() => onChange(icon)}
            className={`w-9 h-9 rounded-lg text-xl flex items-center justify-center transition-all ${
              value === icon
                ? "bg-blue-600 ring-2 ring-blue-400 scale-110"
                : "bg-zinc-800 hover:bg-zinc-700"
            }`}
          >
            {icon}
          </button>
        ))}
      </div>
      {!showCustom ? (
        <button
          onClick={() => setShowCustom(true)}
          className="text-xs text-zinc-500 hover:text-zinc-300"
        >
          + Custom emoji
        </button>
      ) : (
        <div className="flex items-center gap-2">
          <input
            value={custom}
            onChange={(e) => setCustom(e.target.value)}
            placeholder="Paste emoji"
            className="bg-zinc-800 text-white px-2 py-1 rounded text-sm w-20 border border-zinc-700"
            maxLength={2}
          />
          <button
            onClick={() => {
              if (custom) onChange(custom);
              setShowCustom(false);
            }}
            className="text-xs bg-blue-600 text-white px-2 py-1 rounded"
          >
            Use
          </button>
        </div>
      )}
    </div>
  );
}
