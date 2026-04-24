export default function LoadingSpinner() {
  return (
    <div className="flex items-center justify-center p-12" role="status">
      <div className="relative">
        <div className="w-12 h-12 rounded-full border-2 border-bs-border"></div>
        <div className="w-12 h-12 rounded-full border-2 border-bs-accent-primary border-t-transparent animate-spin absolute inset-0"></div>
      </div>
      <span className="sr-only">加载中...</span>
    </div>
  );
}