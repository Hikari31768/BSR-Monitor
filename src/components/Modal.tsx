interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export default function Modal({ isOpen, onClose, title, children }: ModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
      <div className="bg-[#15151c] rounded-2xl p-6 max-w-md w-full mx-4 border border-[#2a2a3a] shadow-2xl animate-slide-up">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-bold text-[#f0f0f5]">{title}</h2>
          <button
            onClick={onClose}
            className="text-[#6b7280] hover:text-[#f0f0f5] transition-colors text-xl"
          >
            ✕
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}