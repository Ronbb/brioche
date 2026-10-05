import { useEffect, useState } from "react";
import { Link, redirect } from "react-router";
import { getIdentity } from "../lib/api.server";
import {
  ApiRequestError,
  definitiveWriteFailure,
  privateRequest,
} from "../lib/api.client";
import { clearPending } from "../lib/learning-draft";
import { pendingOwned } from "../lib/owned-draft";
import { Icon } from "../components/icon";
import type { Route } from "./+types/pending-saves";
export async function loader({ request }: Route.LoaderArgs) {
  const identity = await getIdentity(request);
  if (!identity.user) throw redirect("/login?next=/pending-saves");
  return { userId: identity.user.id };
}
export default function PendingSaves({
  loaderData: { userId },
}: Route.ComponentProps) {
  const [items, setItems] = useState<ReturnType<typeof pendingOwned>>([]),
    [ready, setReady] = useState(false),
    [busy, setBusy] = useState(""),
    [error, setError] = useState("");
  useEffect(() => {
    setItems(pendingOwned(userId));
    setReady(true);
  }, [userId]);
  async function confirm(item: (typeof items)[number]) {
    if (busy) return;
    setBusy(item.key);
    setError("");
    try {
      await privateRequest(item.job.path, item.job.method, item.job.body);
      clearPending(item.key, item.job.body.idempotencyKey);
      setItems(pendingOwned(userId));
    } catch (failure) {
      if (
        failure instanceof ApiRequestError &&
        definitiveWriteFailure(failure.status)
      ) {
        clearPending(item.key, item.job.body.idempotencyKey);
        setItems(pendingOwned(userId));
        setError(
          failure.status === 409
            ? "记录已在其他地方更新，请回到原页面检查后继续。"
            : failure.message,
        );
      } else setError("保存仍未确认，可以稍后重试同一请求。");
    } finally {
      setBusy("");
    }
  }
  return (
    <section className="settings-page page-arrive">
      <div className="section-head">
        <h1>未确认保存</h1>
        <Link className="text-button" to="/profile">
          回到我的
        </Link>
      </div>
      <p className="profile-note">此标签页中尚未确认的收藏和复习操作。</p>
      {!ready ? (
        <p role="status">正在读取</p>
      ) : !items.length ? (
        <p role="status">没有待确认的保存。</p>
      ) : (
        <div className="library-list">
          {items.map((item) => (
            <div key={item.key} className="library-entry">
              <div className="section-head">
                <span>
                  {item.target.kind === "bookmark"
                    ? item.job.body.saved
                      ? "收藏表达"
                      : "取消收藏"
                    : item.target.kind === "enroll"
                      ? "加入复习"
                      : item.target.kind === "preference"
                        ? "调整复习状态"
                        : "复习自评"}
                </span>
                <button
                  className="text-button"
                  disabled={!!busy}
                  onClick={() => void confirm(item)}
                >
                  {busy === item.key ? "正在确认" : "确认原提交"}
                  <Icon name="check" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
      {error && (
        <p className="error-message" role="alert">
          {error}
        </p>
      )}
      <Link className="text-button practice-back" to="/library">
        查看收藏与复习
      </Link>
    </section>
  );
}
