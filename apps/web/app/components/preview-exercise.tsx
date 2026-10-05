import { useEffect, useRef, useState } from "react";
import type { Block } from "@brioche/contracts/Block";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import type { ExerciseAnswer } from "@brioche/contracts/ExerciseAnswer";
import type { GradeResult } from "@brioche/contracts/GradeResult";
import {
  ApiRequestError,
  privateRequest as requestApi,
} from "../lib/api.client";
import { ExerciseEditor } from "./exercise-editor";
import { useLearning } from "./learning";

export function PreviewExercise({
  block,
  lesson,
}: {
  block: Extract<Block, { type: "exercise" }>;
  lesson: PublicLesson;
}) {
  const audio = useLearning();
  const [busy, setBusy] = useState(false),
    [hinted, setHinted] = useState(false);
  const [latest, setLatest] = useState<{
    id: string;
    answer: ExerciseAnswer;
    result: GradeResult;
  }>();
  const pending = useRef(false),
    sequence = useRef(0),
    active = useRef(true);
  const owner = audio.profile?.role === "operator" ? audio.profile.id : null;
  const ownerRef = useRef(owner);
  ownerRef.current = owner;
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  async function submit(answer: ExerciseAnswer, onConfirmed: () => void) {
    if (pending.current || !owner) return;
    const expectedOwner = owner;
    pending.current = true;
    setBusy(true);
    try {
      const result = await requestApi<GradeResult>(
        `/api/v1/operator/lessons/${encodeURIComponent(lesson.id)}/revisions/${lesson.revision}/grade`,
        "POST",
        { revision: lesson.revision, exerciseId: block.id, answer },
      );
      if (active.current && ownerRef.current === expectedOwner) {
        setLatest({ id: `preview-${++sequence.current}`, answer, result });
        onConfirmed();
      }
    } catch (error) {
      if (active.current && ownerRef.current === expectedOwner)
        audio.toast(
          error instanceof ApiRequestError
            ? error.message
            : "预览判分暂时无法连接，请稍后重试。",
        );
    } finally {
      pending.current = false;
      if (active.current) setBusy(false);
    }
  }
  if (!owner) return null;
  return (
    <ExerciseEditor
      block={block}
      latest={latest}
      hinted={hinted}
      blocked={busy}
      completed={false}
      submit={submit}
      hint={() => setHinted(true)}
    />
  );
}
