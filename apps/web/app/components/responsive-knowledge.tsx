import { useEffect, useRef, useState, type ReactNode } from "react";

/** Keep one mounted knowledge view: a desktop sidebar or a mobile modal drawer. */
export function ResponsiveKnowledge({
  open,
  onDismiss,
  labelledBy,
  children,
}: {
  open: boolean;
  onDismiss: () => void;
  labelledBy: string;
  children: ReactNode;
}) {
  const [mobile, setMobile] = useState(false);
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const query = matchMedia("(max-width: 650px)");
    const update = () => setMobile(query.matches);
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, []);
  useEffect(() => {
    const element = dialog.current;
    if (!element) return;
    if (mobile && open) {
      if (!element.open) element.showModal();
    } else element.close();
  }, [mobile, open]);
  return (
    <>
      <aside className="knowledge" aria-labelledby={labelledBy}>
        {!mobile && children}
      </aside>
      <dialog
        ref={dialog}
        className="knowledge knowledge-sheet"
        aria-labelledby={labelledBy}
        onClose={() => {
          if (mobile && open && !dialog.current?.open) onDismiss();
        }}
        onClick={(event) => {
          const element = dialog.current;
          if (!element || event.target !== element) return;
          const rect = element.getBoundingClientRect();
          if (
            event.clientX < rect.left ||
            event.clientX > rect.right ||
            event.clientY < rect.top ||
            event.clientY > rect.bottom
          )
            element.close();
        }}
      >
        {mobile && children}
      </dialog>
    </>
  );
}
