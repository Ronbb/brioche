import { Form, Link, data } from "react-router";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import type { PreviewRelease } from "@brioche/contracts/PreviewRelease";
import { getIdentity, getPrivate } from "../lib/api.server";
import { ReadingBlock } from "../components/reading-block";
import { TeachingBlock } from "../components/teaching-block";
import type { Route } from "./+types/author-preview";

export async function loader({ request }: Route.LoaderArgs) {
  const { user } = await getIdentity(request);
  if (!user) throw new Response("请先登录。", { status: 401 });
  if (user.role !== "operator")
    throw new Response("仅内容管理员可以预览。", { status: 403 });
  const query = new URL(request.url).searchParams;
  const id = query.get("lessonId") ?? "",
    revision = query.get("revision") ?? "";
  const releaseId = query.get("releaseId") ?? "";
  let release: PreviewRelease | null = null;
  if (releaseId) {
    if (!/^[A-Za-z0-9_-]{1,100}$/.test(releaseId))
      throw new Response("发布批次编号无效。", { status: 400 });
    release = await getPrivate<PreviewRelease>(
      request,
      `/api/v1/operator/releases/${releaseId}`,
    );
  }
  let lesson: PublicLesson | null = null;
  if (id || revision) {
    if (
      !/^[A-Za-z0-9_-]{1,100}$/.test(id) ||
      !/^[1-9][0-9]*$/.test(revision) ||
      Number(revision) > 2147483647
    )
      throw new Response("课程编号或版本无效。", { status: 400 });
    if (
      release &&
      !release.catalog.levels.some((level) =>
        level.units.some((unit) =>
          unit.lessons.some(
            (lesson) =>
              lesson.id === id && lesson.revision === Number(revision),
          ),
        ),
      )
    )
      throw new Response("这堂课不属于所选发布批次。", { status: 404 });
    lesson = await getPrivate<PublicLesson>(
      request,
      `/api/v1/operator/lessons/${id}/revisions/${revision}`,
    );
  }
  return data(
    { lesson, id, revision, release, releaseId },
    { headers: { "Cache-Control": "private, no-store", Vary: "Cookie" } },
  );
}
export function headers() {
  return { "Cache-Control": "private, no-store", Vary: "Cookie" };
}

export default function AuthorPreview({
  loaderData: { lesson, id, revision, release, releaseId },
}: Route.ComponentProps) {
  return (
    <section className="page-arrive author-preview">
      <div className="lesson-header">
        <h1>课程预览</h1>
        <p className="profile-note">预览已导入的固定版本，不记录学习进度。</p>
      </div>
      <Form method="get" className="auth-form">
        <label>
          发布批次
          <input
            name="releaseId"
            defaultValue={releaseId}
            required
            pattern="[A-Za-z0-9_-]{1,100}"
          />
        </label>
        <button className="primary">预览整批课程</button>
      </Form>
      {release && (
        <div className="course-directory">
          <h2>{release.id}</h2>
          {release.catalog.levels.map((level) => (
            <section key={level.id}>
              <h2>{level.label}</h2>
              {level.units.map((unit) => (
                <section key={unit.id}>
                  <h3>{unit.titleZh}</h3>
                  <ol className="lesson-list">
                    {unit.lessons.map((item) => (
                      <li key={item.id}>
                        {release.withdrawnLessonIds.includes(item.id) ? (
                          <p>
                            {item.title.zh} · 第 {item.revision} 版 · 已撤回
                          </p>
                        ) : (
                          <Link
                            className="setting-row setting-link"
                            to={
                              "/author-preview?" +
                              new URLSearchParams({
                                releaseId: release.id,
                                lessonId: item.id,
                                revision: String(item.revision),
                              })
                            }
                          >
                            <span>
                              {item.title.zh}
                              <small lang="fr">{item.title.fr}</small>
                            </span>
                            <span>第 {item.revision} 版</span>
                          </Link>
                        )}
                      </li>
                    ))}
                  </ol>
                </section>
              ))}
            </section>
          ))}
          {!release.catalog.levels.length && <p>此发布批次没有课程。</p>}
        </div>
      )}
      <Form method="get" className="auth-form">
        {releaseId && (
          <input type="hidden" name="releaseId" value={releaseId} />
        )}
        <label>
          课程编号
          <input
            name="lessonId"
            defaultValue={id}
            required
            pattern="[A-Za-z0-9_-]{1,100}"
          />
        </label>
        <label>
          版本
          <input
            name="revision"
            type="number"
            min="1"
            max="2147483647"
            step="1"
            defaultValue={revision || "1"}
            required
          />
        </label>
        <button className="primary">打开预览</button>
      </Form>
      {lesson && (
        <div key={lesson.id + ":" + lesson.revision}>
          <div className="lesson-header">
            <h2 lang="fr">{lesson.title.fr}</h2>
            <p>
              {lesson.title.zh} · 第 {lesson.revision} 版
            </p>
          </div>
          {lesson.steps.map((step) => (
            <section key={step.id} className="reading">
              <h2>{step.titleZh}</h2>
              {step.blockIds.map((id) => {
                const block = lesson.blocks.find((b) => b.id === id)!;
                if (block.type === "dialogue" || block.type === "article")
                  return (
                    <ReadingBlock key={id} block={block} lesson={lesson} />
                  );
                if (block.type === "exercise")
                  return (
                    <div key={id} className="lesson-note">
                      <h3>{block.promptZh}</h3>
                      {block.exerciseType === "single-choice" && (
                        <ul>
                          {block.options.map((option) => (
                            <li key={option.id}>{option.text}</li>
                          ))}
                        </ul>
                      )}
                      {block.exerciseType === "fill-blank" && (
                        <>
                          <p lang="fr">{block.templateFr}</p>
                          <p>{block.hintZh}</p>
                        </>
                      )}
                      {block.exerciseType === "order" && (
                        <p lang="fr">
                          {block.tokens.map((token) => token.text).join(" · ")}
                        </p>
                      )}
                    </div>
                  );
                return <TeachingBlock key={id} block={block} lesson={lesson} />;
              })}
            </section>
          ))}
        </div>
      )}
    </section>
  );
}
