interface Props {
  value: number; // 0-100
  label?: string;
  showPercent?: boolean;
  size?: "sm" | "md";
}

export default function ProgressBar({
  value,
  label,
  showPercent = true,
  size = "md",
}: Props) {
  const clamped = Math.max(0, Math.min(100, value));
  const height = size === "sm" ? "h-1.5" : "h-2.5";

  return (
    <div className="w-full">
      {(label || showPercent) && (
        <div className="mb-1 flex items-center justify-between text-xs text-zinc-400">
          {label && <span>{label}</span>}
          {showPercent && <span>{Math.round(clamped)}%</span>}
        </div>
      )}
      <div className={`w-full rounded-full bg-zinc-800 ${height}`}>
        <div
          className={`${height} rounded-full bg-green-500 transition-all duration-300`}
          style={{ width: `${clamped}%` }}
        />
      </div>
    </div>
  );
}
