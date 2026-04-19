export default function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center p-12">
      <div className="relative">
        <div className="w-12 h-12 rounded-full border-2 border-[#2a2a3a]"></div>
        <div className="w-12 h-12 rounded-full border-2 border-[#ff6b6b] border-t-transparent animate-spin absolute inset-0"></div>
      </div>
    </div>
  );
}