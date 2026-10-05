import { useEffect, useRef, useState } from "react";
import type { LearningSession } from "@brioche/contracts/LearningSession";
import type { LearningState } from "@brioche/contracts/LearningState";
import type { AttemptResult } from "@brioche/contracts/AttemptResult";
import type { HintResult } from "@brioche/contracts/HintResult";
import { ApiRequestError, privateRequest } from "../lib/api.client";
import { operationKey } from "../lib/operation-key";
type Result = LearningState | AttemptResult | HintResult;
type Pending = {
  path: string;
  method: "POST" | "PUT";
  body: object;
  onSaved?: () => void;
};
export function useLearningSession(initial: LearningSession) {
  const [progress, setProgress] = useState(initial.progress),
    [saving, setSaving] = useState(false),
    [error, setError] = useState(""),
    [uncertain, setUncertain] = useState(false);
  const latest = useRef(initial.progress),
    busy = useRef(false),
    pending = useRef<Pending | null>(null),
    alive = useRef(true);
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    if (!saving && !uncertain) return;
    const beforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", beforeUnload);
    return () => window.removeEventListener("beforeunload", beforeUnload);
  }, [saving, uncertain]);
  function accept(value: LearningState) {
    if (value.version < latest.current.version) return;
    latest.current = value;
    setProgress(value);
  }
  async function send<T extends Result>(job: Pending): Promise<T | null> {
    if (busy.current || !alive.current) return null;
    busy.current = true;
    pending.current = job;
    setSaving(true);
    setError("");
    try {
      const result = await privateRequest<T>(job.path, job.method, job.body);
      if (!alive.current) return null;
      accept("progress" in result ? result.progress : result);
      pending.current = null;
      setUncertain(false);
      job.onSaved?.();
      return result;
    } catch (failure) {
      if (!alive.current) return null;
      if (failure instanceof ApiRequestError && failure.status < 500) {
        pending.current = null;
        setUncertain(false);
        if (failure.status === 409) {
          try {
            const fresh = await privateRequest<LearningSession>(
              "/api/v1/learning-sessions/" + initial.progress.id,
              "GET",
            );
            if (alive.current) accept(fresh.progress);
          } catch {
            /* preserve the draft and known state */
          }
        }
        setError(
          failure.status === 409
            ? "另一处学习进度已更新，请检查当前记录后再确认。"
            : failure.status === 410
              ? "课程已撤回，暂时无法继续学习。"
              : failure.status === 400
                ? "请检查答案后再确认。"
                : failure.message,
        );
      } else {
        // Keep the exact body/key. A server commit may have happened before the connection failed.
        setUncertain(true);
        setError("保存尚未确认。重试会确认这次提交，答案仍保留在当前页面。");
      }
      return null;
    } finally {
      busy.current = false;
      if (alive.current) setSaving(false);
    }
  }
  function write<T extends Result>(
    suffix: string,
    method: "POST" | "PUT",
    fields: object = {},
    onSaved?: () => void,
  ): Promise<T | null> {
    if (pending.current || busy.current) return Promise.resolve(null);
    return send<T>({
      path: "/api/v1/learning-sessions/" + initial.progress.id + suffix,
      method,
      body: {
        ...fields,
        version: latest.current.version,
        idempotencyKey: operationKey(),
      },
      onSaved,
    });
  }
  function retry() {
    if (pending.current) void send(pending.current);
  }
  return {
    progress,
    saving,
    error,
    uncertain,
    blocked: saving || uncertain,
    write,
    retry,
  };
}
