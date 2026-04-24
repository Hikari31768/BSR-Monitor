import { Inbox } from 'lucide-react';

interface EmptyStateProps {
  message: string;
  icon?: React.ReactNode;
}

export default function EmptyState({ message, icon }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center p-12 animate-fade-in" role="status" aria-live="polite">
      <div className="w-20 h-20 rounded-2xl bg-bs-bg-tertiary flex items-center justify-center text-bs-text-muted mb-4 ring-2 ring-bs-border">
        {icon ?? <Inbox size={32} />}
      </div>
      <p className="text-bs-text-muted text-lg">{message}</p>
    </div>
  );
}