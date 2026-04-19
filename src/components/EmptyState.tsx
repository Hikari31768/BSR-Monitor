interface EmptyStateProps {
  message: string;
  icon?: string;
}

export default function EmptyState({ message, icon = '📭' }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center p-12 animate-fade-in">
      <div className="w-20 h-20 rounded-2xl bg-[#1e1e28] flex items-center justify-center text-4xl mb-4 ring-2 ring-[#2a2a3a]">
        {icon}
      </div>
      <p className="text-[#6b7280] text-lg">{message}</p>
    </div>
  );
}