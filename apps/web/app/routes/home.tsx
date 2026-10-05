import { Link } from "react-router";
import { useState, useRef } from "react";
import {
  getCatalog,
  getIdentity,
  getLesson,
  getPrivate,
} from "../lib/api.server";
import type { StudyDashboard } from "@brioche/contracts/StudyDashboard";
import type { LearningSession } from "@brioche/contracts/LearningSession";
import { StudyOverview } from "../components/study-overview";
import { StartLearning } from "../components/start-learning";
import { useLearning } from "../components/learning";
import { Icon } from "../components/icon";
import { Illustration, illustration } from "../components/illustration";
import type { Route } from "./+types/home";
export async function loader({ request }: Route.LoaderArgs) {
  const [initialCatalog, identity] = await Promise.all([
    getCatalog(),
    getIdentity(request),
  ]);
  let catalog = initialCatalog;
  let learning: StudyDashboard | null = null;
  if (identity.user) {
    try {
      learning = await getPrivate<StudyDashboard>(
        request,
        "/api/v1/me/dashboard",
      );
      catalog = learning.catalog;
    } catch (error) {
      if (!(error instanceof Response && error.status === 401)) throw error;
    }
  }
  const first = catalog.levels.flatMap((l) =>
    l.units.flatMap((u) => u.lessons),
  )[0];
  return {
    catalog,
    lesson: learning?.resume
      ? (
          await getPrivate<LearningSession>(
            request,
            "/api/v1/learning-sessions/" + learning.resume.sessionId,
          )
        ).lesson
      : learning?.recommendedLesson
        ? await getLesson(
            learning.recommendedLesson.id,
            learning.recommendedLesson.revision,
          )
        : first
          ? await getLesson(first.id, first.revision)
          : null,
    learning,
  };
}
export default function Home({
  loaderData: { catalog, lesson, learning },
}: Route.ComponentProps) {
  const [open, setOpen] = useState(false),
    card = useRef<HTMLButtonElement>(null);
  const context = useLearning();
  const resume = learning?.resume;
  const expression = lesson?.knowledge.vocabulary.find((entry) =>
    lesson.reviewItemIds.includes(entry.id),
  );
  const scene = lesson?.blocks.find((block) => block.type === "scene");
  const artwork = lesson ? illustration(lesson, scene?.illustrationId) : null;
  return (
    <section className="home page-arrive">
      <div className="intro">
        <div>
          <h2>Bonjour，今天从一件小事开始。</h2>
          <p>一点法语，一点生活。</p>
        </div>
      </div>
      {learning && <StudyOverview dashboard={learning} />}
      {resume && (
        <Link className="resume-learning" to={"/learning/" + resume.sessionId}>
          <span>
            <small>接着上次</small>
            <strong>{resume.title.zh}</strong>
          </span>
          <Icon name="arrow" />
        </Link>
      )}
      {!lesson ? (
        <div className="empty-state">
          <h1>课程正在准备中</h1>
          <p>发布课程后，就可以在这里开始学习。</p>
        </div>
      ) : (
        <>
          <div className={"hero" + (artwork ? "" : " hero-without-art")}>
            <div className="hero-copy">
              <span className="pill">
                {catalog.levels.find((level) => level.id === lesson.levelId)
                  ?.label ?? lesson.levelId.toUpperCase()}{" "}
                ·{" "}
                {catalog.levels
                  .flatMap((level) => level.units)
                  .find((unit) => unit.id === lesson.unitId)?.titleZh ??
                  "日常法语"}
              </span>
              <h1>
                把法语，
                <br />
                放进每一天。
              </h1>
              <p>{lesson.summaryZh}</p>
              {context.profile ? (
                <StartLearning key={lesson.id} lessonId={lesson.id}>
                  {resume
                    ? "继续这堂课"
                    : learning?.allAvailableCompleted
                      ? "再读一课"
                      : "开始今天的课程"}
                </StartLearning>
              ) : (
                <Link className="primary" to={"/lessons/" + lesson.id}>
                  {resume
                    ? "继续这堂课"
                    : learning?.allAvailableCompleted
                      ? "再读一课"
                      : "开始今天的课程"}
                </Link>
              )}
              <div className="meta">
                约 {lesson.estimatedMinutes} 分钟 · 场景、表达与练习
              </div>
            </div>
            {artwork && (
              <div className="hero-art">
                <Illustration asset={artwork} />
              </div>
            )}
          </div>
          <div className="below">
            <section>
              <div className="section-head">
                <h2>课程</h2>
                <Link className="text-button" to="/courses">
                  浏览与搜索 <Icon name="arrow" />
                </Link>
              </div>
              {catalog.levels.map((level) => (
                <div key={level.id} className="course-level">
                  <p className="eyebrow">{level.label}</p>
                  {level.units.map((unit) => (
                    <div key={unit.id} className="course-unit">
                      <div className="section-head">
                        <h2>{unit.titleZh}</h2>
                        {catalog.developmentFixture && <small>示例课程</small>}
                      </div>
                      {unit.lessons.map((entry, i) => (
                        <Link
                          key={entry.id}
                          className="lesson-row current"
                          to={
                            learning?.courseStates.find(
                              (item) =>
                                item.lessonId === entry.id && !item.completedAt,
                            )
                              ? "/learning/" +
                                learning.courseStates.find(
                                  (item) =>
                                    item.lessonId === entry.id &&
                                    !item.completedAt,
                                )!.sessionId
                              : "/lessons/" + entry.id
                          }
                        >
                          <span className="lesson-number">
                            {String(i + 1).padStart(2, "0")}
                          </span>
                          <span className="lesson-label">
                            <b>{entry.title.zh}</b>
                            <small lang="fr">{entry.title.fr}</small>
                          </span>
                          <span className="row-state">
                            {learning?.courseStates.find(
                              (item) => item.lessonId === entry.id,
                            )?.firstCompletedAt
                              ? "已学过"
                              : learning?.courseStates.some(
                                    (item) => item.lessonId === entry.id,
                                  )
                                ? "继续"
                                : "开始"}
                          </span>
                        </Link>
                      ))}
                    </div>
                  ))}
                </div>
              ))}
            </section>
            <aside className="home-review">
              <button
                ref={card}
                className="review"
                aria-expanded={open}
                onClick={() => {
                  const start = card.current?.getBoundingClientRect().height;
                  setOpen(!open);
                  requestAnimationFrame(() => {
                    if (
                      card.current &&
                      start &&
                      !matchMedia("(prefers-reduced-motion:reduce)").matches
                    )
                      card.current.animate(
                        [
                          { height: start + "px" },
                          {
                            height:
                              card.current.getBoundingClientRect().height +
                              "px",
                          },
                        ],
                        { duration: 320, easing: "cubic-bezier(.22,.8,.25,1)" },
                      );
                  });
                }}
              >
                <span className="eyebrow">记住一句日常表达</span>
                <span className="fr" lang="fr">
                  {expression?.lemma}
                </span>
                <span className="review-description">{lesson.summaryZh}</span>
                {open && (
                  <span className="review-answer">
                    <strong>{expression?.meaningZh}</strong>
                    <span className="review-line">{expression?.noteZh}</span>
                  </span>
                )}
              </button>
              <Link
                className="home-review-link text-button"
                to={context.profile ? "/reviews" : "/review/" + lesson.id}
              >
                <span>复习这组表达</span>
                <Icon name="arrow" />
              </Link>
            </aside>
          </div>
          <div className="home-footer">
            {lesson.blocks.find((block) => block.type === "habit")?.taskZh}
          </div>
        </>
      )}
    </section>
  );
}
