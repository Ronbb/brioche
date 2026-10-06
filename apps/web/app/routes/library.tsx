import { useEffect, useRef, useState } from "react";
import { Link, redirect } from "react-router";
import type { SavedItem } from "@brioche/contracts/SavedItem";
import type { SavedPage } from "@brioche/contracts/SavedPage";
import type { ReviewCard } from "@brioche/contracts/ReviewCard";
import type { ReviewCardsPage } from "@brioche/contracts/ReviewCardsPage";
import { getPrivate } from "../lib/api.server";
import { privateRequest } from "../lib/api.client";
import { useOwnedWrite } from "../components/owned-write";
import { useLearning } from "../components/learning";
import { Bookmark } from "../components/bookmark";
import { Enroll } from "../components/enroll";
import { Icon } from "../components/icon";
import type { Route } from "./+types/library";
export async function loader({ request }: Route.LoaderArgs) {
  const url = new URL(request.url),
    view = url.searchParams.get("view") === "reviews" ? "reviews" : "saved",
    cursor = url.searchParams.get("cursor"),
    query = cursor ? "?cursor=" + encodeURIComponent(cursor) : "";
  try {
    return view === "saved"
      ? {
          view: "saved" as const,
          page: await getPrivate<SavedPage>(
            request,
            "/api/v1/me/saved-items" + query,
          ),
          cursor,
        }
      : {
          view: "reviews" as const,
          page: await getPrivate<ReviewCardsPage>(
            request,
            "/api/v1/me/review-cards" + query,
          ),
          cursor,
        };
  } catch (error) {
    if (error instanceof Response && error.status === 401)
      throw redirect("/login?next=/library");
    throw error;
  }
}
export default function Library({ loaderData }: Route.ComponentProps) {
  return (
    <section className="settings-page page-arrive">
      <div className="settings-title-row">
        <h1>我的表达</h1>
        <Link className="text-button" to="/profile">
          我的
        </Link>
      </div>
      <nav className="reader-mode" aria-label="表达分类">
        <Link
          className={loaderData.view === "saved" ? "active" : ""}
          aria-current={loaderData.view === "saved" ? "page" : undefined}
          to="/library"
        >
          收藏
        </Link>
        <Link
          className={loaderData.view === "reviews" ? "active" : ""}
          aria-current={loaderData.view === "reviews" ? "page" : undefined}
          to="/library?view=reviews"
        >
          复习
        </Link>
      </nav>
      {loaderData.view === "saved" ? (
        <SavedList key={"saved" + loaderData.cursor} page={loaderData.page} />
      ) : (
        <Cards key={"reviews" + loaderData.cursor} page={loaderData.page} />
      )}
      <div className="review-summary-actions">
        <Link className="text-button" to="/review-history">
          复习记录
          <Icon name="chevron" />
        </Link>
        <Link className="text-button" to="/reviews">
          开始复习
          <Icon name="arrow" />
        </Link>
      </div>
    </section>
  );
}
function SavedList({ page }: { page: SavedPage }) {
  const [items, setItems] = useState(page.items);
  return (
    <>
      {!items.length && (
        <p className="profile-note">阅读时收藏的表达会放在这里。</p>
      )}
      <div className="library-list">
        {items.map((item) => (
          <SavedRow
            key={item.id}
            item={item}
            remove={() =>
              setItems((old) => old.filter((row) => row.id !== item.id))
            }
          />
        ))}
      </div>
      {page.nextCursor && (
        <Link
          className="text-button"
          to={"/library?cursor=" + encodeURIComponent(page.nextCursor)}
        >
          下一页
          <Icon name="chevron" />
        </Link>
      )}
    </>
  );
}
function SavedRow({ item, remove }: { item: SavedItem; remove: () => void }) {
  const [open, setOpen] = useState(false),
    audio = useLearning();
  return (
    <article className="library-entry">
      <button
        className="library-entry-heading"
        aria-expanded={open}
        onClick={() => {
          setOpen(!open);
          if (!open && item.vocabulary)
            audio.play([
              { id: "saved-" + item.id, text: item.vocabulary.lemma },
            ]);
        }}
      >
        <span>
          <strong lang={item.withdrawn ? "zh-CN" : "fr"}>
            {item.vocabulary?.lemma ?? "来源内容已撤回"}
          </strong>
          <small>{item.vocabulary?.meaningZh}</small>
        </span>
        <Icon name="chevron" />
      </button>
      {open && (
        <div className="library-entry-body">
          <p>{item.vocabulary?.noteZh}</p>
          {!item.withdrawn && (
            <Link
              className="text-button"
              to={"/lessons/" + item.sourceLessonId}
            >
              回看来源课程
            </Link>
          )}
          <Bookmark
            initial={item}
            knowledgeId={item.knowledgeId}
            lessonId={item.sourceLessonId}
            revision={item.sourceRevision}
            onChange={(saved) => {
              if (!saved.saved) remove();
            }}
          />
          {!item.withdrawn && (
            <Enroll
              knowledgeId={item.knowledgeId}
              lessonId={item.sourceLessonId}
              revision={item.sourceRevision}
            />
          )}
        </div>
      )}
    </article>
  );
}
function Cards({ page }: { page: ReviewCardsPage }) {
  return (
    <>
      {!page.items.length && (
        <p className="profile-note">
          学完课程或加入表达后，可以在这里管理复习。
        </p>
      )}
      <div className="library-list">
        {page.items.map((card) => (
          <ManagedCard key={card.id} initial={card} />
        ))}
      </div>
      {page.nextCursor && (
        <Link
          className="text-button"
          to={
            "/library?view=reviews&cursor=" +
            encodeURIComponent(page.nextCursor)
          }
        >
          下一页
          <Icon name="chevron" />
        </Link>
      )}
    </>
  );
}
function ManagedCard({ initial }: { initial: ReviewCard }) {
  const [card, setCard] = useState(initial),
    [open, setOpen] = useState(false),
    [readFailed, setReadFailed] = useState(false),
    [refreshing, setRefreshing] = useState(false),
    audio = useLearning();
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  async function refresh() {
    setRefreshing(true);
    try {
      const fresh = await privateRequest<ReviewCard>(
        "/api/v1/me/reviews/" + initial.id,
        "GET",
      );
      if (mounted.current) {
        setCard(fresh);
        setReadFailed(false);
      }
    } catch (error) {
      if (mounted.current) setReadFailed(true);
      throw error;
    } finally {
      if (mounted.current) setRefreshing(false);
    }
  }
  const write = useOwnedWrite<ReviewCard>(refresh, {
    userId: audio.profile?.id,
    target: { kind: "preference", cardId: initial.id },
    accept: setCard,
  });
  return (
    <article className="library-entry">
      <button
        className="library-entry-heading"
        aria-expanded={open}
        onClick={() => {
          setOpen(!open);
          if (!open)
            audio.play([
              { id: "managed-" + card.id, text: card.vocabulary.lemma },
            ]);
        }}
      >
        <span>
          <strong lang="fr">{card.vocabulary.lemma}</strong>
          <small>{card.suspended ? "已暂停" : card.vocabulary.meaningZh}</small>
        </span>
        <Icon name="chevron" />
      </button>
      {open && (
        <div className="library-entry-body">
          <p>{card.vocabulary.noteZh}</p>
          <button
            className="text-button"
            disabled={write.blocked || readFailed || refreshing}
            onClick={() =>
              write.write(
                "/api/v1/me/reviews/" + card.id + "/preferences",
                { cardVersion: card.version, suspended: !card.suspended },
                setCard,
              )
            }
          >
            {write.saving
              ? "正在保存"
              : card.suspended
                ? "恢复复习"
                : "暂停复习"}
          </button>
          {write.error && (
            <p className="error-message" role="alert">
              {write.error}
            </p>
          )}
          {readFailed && (
            <>
              <p className="error-message" role="alert">
                最新记录暂时无法读取，读取成功后再继续操作。
              </p>
              <button
                className="text-button"
                disabled={refreshing}
                onClick={() => void refresh().catch(() => {})}
              >
                {refreshing ? "正在读取" : "重新读取记录"}
              </button>
            </>
          )}
          {write.uncertain && (
            <button
              className="text-button"
              disabled={write.saving}
              onClick={write.retry}
            >
              重试保存
            </button>
          )}
          <Link className="text-button" to={"/lessons/" + card.sourceLessonId}>
            回看来源课程
          </Link>
        </div>
      )}
    </article>
  );
}
