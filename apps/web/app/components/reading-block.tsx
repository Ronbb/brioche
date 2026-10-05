import { useRef, useState } from "react";
import type { Block } from "@brioche/contracts/Block";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import type { Vocabulary } from "@brioche/contracts/Vocabulary";
import type { Grammar } from "@brioche/contracts/Grammar";
import { Sentence, avatar } from "../routes/lesson";
import { Player, useLearning } from "./learning";
import { Icon } from "./icon";
export function ReadingBlock({
  block,
  lesson,
}: {
  block: Extract<Block, { type: "dialogue" | "article" }>;
  lesson: PublicLesson;
}) {
  const learning = useLearning(),
    dialog = useRef<HTMLDialogElement>(null);
  const [revealed, setRevealed] = useState(new Set<string>()),
    [term, setTerm] = useState<Vocabulary | null>(null),
    [grammar, setGrammar] = useState<Grammar | null>(null);
  const entries = block.type === "dialogue" ? block.turns : block.paragraphs;
  function showTerm(value: Vocabulary) {
    setTerm(value);
    setGrammar(null);
    dialog.current?.showModal();
  }
  function showGrammar(value: Grammar) {
    setGrammar(value);
    setTerm(null);
    if (!dialog.current?.open) dialog.current?.showModal();
  }
  const units = entries.map((entry) => ({
    id: block.id + ":" + entry.id,
    text: entry.segments.map((segment) => segment.text).join(""),
  }));
  return (
    <div className="reading session-reading">
      <h3 className="reading-block-title">{block.titleZh}</h3>
      <Player units={units} />
      {block.type === "dialogue" && (
        <ul className="reading-characters">
          {block.speakers.map((speaker) => (
            <li key={speaker.id}>
              <img src={avatar(speaker.avatarId)} alt="" />
              <div>
                <span lang="fr">{speaker.displayName}</span>
                <small>{speaker.labelZh}</small>
              </div>
            </li>
          ))}
        </ul>
      )}
      {entries.map((entry) => {
        const speaker =
          block.type === "dialogue" && "speakerId" in entry
            ? block.speakers.find((speaker) => speaker.id === entry.speakerId)
            : null;
        const id = block.id + ":" + entry.id;
        return (
          <div
            key={entry.id}
            className={
              (speaker ? "dialogue-turn" : "article-paragraph") +
              (learning.player.id === id ? " is-speaking" : "")
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
                      id,
                      text: entry.segments
                        .map((segment) => segment.text)
                        .join(""),
                      locale: lesson.cast.find(
                        (character) =>
                          character.characterId === speaker.characterId,
                      )?.speechLocale,
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
                prefix={block.id + ":"}
                onTerm={showTerm}
                onGrammar={showGrammar}
              />
              {(learning.translation || revealed.has(entry.id)) && (
                <p className="translation">{entry.translationZh}</p>
              )}
            </div>
          </div>
        );
      })}
      <dialog
        className="knowledge-dialog"
        ref={dialog}
        aria-labelledby={block.id + "-knowledge-title"}
        onClick={(event) => {
          if (event.target !== dialog.current || !dialog.current) return;
          const rect = dialog.current.getBoundingClientRect();
          if (
            event.clientX < rect.left ||
            event.clientX > rect.right ||
            event.clientY < rect.top ||
            event.clientY > rect.bottom
          )
            dialog.current.close();
        }}
      >
        <div className="rate-heading">
          <span className="knowledge-label">
            {grammar ? "语法" : "表达与词汇"}
          </span>
          <button
            className="icon-button"
            aria-label="关闭解释"
            onClick={() => dialog.current?.close()}
          >
            <Icon name="close" />
          </button>
        </div>
        <h2 id={block.id + "-knowledge-title"} lang={grammar ? "zh-CN" : "fr"}>
          {grammar?.titleZh ?? term?.lemma}
        </h2>
        {term && (
          <>
            <p className="meaning">{term.meaningZh}</p>
            <p className="explain">{term.noteZh}</p>
            <Bookmark
              key={term.id}
              knowledgeId={term.id}
              lessonId={lesson.id}
              revision={lesson.revision}
            />
            <Enroll
              key={"enroll-" + term.id}
              knowledgeId={term.id}
              lessonId={lesson.id}
              revision={lesson.revision}
            />
            <div className="example" lang="fr">
              {entries
                .find((entry) =>
                  entry.segments.some(
                    (segment) => segment.vocabularyId === term.id,
                  ),
                )
                ?.segments.map((segment) => segment.text)
                .join("")}
            </div>
          </>
        )}
        {grammar && (
          <>
            <p className="explain">{grammar.bodyZh}</p>
            {grammar.examples.map((example, index) => (
              <div className="grammar-example" key={index}>
                <button
                  lang="fr"
                  onClick={() =>
                    learning.play([
                      {
                        id: block.id + ":" + grammar.id + index,
                        text: example.fr,
                      },
                    ])
                  }
                >
                  {example.fr}
                </button>
                <p>{example.zh}</p>
              </div>
            ))}
          </>
        )}
      </dialog>
    </div>
  );
}
import { Bookmark } from "./bookmark";
import { Enroll } from "./enroll";
