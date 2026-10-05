import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router";
import type { LearningSession } from "@brioche/contracts/LearningSession";
import { ApiRequestError, privateRequest } from "../lib/api.client";
import { operationKey } from "../lib/operation-key";
import { Icon } from "./icon";
export function StartLearning({
  lessonId,
  children = "开始学习",
}: {
  lessonId: string;
  children?: React.ReactNode;
}) {
  const navigate = useNavigate(),
    busy = useRef(false),
    alive = useRef(true),
    key = useRef<string | null>(null);
  const [pending, setPending] = useState(false),
    [error, setError] = useState("");
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  async function start() {
    if (busy.current) return;
    busy.current = true;
    setPending(true);
    setError("");
    key.current ??= operationKey();
    try {
      const session = await privateRequest<LearningSession>(
        "/api/v1/learning-sessions",
        "POST",
        { lessonId, schemaVersion: "1.0", idempotencyKey: key.current },
      );
      if (alive.current) void navigate("/learning/" + session.progress.id);
    } catch (failure) {
      if (alive.current)
        setError(
          failure instanceof ApiRequestError && failure.status === 409
            ? "课程暂时无法开始，请重新打开课程。"
            : failure instanceof ApiRequestError
              ? failure.message
              : "学习尚未打开，请重试。",
        );
    } finally {
      busy.current = false;
      if (alive.current) setPending(false);
    }
  }
  return (
    <div className="start-learning">
      <button
        className="primary"
        disabled={pending}
        onClick={() => void start()}
      >
        {pending ? "正在打开" : children}
        <Icon name="arrow" />
      </button>
      {error && (
        <p className="error-message" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
