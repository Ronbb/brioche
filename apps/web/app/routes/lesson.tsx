import { useState } from "react";
import { Link } from "react-router";
import type { Segment } from "@brioche/contracts/Segment";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import type { Vocabulary } from "@brioche/contracts/Vocabulary";
import { getLesson } from "../lib/api.server";
import { Player, useLearning } from "../components/learning";
import { Icon } from "../components/icon";
import type { Route } from "./+types/lesson";
export async function loader({ params }: Route.LoaderArgs) {
  return { lesson: await getLesson(params.lessonId) };
}
const avatar = (id: string) =>
  "/assets/avatars/" +
  ({
    "avatar-camille-v1": "camille",
    "avatar-luc-v1": "luc",
    "avatar-lea-v1": "lea",
  }[id] ?? "learner") +
  ".svg";
export function Sentence({
  segments,
  lesson,
  onTerm,
}: {
  segments: Segment[];
  lesson: PublicLesson;
  onTerm: (v: Vocabulary) => void;
}) {
  const learning = useLearning();
  return (
    <span className="sentence" lang="fr">
      {segments.flatMap((segment) =>
        Array.from(
          new Intl.Segmenter("fr", { granularity: "word" }).segment(
            segment.text,
          ),
        ).map((token, i) =>
          token.isWordLike ? (
            <button
              key={segment.id + i}
              className={"word" + (segment.vocabularyId ? " known" : "")}
              onClick={() => {
                learning.play([
                  { id: "word-" + segment.id + "-" + i, text: token.segment },
                ]);
                const term = lesson.knowledge.vocabulary.find(
                  (v) => v.id === segment.vocabularyId,
                );
                if (term) onTerm(term);
              }}
            >
              {token.segment}
            </button>
          ) : (
            <span key={segment.id + i}>{token.segment}</span>
          ),
        ),
      )}
    </span>
  );
}
export default function Lesson({
  loaderData: { lesson },
}: Route.ComponentProps) {
  const learning = useLearning(),
    [mode, setMode] = useState<"dialogue" | "article">("dialogue"),
    [revealed, setRevealed] = useState<Set<string>>(new Set()),
    [term, setTerm] = useState<Vocabulary | null>(null);
  const dialogue = lesson.blocks.find((b) => b.type === "dialogue"),
    article = lesson.blocks.find((b) => b.type === "article");
  const entries =
    mode === "dialogue" && dialogue?.type === "dialogue"
      ? dialogue.turns
      : article?.type === "article"
        ? article.paragraphs
        : [];
  const units = entries.map((e) => ({
    id: e.id,
    text: e.segments.map((s) => s.text).join(""),
  }));
  function changeMode(next: "dialogue" | "article") {
    learning.stop();
    setMode(next);
    setTerm(null);
  }
  return (
    <section className="page-arrive">
      <div className="lesson-header">
        <div className="crumb">
          {lesson.levelId.toUpperCase()} / {lesson.title.zh}
        </div>
        <h1 lang="fr">{lesson.title.fr}</h1>
        <Player units={units} />
      </div>
      <div className="reading-layout">
        <div className="reading">
          <div className="reading-tabs" role="tablist" aria-label="正文">
            {(["dialogue", "article"] as const).map((value) => (
              <button
                key={value}
                role="tab"
                aria-selected={mode === value}
                onClick={() => changeMode(value)}
              >
                {value === "dialogue" ? "对话" : "短文"}
              </button>
            ))}
          </div>
          {mode === "dialogue" && dialogue?.type === "dialogue" && (
            <ul className="reading-characters">
              {dialogue.speakers.map((s) => (
                <li key={s.id}>
                  <img src={avatar(s.avatarId)} alt="" />
                  <div>
                    <span lang="fr">{s.displayName}</span>
                    <small>{s.labelZh}</small>
                  </div>
                </li>
              ))}
            </ul>
          )}
          {entries.map((entry) => {
            const speaker =
              "speakerId" in entry && dialogue?.type === "dialogue"
                ? dialogue.speakers.find((s) => s.id === entry.speakerId)
                : null;
            return (
              <div
                key={entry.id}
                className={
                  (speaker ? "dialogue-turn" : "article-paragraph") +
                  (learning.player.id === entry.id ? " is-speaking" : "")
                }
              >
                {speaker && (
                  <button
                    className="speaker"
                    aria-label={speaker.displayName + "：译文与朗读"}
                    onClick={() => {
                      setRevealed((old) => new Set([...old, entry.id]));
                      learning.play([
                        {
                          id: entry.id,
                          text: entry.segments.map((s) => s.text).join(""),
                        },
                      ]);
                    }}
                  >
                    <img src={avatar(speaker.avatarId)} alt="" />
                  </button>
                )}
                <div>
                  <Sentence
                    segments={entry.segments}
                    lesson={lesson}
                    onTerm={setTerm}
                  />
                  {(learning.translation || revealed.has(entry.id)) && (
                    <p className="translation">{entry.translationZh}</p>
                  )}
                </div>
              </div>
            );
          })}
          <div className="reading-footer">
            <Link className="primary" to={"/review/" + lesson.id}>
              复习表达
              <Icon name="arrow" />
            </Link>
          </div>
        </div>
        <aside
          className={"knowledge" + (term ? " is-open" : "")}
          aria-label="表达解释"
        >
          <button
            className="icon-button note-close"
            aria-label="关闭解释"
            onClick={() => setTerm(null)}
          >
            <Icon name="close" />
          </button>
          {term ? (
            <>
              <span className="knowledge-label">表达与词汇</span>
              <h2 lang="fr">{term.lemma}</h2>
              <p className="meaning">{term.meaningZh}</p>
              <p className="explain">{term.noteZh}</p>
              <div className="example" lang="fr">
                {entries
                  .find((e) =>
                    e.segments.some((s) => s.vocabularyId === term.id),
                  )
                  ?.segments.map((s) => s.text)
                  .join("")}
              </div>
            </>
          ) : (
            <>
              <span className="knowledge-label">本课表达</span>
              <h2 lang="fr">{lesson.knowledge.vocabulary[1]?.lemma}</h2>
              <p className="meaning">
                {lesson.knowledge.vocabulary[1]?.meaningZh}
              </p>
            </>
          )}
        </aside>
      </div>
      {term && (
        <button
          className="knowledge-backdrop"
          aria-label="关闭解释"
          onClick={() => setTerm(null)}
        />
      )}
    </section>
  );
}
