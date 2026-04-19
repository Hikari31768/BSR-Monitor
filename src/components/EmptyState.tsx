interface EmptyStateProps {
  message: string;
  icon?: string;
}

export default function EmptyState({ message, icon = '📭' }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center p-8 text-gray-400">
      <span className="text-4xl mb-2">{icon}</span>
      <p>{message}</p>
    </div>
  );
}