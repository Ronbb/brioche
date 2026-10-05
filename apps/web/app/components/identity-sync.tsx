import { useEffect, useRef } from "react";
import type { UserProfile } from "@brioche/contracts/UserProfile";
import { watchIdentity } from "../lib/identity-sync";
import { clearLearningDrafts } from "../lib/learning-draft";
import { useLearning } from "./learning";

export function IdentitySync({
  user,
  enabled,
}: {
  user: UserProfile | null;
  enabled: boolean;
}) {
  const learning = useLearning();
  const current = useRef(learning);
  current.current = learning;
  useEffect(() => {
    if (!enabled) return;
    return watchIdentity({
      identity: user ? { id: user.id, role: user.role } : null,
      window,
      document,
      visible: () => document.visibilityState === "visible",
      read: async (signal) => {
        const response = await fetch("/api/v1/me", {
          cache: "no-store",
          signal: AbortSignal.any([signal, AbortSignal.timeout(10000)]),
        });
        if (response.status === 401) return null;
        if (!response.ok) throw Error("Identity unavailable");
        return (await response.json()) as UserProfile;
      },
      stopPlayback: () => current.current.stop(),
      invalidate: () => {
        if (user) clearLearningDrafts(user.id);
        window.location.reload();
      },
      every: (callback) => {
        const timer = window.setInterval(callback, 30000);
        return () => window.clearInterval(timer);
      },
    });
  }, [enabled, user?.id, user?.role]);
  return null;
}
