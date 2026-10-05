import { useEffect, useRef, useState } from "react";
import { ApiRequestError, privateRequest } from "../lib/api.client";
import { operationKey } from "../lib/operation-key";
type Job<T> = {
  path: string;
  body: object;
  method: "PUT" | "POST";
  accept: (result: T) => void;
};
export function useOwnedWrite<T>(refresh?: () => Promise<void>) {
  const [saving, setSaving] = useState(false),
    [uncertain, setUncertain] = useState(false),
    [error, setError] = useState("");
  const pending = useRef<Job<T> | null>(null),
    busy = useRef(false),
    alive = useRef(true);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    if (!saving && !uncertain) return;
    const warn = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", warn);
    return () => window.removeEventListener("beforeunload", warn);
  }, [saving, uncertain]);
  async function send(job: Job<T>) {
    if (busy.current || !alive.current) return;
    busy.current = true;
    pending.current = job;
    setSaving(true);
    setError("");
    try {
      const result = await privateRequest<T>(job.path, job.method, job.body);
      if (alive.current) {
        pending.current = null;
        setUncertain(false);
        job.accept(result);
      }
    } catch (failure) {
      if (!alive.current) return;
      if (failure instanceof ApiRequestError && failure.status < 500) {
        pending.current = null;
        setUncertain(false);
        if (failure.status === 409) {
          try {
            await refresh?.();
          } catch {
            /* retain last known state */
          }
        }
        setError(
          failure.status === 409
            ? "其他设备已更新，请确认最新记录后重试。"
            : failure.status === 410
              ? "来源内容已撤回，暂时无法操作。"
              : failure.message,
        );
      } else {
        setUncertain(true);
        setError("保存尚未确认，请重试原提交。");
      }
    } finally {
      busy.current = false;
      if (alive.current) setSaving(false);
    }
  }
  return {
    saving,
    uncertain,
    error,
    blocked: saving || uncertain,
    write: (
      path: string,
      body: object,
      accept: Job<T>["accept"],
      method: "PUT" | "POST" = "PUT",
    ) => {
      if (!pending.current)
        void send({
          path,
          method,
          body: { ...body, idempotencyKey: operationKey() },
          accept,
        });
    },
    retry: () => {
      if (pending.current) void send(pending.current);
    },
  };
}
