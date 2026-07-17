import { type ReactNode } from "react";

interface Props {
  title?: string;
  children: ReactNode;
}

export default function PageContainer({ title, children }: Props) {
  return (
    <div className="mx-auto max-w-6xl">
      {title && (
        <h1 className="mb-6 text-2xl font-bold tracking-tight">{title}</h1>
      )}
      {children}
    </div>
  );
}
