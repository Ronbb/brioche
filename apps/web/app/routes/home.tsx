import { Link } from "react-router";
import { useState, useRef } from "react";
import { getCatalog, getLesson } from "../lib/api.server";
import { Icon } from "../components/icon";
import type { Route } from "./+types/home";
export async function loader() {
  const catalog = await getCatalog();
  const first = catalog.levels.flatMap((l) =>
    l.units.flatMap((u) => u.lessons),
  )[0];
  return { catalog, lesson: first ? await getLesson(first.id) : null };
}
export default function Home({
  loaderData: { catalog, lesson },
}: Route.ComponentProps) {
  const [open, setOpen] = useState(false),
    card = useRef<HTMLButtonElement>(null);
  return (
    <section className="home page-arrive">
      <div className="intro">
        <div>
          <h2>Bonjour，今天从一件小事开始。</h2>
          <p>一点法语，一点生活。</p>
        </div>
      </div>
      {!lesson ? (
        <div className="empty-state">
          <h1>课程正在准备中</h1>
          <p>发布课程后，就可以在这里开始学习。</p>
        </div>
      ) : (
        <>
          <div className="hero">
            <div className="hero-copy">
              <span className="pill">
                {lesson.levelId.toUpperCase()} 入门 ·{" "}
                {catalog.levels[0]?.units[0]?.titleZh}
              </span>
              <h1>
                把法语，
                <br />
                放进每一天。
              </h1>
              <p>走进街角的面包店。用一句礼貌的请求，为自己买一份早餐。</p>
              <Link className="primary" to={"/lessons/" + lesson.id}>
                走进面包店
              </Link>
              <div className="meta">
                约 {lesson.estimatedMinutes} 分钟 · 对话、表达与生活练习
              </div>
            </div>
            <div className="hero-art">
              <img
                src="/assets/bakery.svg"
                width="640"
                height="470"
                alt="社区面包店的清晨"
              />
            </div>
          </div>
          <div className="below">
            <section>
              <div className="section-head">
                <h2>{catalog.levels[0]?.units[0]?.titleZh}</h2>
                {catalog.developmentFixture && <small>示例课程</small>}
              </div>
              {catalog.levels
                .flatMap((l) => l.units.flatMap((u) => u.lessons))
                .map((entry, i) => (
                  <Link
                    key={entry.id}
                    className="lesson-row current"
                    to={"/lessons/" + entry.id}
                  >
                    <span className="lesson-number">
                      {String(i + 1).padStart(2, "0")}
                    </span>
                    <span className="lesson-label">
                      <b>{entry.title.zh}</b>
                      <small lang="fr">{entry.title.fr}</small>
                    </span>
                    <span className="row-state">开始</span>
                  </Link>
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
                  Je voudrais…
                </span>
                <span className="review-description">
                  买早餐、点饮品时，都可以试着用它开口。
                </span>
                {open && (
                  <span className="review-answer">
                    <strong>我想要……</strong>
                    <span className="review-line">礼貌地提出请求。</span>
                  </span>
                )}
              </button>
              <Link
                className="home-review-link text-button"
                to={"/review/" + lesson.id}
              >
                <span>复习这组表达</span>
                <Icon name="arrow" />
              </Link>
            </aside>
          </div>
          <div className="home-footer">
            今天准备早餐时，试着用法语说出你的选择。
          </div>
        </>
      )}
    </section>
  );
}
