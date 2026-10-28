interface CopyKeyButtonProps {
  mapKey: string;
  onButtonClick: Function
}

export default function CopyKeyButton({ mapKey, onButtonClick }: CopyKeyButtonProps) {
  function handleClick() {
    navigator.clipboard.writeText(`!bsr ${mapKey}`);
    onButtonClick();
  }

  return (
    <button
      onClick={handleClick}
      className="inline-flex items-center justify-center rounded w-5 h-5 text-xs font-semibold leading-none text-bs-accent-purple bg-bs-accent-purple/20 hover:bg-bs-accent-purple/40 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-100 border border-bs-accent-purple/40"
      title="复制 !bsr 命令"
    >
      !
    </button>
  );
}
