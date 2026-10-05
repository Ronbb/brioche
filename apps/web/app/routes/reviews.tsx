import { useEffect, useRef, useState } from "react";
import { Link, redirect } from "react-router";
import type { ReviewQueue } from "@brioche/contracts/ReviewQueue";
import type { ReviewCard } from "@brioche/contracts/ReviewCard";
import type { ReviewRating } from "@brioche/contracts/ReviewRating";
import type { ReviewAttemptRequest } from "@brioche/contracts/ReviewAttemptRequest";
import type { ReviewAttemptResult } from "@brioche/contracts/ReviewAttemptResult";
import { getPrivate } from "../lib/api.server";
import { privateRequest, ApiRequestError } from "../lib/api.client";
import { operationKey } from "../lib/operation-key";
import { useLearning } from "../components/learning";
import { Icon } from "../components/icon";
import type { Route } from "./+types/reviews";
export async function loader({ request }: Route.LoaderArgs) {
  try {
    return await getPrivate<ReviewQueue>(request, "/api/v1/me/reviews");
  } catch (error) {
    if (error instanceof Response && error.status === 401)
      throw redirect("/login?next=/reviews");
    throw error;
  }
}
const ratings: { value: ReviewRating; label: string }[] = [
  { value: "again", label: "还不熟" },
  { value: "remembered", label: "有印象" },
  { value: "familiar", label: "记住了" },
];
export default function Reviews({ loaderData }: Route.ComponentProps) {
  const [queue, setQueue] = useState(loaderData),
    [index, setIndex] = useState(0),
    [revealed, setRevealed] = useState(false),
    [saving, setSaving] = useState(false),
    [uncertain, setUncertain] = useState(false),
    [error, setError] = useState(""),
    [results, setResults] = useState<ReviewAttemptResult[]>([]);
  const pending = useRef<{
      card: ReviewCard;
      body: ReviewAttemptRequest;
    } | null>(null),
    busy = useRef(false),
    alive = useRef(true),
    cardButton = useRef<HTMLButtonElement>(null),
    heading = useRef<HTMLHeadingElement>(null),
    animation = useRef<Animation | null>(null);
  const audio = useLearning(),
    term = queue.items[index];
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
      animation.current?.cancel();
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
  async function submit(job: NonNullable<typeof pending.current>) {
    if (busy.current) return;
    busy.current = true;
    pending.current = job;
    setSaving(true);
    setError("");
    try {
      const saved = await privateRequest<ReviewAttemptResult>(
        "/api/v1/me/reviews/" + job.card.id + "/attempts",
        "POST",
        job.body,
      );
      if (!alive.current) return;
      pending.current = null;
      setUncertain(false);
      setResults((old) => [...old, saved]);
      setIndex((old) => old + 1);
      setRevealed(false);
      audio.stop();
      requestAnimationFrame(() => {
        heading.current?.focus();
        heading.current?.scrollIntoView({
          block: "start",
          behavior: "instant",
        });
      });
    } catch (failure) {
      if (!alive.current) return;
      if (failure instanceof ApiRequestError && failure.status < 500) {
        pending.current = null;
        setUncertain(false);
        if (
          failure.status === 409 ||
          failure.status === 410 ||
          failure.status === 404
        ) {
          try {
            const fresh = await privateRequest<ReviewQueue>(
              "/api/v1/me/reviews",
              "GET",
            );
            if (alive.current) {
              setQueue(fresh);
              setIndex(0);
              setRevealed(false);
            }
          } catch {
            /* keep the current card until the next explicit retry */
          }
          setError("复习记录已变化，请确认最新队列后继续。");
        } else setError(failure.message);
      } else {
        setUncertain(true);
        setError("这次保存尚未确认，请重试确认原提交。");
      }
    } finally {
      busy.current = false;
      if (alive.current) setSaving(false);
    }
  }
  async function nextBatch() {
    if (busy.current) return;
    busy.current = true;
    setSaving(true);
    setError("");
    try {
      const fresh = await privateRequest<ReviewQueue>(
        "/api/v1/me/reviews",
        "GET",
      );
      if (alive.current) {
        setQueue(fresh);
        setIndex(0);
        setRevealed(false);
      }
    } catch (failure) {
      if (alive.current)
        setError(
          failure instanceof Error ? failure.message : "暂时无法读取复习队列。",
        );
    } finally {
      busy.current = false;
      if (alive.current) setSaving(false);
    }
  }
  function reveal() {
    const height = cardButton.current?.getBoundingClientRect().height;
    animation.current?.cancel();
    setRevealed(!revealed);
    if (!revealed && term)
      audio.play([
        {
          id: "review-" + term.id,
          text:
            (term.vocabulary.gender === "feminine"
              ? "une "
              : term.vocabulary.gender === "masculine"
                ? "un "
                : "") + term.vocabulary.lemma,
        },
      ]);
    else audio.stop();
    requestAnimationFrame(() => {
      if (
        cardButton.current &&
        height &&
        !matchMedia("(prefers-reduced-motion:reduce)").matches
      )
        animation.current = cardButton.current.animate(
          [
            { height: height + "px" },
            {
              height: cardButton.current.getBoundingClientRect().height + "px",
            },
          ],
          { duration: 420, easing: "cubic-bezier(.22,.8,.25,1)" },
        );
    });
  }
  return (
    <section className="review-page page-arrive">
      <div className="review-session-header">
        <div>
          <h1 ref={heading} tabIndex={-1}>
            复习
          </h1>
          <p>今天的表达</p>
        </div>
        {queue.items.length > 0 && (
          <span className="small">
            {Math.min(index + 1, queue.items.length)} / {queue.items.length}
          </span>
        )}
      </div>
      <div
        className="review-progress"
        role="progressbar"
        aria-label="本轮复习"
        aria-valuemin={0}
        aria-valuemax={queue.items.length || 1}
        aria-valuenow={index}
      >
        <span
          style={{
            width: queue.items.length
              ? (index / queue.items.length) * 100 + "%"
              : "100%",
          }}
        />
      </div>
      {term ? (
        <>
          <div className="review-context">
            <Link to={"/lessons/" + term.sourceLessonId}>
              <Icon name="book" />
              回看来源课程
            </Link>
          </div>
          <button
            ref={cardButton}
            className="review-flashcard"
            aria-expanded={revealed}
            disabled={saving || uncertain}
            onClick={reveal}
          >
            <span className="review-kind">
              {term.vocabulary.partOfSpeech === "phrase"
                ? "常用表达"
                : "日常词汇"}
            </span>
            <span className="review-expression" lang="fr">
              {term.vocabulary.gender === "feminine"
                ? "une "
                : term.vocabulary.gender === "masculine"
                  ? "un "
                  : ""}
              {term.vocabulary.lemma}
            </span>
            {revealed && (
              <span className="review-solution">
                <span className="review-meaning">
                  {term.vocabulary.meaningZh}
                </span>
                <span className="review-explanation">
                  {term.vocabulary.noteZh}
                </span>
              </span>
            )}
          </button>
          {revealed && !uncertain && (
            <div
              className="review-ratings review-choices-enter"
              role="group"
              aria-label="这次回想的感觉"
            >
              {ratings.map((rating, grade) => (
                <button
                  key={rating.value}
                  data-grade={grade}
                  disabled={saving}
                  onClick={() =>
                    void submit({
                      card: term,
                      body: {
                        cardVersion: term.version,
                        idempotencyKey: operationKey(),
                        rating: rating.value,
                      },
                    })
                  }
                >
                  <span className="rating-dot" aria-hidden="true" />
                  <span>{rating.label}</span>
                  <Icon name="chevron" />
                </button>
              ))}
            </div>
          )}
        </>
      ) : (
        <div className="review-summary">
          <h2>{results.length ? "本轮回顾" : "暂时没有到期表达"}</h2>
          <p>
            {results.length
              ? `已保存 ${results.length} 个表达的复习记录。`
              : queue.nextDueAt
                ? "下一次复习：" +
                  new Date(queue.nextDueAt).toLocaleDateString("zh-CN", {
                    timeZone: queue.timeZone,
                    month: "numeric",
                    day: "numeric",
                  })
                : "学完课程后，表达会加入这里。"}
          </p>
          {results.length > 0 && (
            <ul className="review-result-list">
              {results.map((result) => (
                <li key={result.card.id}>
                  <div>
                    <span className="result-expression" lang="fr">
                      {result.card.vocabulary.lemma}
                    </span>
                    <small>{result.card.vocabulary.meaningZh}</small>
                  </div>
                  <span className="review-result-grade">
                    下次{" "}
                    {new Date(result.card.dueAt).toLocaleDateString("zh-CN", {
                      timeZone: result.timeZone,
                      month: "numeric",
                      day: "numeric",
                    })}
                  </span>
                </li>
              ))}
            </ul>
          )}
          <div className="review-summary-actions">
            {queue.dueCount > queue.items.length && (
              <button
                className="primary summary-main"
                disabled={saving}
                onClick={() => void nextBatch()}
              >
                查看下一组
                <Icon name="arrow" />
              </button>
            )}
            <Link className="text-button" to="/">
              回到今天
            </Link>
          </div>
        </div>
      )}
      {error && (
        <p className="error-message" role="alert">
          {error}
        </p>
      )}
      {uncertain && (
        <button
          className="primary"
          disabled={saving}
          onClick={() => {
            if (pending.current) void submit(pending.current);
          }}
        >
          重试保存
          <Icon name="check" />
        </button>
      )}
      {saving && (
        <p className="profile-note" role="status">
          正在保存
        </p>
      )}
    </section>
  );
}
