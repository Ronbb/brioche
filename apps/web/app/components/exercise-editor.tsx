import { useState } from "react";
import type { Block } from "@brioche/contracts/Block";
import type { AttemptRecord } from "@brioche/contracts/AttemptRecord";
import type { ExerciseAnswer } from "@brioche/contracts/ExerciseAnswer";
import { Icon } from "./icon";
export function ExerciseEditor({
  block,
  latest,
  hinted,
  blocked,
  completed,
  submit,
  hint,
}: {
  block: Extract<Block, { type: "exercise" }>;
  latest?: AttemptRecord;
  hinted: boolean;
  blocked: boolean;
  completed: boolean;
  submit: (answer: ExerciseAnswer, onSaved: () => void) => Promise<unknown>;
  hint: () => void;
}) {
  const [choice, setChoice] = useState(
      latest?.answer.kind === "choice" ? latest.answer.optionId : "",
    ),
    [text, setText] = useState(
      latest?.answer.kind === "text" ? latest.answer.text : "",
    ),
    [order, setOrder] = useState<string[]>(
      latest?.answer.kind === "order" ? latest.answer.tokenIds : [],
    ),
    [editing, setEditing] = useState(!latest);
  const shownChoice =
    !editing && latest?.answer.kind === "choice"
      ? latest.answer.optionId
      : choice;
  const shownText =
    !editing && latest?.answer.kind === "text" ? latest.answer.text : text;
  const shownOrder =
    !editing && latest?.answer.kind === "order"
      ? latest.answer.tokenIds
      : order;
  const result = !editing ? latest?.result : null;
  const ready =
    block.exerciseType === "single-choice"
      ? !!choice
      : block.exerciseType === "fill-blank"
        ? !!text.trim()
        : order.length === block.tokens.length;
  return (
    <form
      className="exercise-sheet"
      onSubmit={(event) => {
        event.preventDefault();
        if (!ready || blocked || completed) return;
        void submit(
          block.exerciseType === "single-choice"
            ? { kind: "choice", optionId: choice }
            : block.exerciseType === "fill-blank"
              ? { kind: "text", text }
              : { kind: "order", tokenIds: order },
          () => setEditing(false),
        );
      }}
    >
      <h2>{block.promptZh}</h2>
      <fieldset disabled={blocked || !!result || completed}>
        <legend className="sr-only">你的答案</legend>
        {block.exerciseType === "single-choice" && (
          <div className="practice-options">
            {block.options.map((option) => (
              <label
                key={option.id}
                className={
                  "practice-option" +
                  (shownChoice === option.id ? " is-selected" : "")
                }
              >
                <input
                  type="radio"
                  name={block.id + "-answer"}
                  checked={shownChoice === option.id}
                  value={option.id}
                  onChange={() => setChoice(option.id)}
                />
                <span>{option.text}</span>
                <Icon name="check" />
              </label>
            ))}
          </div>
        )}
        {block.exerciseType === "fill-blank" && (
          <>
            <p className="practice-sentence" lang="fr">
              {block.templateFr}
            </p>
            <label className="answer-label" htmlFor={block.id + "-answer"}>
              你的答案
            </label>
            <input
              className="practice-input"
              id={block.id + "-answer"}
              lang="fr"
              autoComplete="off"
              autoCapitalize="none"
              spellCheck={false}
              maxLength={1024}
              value={shownText}
              onChange={(event) => setText(event.target.value)}
            />
          </>
        )}
        {block.exerciseType === "order" && (
          <>
            <div className="order-answer" aria-label="当前句子" lang="fr">
              {shownOrder.length ? (
                shownOrder.map((id) => (
                  <button
                    key={id}
                    type="button"
                    aria-label={
                      "移回词库：" +
                      block.tokens.find((token) => token.id === id)?.text
                    }
                    onClick={() =>
                      setOrder((old) => old.filter((value) => value !== id))
                    }
                  >
                    {block.tokens.find((token) => token.id === id)?.text}
                  </button>
                ))
              ) : (
                <span className="order-empty">组成一句话</span>
              )}
            </div>
            <div className="order-bank" lang="fr">
              {block.tokens.map((token) => (
                <button
                  key={token.id}
                  type="button"
                  disabled={shownOrder.includes(token.id)}
                  onClick={() => setOrder((old) => [...old, token.id])}
                >
                  {token.text}
                </button>
              ))}
            </div>
          </>
        )}
      </fieldset>
      {block.exerciseType === "fill-blank" &&
        (hinted ? (
          <p className="profile-note">{block.hintZh}</p>
        ) : (
          <button
            className="text-button practice-hint"
            type="button"
            disabled={blocked || completed}
            onClick={hint}
          >
            提示
          </button>
        ))}
      {result && (
        <div
          className={
            "practice-feedback" + (result.correct ? " is-correct" : "")
          }
          role="status"
        >
          <strong>{result.correct ? "答对了" : "再看看这个表达"}</strong>
          <p>{result.feedbackZh}</p>
        </div>
      )}
      {!completed &&
        (result ? (
          <button
            type="button"
            className="text-button"
            disabled={blocked}
            onClick={() => setEditing(true)}
          >
            再试一次
          </button>
        ) : (
          <button className="primary" disabled={!ready || blocked}>
            确认答案
            <Icon name="check" />
          </button>
        ))}
    </form>
  );
}
