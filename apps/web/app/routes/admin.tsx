import { Link, data, useRevalidator } from "react-router";
import { useEffect, useRef, useState } from "react";
import type { AdminImportResult } from "@brioche/contracts/AdminImportResult";
import type { AdminOverview } from "@brioche/contracts/AdminOverview";
import type { AdminLesson } from "@brioche/contracts/AdminLesson";
import { getIdentity, getPrivate } from "../lib/api.server";
import { adminWrite } from "../lib/admin.client";
import type { Route } from "./+types/admin";

export async function loader({ request }: Route.LoaderArgs) {
  const { user } = await getIdentity(request);
  if (!user) throw new Response("请先登录。", { status: 401 });
  if (user.role !== "operator")
    throw new Response("仅管理员可以进入。", { status: 403 });
  return data(
    await getPrivate<AdminOverview>(request, "/api/v1/operator/overview"),
    {
      headers: { "Cache-Control": "private, no-store", Vary: "Cookie" },
    },
  );
}
export function headers() {
  return { "Cache-Control": "private, no-store", Vary: "Cookie" };
}

export default function Admin({ loaderData: overview }: Route.ComponentProps) {
  const refresh = useRevalidator();
  const [tab, setTab] = useState<"lessons" | "releases">("lessons");
  const [target, setTarget] = useState<{
    lesson?: AdminLesson;
    release?: string;
    operation:
      "approve" | "reject" | "withdraw" | "activate" | "import" | "stage";
  } | null>(null);
  const [reason, setReason] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [document, setDocument] = useState("");
  const [filename, setFilename] = useState("");
  const [readingFile, setReadingFile] = useState(false);
  const [fileSession, setFileSession] = useState(0);
  const fileSequence = useRef(0);
  const write = useRef<AbortController | null>(null);
  useEffect(
    () => () => {
      fileSequence.current += 1;
      write.current?.abort();
    },
    [],
  );
  const busy = useRef(false);
  const dialog = useRef<HTMLDialogElement>(null);
  function open(next: NonNullable<typeof target>) {
    setTarget(next);
    setReason("");
    setError("");
    setDocument("");
    setFilename("");
    setReadingFile(false);
    fileSequence.current += 1;
    setFileSession((session) => session + 1);
    dialog.current?.showModal();
  }
  async function readFile(file?: File) {
    const sequence = ++fileSequence.current;
    setDocument("");
    setFilename("");
    setError("");
    setReadingFile(false);
    if (!file) return;
    if (file.size > 2 * 1024 * 1024) {
      setError("文件不能超过 2 MiB。");
      return;
    }
    setReadingFile(true);
    try {
      const text = await file.text();
      if (fileSequence.current !== sequence || !dialog.current?.open) return;
      setDocument(text);
      setFilename(file.name);
    } catch {
      if (fileSequence.current === sequence)
        setError("文件读取失败，请重新选择。");
    } finally {
      if (fileSequence.current === sequence) setReadingFile(false);
    }
  }
  async function submit() {
    if (
      busy.current ||
      readingFile ||
      !target ||
      !reason.trim() ||
      (["import", "stage"].includes(target.operation) && !document)
    )
      return;
    busy.current = true;
    const controller = new AbortController();
    write.current = controller;
    setPending(true);
    setError("");
    try {
      const lesson = target.lesson;
      if (target.operation === "import") {
        const result = await adminWrite<AdminImportResult>(
          "lessons/import",
          {
            document,
            reason,
          },
          controller.signal,
        );
        controller.signal.throwIfAborted();
        setNotice(`课程 v${result.revision} 已导入，可以预览和审批。`);
      } else if (target.operation === "stage") {
        await adminWrite(
          "releases/stage",
          { document, reason },
          controller.signal,
        );
        controller.signal.throwIfAborted();
        setNotice("发布目录已通过检查，可以预览或切换。");
      } else if (target.operation === "activate") {
        await adminWrite(
          "releases/activate",
          {
            releaseId: target.release,
            generation: overview.generation,
            reason,
          },
          controller.signal,
        );
      } else if (lesson) {
        const path = `lessons/${encodeURIComponent(lesson.id)}/revisions/${lesson.revision}`;
        if (target.operation === "withdraw")
          await adminWrite(
            `${path}/withdraw`,
            {
              generation: overview.generation,
              reason,
            },
            controller.signal,
          );
        else
          await adminWrite(
            `${path}/review`,
            {
              version: lesson.reviewVersion,
              approved: target.operation === "approve",
              reason,
            },
            controller.signal,
          );
      }
      controller.signal.throwIfAborted();
      if (!["import", "stage"].includes(target.operation))
        setNotice("操作已保存。");
      dialog.current?.close();
      refresh.revalidate();
    } catch (error) {
      if (controller.signal.aborted) return;
      setError(
        error instanceof Error ? error.message : "操作未确认，请刷新核对。",
      );
    } finally {
      busy.current = false;
      if (!controller.signal.aborted) setPending(false);
    }
  }
  const labels = {
    approve: "批准课程",
    reject: "退回课程",
    withdraw: "撤回课程版本",
    activate: "切换发布目录",
    import: "导入课程",
    stage: "创建发布目录",
  };
  return (
    <section className="admin-page page-arrive">
      <div className="admin-heading">
        <div>
          <p className="eyebrow">BRIOCHE STUDIO</p>
          <h1>管理员后台</h1>
        </div>
        <Link className="text-button" to="/profile">
          个人页
        </Link>
      </div>
      <div className="admin-status">
        <span>当前发布</span>
        <strong>{overview.activeRelease ?? "尚未发布课程"}</strong>
        <span>版本 {overview.generation}</span>
      </div>
      <Link className="text-button" to="/admin/history">
        审批与发布记录
      </Link>
      <Link className="text-button" to="/admin/accounts">
        账号管理
      </Link>
      <div className="reader-mode" role="group" aria-label="管理内容">
        <button
          aria-pressed={tab === "lessons"}
          onClick={() => setTab("lessons")}
        >
          课程审批
        </button>
        <button
          aria-pressed={tab === "releases"}
          onClick={() => setTab("releases")}
        >
          发布目录
        </button>
      </div>
      <div className="admin-toolbar">
        <button
          className="admin-tool"
          aria-label={tab === "lessons" ? "导入课程" : "创建发布目录"}
          onClick={() =>
            open({ operation: tab === "lessons" ? "import" : "stage" })
          }
        >
          <strong>{tab === "lessons" ? "导入课程" : "创建发布目录"}</strong>
          <span>
            {tab === "lessons"
              ? "选择课源文件，导入固定版本"
              : "选择目录文件，检查课程与素材"}
          </span>
        </button>
        <button
          className="text-button"
          aria-disabled={refresh.state !== "idle"}
          onClick={() => {
            if (refresh.state === "idle") refresh.revalidate();
          }}
        >
          刷新列表
        </button>
      </div>
      <p role="status">{notice}</p>
      {tab === "lessons" ? (
        <div className="admin-list">
          {!overview.lessons.length && (
            <div className="admin-empty">
              <h2>还没有导入的课程</h2>
              <p>课程导入后会出现在这里，批准和发布分别管理。</p>
            </div>
          )}
          {overview.lessons.map((lesson) => (
            <article
              className="admin-card"
              key={`${lesson.id}:${lesson.revision}`}
            >
              <div className="admin-card-heading">
                <span className="profile-level">
                  {lesson.level.toUpperCase()} · v{lesson.revision}
                </span>
                <span>
                  {lesson.withdrawn
                    ? "已撤回"
                    : lesson.published
                      ? "曾发布"
                      : lesson.approved
                        ? "已批准"
                        : "待批准"}
                </span>
              </div>
              <h2>{lesson.title}</h2>
              <p className="admin-note">{lesson.reviewNote}</p>
              <div className="admin-card-actions">
                {!lesson.withdrawn && (
                  <Link
                    className="text-button"
                    to={`/author-preview?lessonId=${lesson.id}&revision=${lesson.revision}`}
                  >
                    打开预览
                  </Link>
                )}
                {!lesson.withdrawn && !lesson.published && (
                  <>
                    <button
                      className="text-button"
                      onClick={() =>
                        open({
                          lesson,
                          operation: lesson.approved ? "reject" : "approve",
                        })
                      }
                    >
                      {lesson.approved ? "退回修改" : "批准课程"}
                    </button>
                  </>
                )}
                {!lesson.withdrawn && lesson.published && (
                  <button
                    className="text-button"
                    onClick={() => open({ lesson, operation: "withdraw" })}
                  >
                    撤回版本
                  </button>
                )}
              </div>
            </article>
          ))}
        </div>
      ) : (
        <div className="admin-list">
          {!overview.releases.length && (
            <div className="admin-empty">
              <h2>还没有发布目录</h2>
              <p>通过 staging 检查的目录会显示在这里。</p>
            </div>
          )}
          {overview.releases.map((release) => (
            <article className="admin-card" key={release.id}>
              <h2>{release.id}</h2>
              <p>
                {release.lessonCount} 课 ·{" "}
                {release.id === overview.activeRelease
                  ? "当前目录"
                  : "可切换目录"}
              </p>
              <div className="admin-card-actions">
                <Link
                  className="text-button"
                  to={`/author-preview?releaseId=${release.id}`}
                >
                  预览目录
                </Link>
                {release.id !== overview.activeRelease && (
                  <button
                    className="text-button"
                    onClick={() =>
                      open({ release: release.id, operation: "activate" })
                    }
                  >
                    切换到此目录
                  </button>
                )}
              </div>
            </article>
          ))}
        </div>
      )}
      <dialog
        ref={dialog}
        className="choice-dialog admin-dialog"
        onClose={() => {
          fileSequence.current += 1;
          setDocument("");
          setFilename("");
          setReadingFile(false);
        }}
        onCancel={(event) => {
          if (busy.current) event.preventDefault();
        }}
      >
        <h2>{target && labels[target.operation]}</h2>
        <p>{target?.lesson?.title ?? target?.release}</p>
        {target?.operation === "import" && (
          <p>
            图片、头像和录音需先登记。相同版本不会覆盖已有内容；修改课程请使用新版本。
          </p>
        )}
        {target?.operation === "stage" && (
          <p>
            目录中的课程必须已批准，素材需通过发布检查。创建后先预览，再切换正式目录。
          </p>
        )}
        {target?.operation === "withdraw" && (
          <p>此版本会永久停止访问，已有学习记录保留。恢复内容需要新版本。</p>
        )}
        {target?.operation === "activate" && (
          <p>
            学习目录将整体切换，系统会重新检查审批和素材。旧学习会话继续固定原版本。
          </p>
        )}
        <form
          onSubmit={(event) => {
            event.preventDefault();
            void submit();
          }}
        >
          {target && ["import", "stage"].includes(target.operation) && (
            <>
              <label htmlFor="admin-document">
                {target.operation === "import" ? "课程文件" : "目录文件"}
              </label>
              <input
                id="admin-document"
                type="file"
                accept=".json,application/json"
                disabled={pending}
                key={fileSession}
                onChange={(event) => {
                  void readFile(event.target.files?.[0]);
                }}
              />
              <p role="status">{readingFile ? "正在读取文件" : filename}</p>
            </>
          )}
          <label htmlFor="admin-reason">操作理由</label>
          <textarea
            id="admin-reason"
            required
            maxLength={300}
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            readOnly={pending}
          />
          <p role="alert">{error}</p>
          <button
            className="primary"
            aria-disabled={pending}
            aria-busy={pending}
            disabled={
              !reason.trim() ||
              readingFile ||
              !!(
                target &&
                ["import", "stage"].includes(target.operation) &&
                !document
              )
            }
          >
            {pending ? "正在保存" : "确认操作"}
          </button>
          <button
            type="button"
            className="text-button"
            aria-disabled={pending}
            onClick={() => {
              if (!busy.current) dialog.current?.close();
            }}
          >
            取消
          </button>
        </form>
      </dialog>
    </section>
  );
}
