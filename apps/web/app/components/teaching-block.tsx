import type { Block } from "@brioche/contracts/Block";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import { Link } from "react-router";
import { useLearning } from "./learning";

type TeachingBlock = Exclude<Block, { type: "dialogue" | "article" }>;
export function TeachingBlock({
  block,
  lesson,
}: {
  block: TeachingBlock;
  lesson: PublicLesson;
}) {
  const learning = useLearning();
  switch (block.type) {
    case "scene":
      return (
        <div className="lesson-scene">
          <p className="knowledge-label">{block.placeZh}</p>
          <p>{block.situationZh}</p>
        </div>
      );
    case "explanation":
      return (
        <details className="lesson-note">
          <summary>{block.titleZh}</summary>
          <p>{block.bodyZh}</p>
        </details>
      );
    case "culture":
      return (
        <details className="lesson-note">
          <summary>{block.titleZh}</summary>
          <p>{block.bodyZh}</p>
          <p className="profile-note">{block.scopeZh}</p>
        </details>
      );
    case "vocabulary":
      return (
        <details className="lesson-note">
          <summary>表达与词汇</summary>
          <dl className="vocabulary-list">
            {block.entryIds.map((id) => {
              const word = lesson.knowledge.vocabulary.find(
                (w) => w.id === id,
              )!;
              return (
                <div key={id}>
                  <dt>
                    <button
                      type="button"
                      lang="fr"
                      onClick={() => learning.play([{ id, text: word.lemma }])}
                    >
                      {word.lemma}
                    </button>
                  </dt>
                  <dd>
                    <strong>{word.meaningZh}</strong>
                    <p>{word.noteZh}</p>
                  </dd>
                </div>
              );
            })}
          </dl>
        </details>
      );
    case "grammar":
      return (
        <>
          {block.entryIds.map((id) => {
            const grammar = lesson.knowledge.grammar.find((g) => g.id === id)!;
            return (
              <details key={id} className="lesson-note">
                <summary>{grammar.titleZh}</summary>
                <p>{grammar.bodyZh}</p>
                {grammar.examples.map((example, i) => (
                  <div className="grammar-example" key={i}>
                    <button
                      type="button"
                      lang="fr"
                      onClick={() =>
                        learning.play([{ id: id + i, text: example.fr }])
                      }
                    >
                      {example.fr}
                    </button>
                    <p>{example.zh}</p>
                  </div>
                ))}
              </details>
            );
          })}
        </>
      );
    case "exercise":
      return (
        <Link className="text-button" to={"/practice/" + lesson.id}>
          {block.promptZh}
        </Link>
      );
    case "habit":
      return (
        <details className="lesson-note">
          <summary>带进日常</summary>
          <p>{block.taskZh}</p>
          <p className="profile-note">{block.alternativeZh}</p>
        </details>
      );
    case "summary":
      return (
        <div className="lesson-note">
          <h2>今天能做到</h2>
          <ul>
            {block.takeawaysZh.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>
      );
    default: {
      const unsupported: never = block;
      throw Error("Unsupported teaching block: " + JSON.stringify(unsupported));
    }
  }
}
