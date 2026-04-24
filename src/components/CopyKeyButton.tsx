import { useState } from 'react';

interface CopyKeyButtonProps {
  mapKey: string;
}

export default function CopyKeyButton({ mapKey }: CopyKeyButtonProps) {
  const [copied, setCopied] = useState(false);

  function handleClick() {
    navigator.clipboard.writeText(`!bsr ${mapKey}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  return (
    <>
      <button
        onClick={handleClick}
        className="inline-flex items-center justify-center rounded w-5 h-5 text-xs font-semibold leading-none text-bs-accent-purple bg-bs-accent-purple/20 hover:bg-bs-accent-purple/40 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-100 border border-bs-accent-purple/40"
        title="复制 !bsr 命令"
      >
        !
      </button>
      <span
        className={`inline-flex items-center justify-center rounded h-5 px-2 text-xs font-semibold leading-none text-bs-accent-purple bg-bs-accent-purple/20 transition-opacity duration-100 border border-bs-accent-purple/40 ${
          copied ? 'opacity-100' : 'opacity-0'
        }`}
      >
        !bsr 已复制
      </span>
    </>
  );
}
