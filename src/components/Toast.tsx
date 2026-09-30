import { useEffect } from "react";

export interface ToastMsg {
  msg: string;
  erro?: boolean;
}

export function Toast({ toast, onClose }: { toast: ToastMsg | null; onClose: () => void }) {
  useEffect(() => {
    if (!toast) return;
    // Erros ficam até serem fechados; avisos normais desaparecem sozinhos.
    if (toast.erro) return;
    const t = setTimeout(onClose, 6000);
    return () => clearTimeout(t);
  }, [toast, onClose]);

  if (!toast) return null;
  return (
    <div
      style={{
        position: "fixed",
        left: "50%",
        bottom: 24,
        transform: "translateX(-50%)",
        zIndex: 40,
        display: "flex",
        alignItems: "center",
        gap: 16,
        padding: "10px 12px 10px 20px",
        background: toast.erro ? "#7F1D1D" : "#101828",
        color: "#fff",
        borderRadius: 3,
        fontSize: 14,
        boxShadow: "var(--shadow-lg)",
        maxWidth: "80vw",
      }}
    >
      <span>{toast.msg}</span>
      <button
        onClick={onClose}
        style={{ border: 0, background: "transparent", color: "#93B4F5", cursor: "pointer", fontSize: 18, minWidth: 36, minHeight: 36 }}
      >
        ×
      </button>
    </div>
  );
}
