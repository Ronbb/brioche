import { Form, Link, data, useLocation } from "react-router";
import { useEffect, useRef, useState } from "react";
import type { AdminAccounts } from "@brioche/contracts/AdminAccounts";
import type { AdminTokenResult } from "@brioche/contracts/AdminTokenResult";
import { getIdentity, getPrivate } from "../lib/api.server";
import { adminWrite } from "../lib/admin.client";
import { usePageCursorFocus } from "../components/page-cursor-focus";
import type { Route } from "./+types/admin-accounts";
export async function loader({ request }: Route.LoaderArgs) {
  const { user } = await getIdentity(request);
  if (!user) throw new Response("请先登录。", { status: 401 });
  if (user.role !== "operator")
    throw new Response("仅管理员可以进入。", { status: 403 });
  const query = new URL(request.url).searchParams;
  const search = new URLSearchParams();
  for (const key of ["q", "afterId"]) {
    const value = query.get(key);
    if (value !== null) search.set(key, value);
  }
  return data(
    await getPrivate<AdminAccounts>(
      request,
      `/api/v1/operator/accounts${search.size ? `?${search}` : ""}`,
    ),
    { headers: { "Cache-Control": "private, no-store", Vary: "Cookie" } },
  );
}
export function headers() {
  return { "Cache-Control": "private, no-store", Vary: "Cookie" };
}
export default function Accounts({
  loaderData: accounts,
}: Route.ComponentProps) {
  const location = useLocation();
  const query = new URLSearchParams(location.search);
  const heading = usePageCursorFocus(location.search);
  const dialog = useRef<HTMLDialogElement>(null);
  const busy = useRef(false);
  const write = useRef<AbortController | null>(null);
  useEffect(() => () => write.current?.abort(), []);
  const [kind, setKind] = useState<"invite" | "reset">("invite");
  const [email, setEmail] = useState("");
  const [operator, setOperator] = useState(false);
  const [reason, setReason] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const [link, setLink] = useState("");
  const [expires, setExpires] = useState(0);
  function open(next: "invite" | "reset", email = "") {
    setKind(next);
    setEmail(email);
    setOperator(false);
    setReason("");
    setError("");
    setLink("");
    setExpires(0);
    dialog.current?.showModal();
  }
  async function submit() {
    if (busy.current || link || !email.trim() || !reason.trim()) return;
    busy.current = true;
    const controller = new AbortController();
    write.current = controller;
    setPending(true);
    setError("");
    try {
      const result = await adminWrite<AdminTokenResult>(
        "accounts/token",
        {
          kind,
          email,
          operator,
          reason,
        },
        controller.signal,
      );
      controller.signal.throwIfAborted();
      const fragment = new URLSearchParams({
        token: result.token,
        email: result.email,
      });
      setLink(
        `${window.location.origin}/${result.kind === "invite" ? "invite" : "reset-password"}#${fragment}`,
      );
      setExpires(result.expiresInSeconds / 60);
    } catch (error) {
      if (controller.signal.aborted) return;
      setError(
        error instanceof Error ? error.message : "生成未确认，请核对后重试。",
      );
    } finally {
      busy.current = false;
      if (!controller.signal.aborted) setPending(false);
    }
  }
  const next = new URLSearchParams();
  if (query.get("q")) next.set("q", query.get("q")!);
  if (accounts.nextId) next.set("afterId", accounts.nextId);
  return (
    <section className="admin-page page-arrive">
      <div className="admin-heading">
        <h1 ref={heading} tabIndex={-1}>
          账号管理
        </h1>
        <Link className="text-button" to="/admin">
          管理员后台
        </Link>
      </div>
      <Form method="get" className="course-search">
        <label htmlFor="account-search">姓名或邮箱</label>
        <input
          id="account-search"
          name="q"
          type="search"
          maxLength={100}
          defaultValue={query.get("q") ?? ""}
          key={query.get("q") ?? ""}
        />
        <button className="text-button">搜索</button>
      </Form>
      <button className="admin-tool" onClick={() => open("invite")}>
        邀请新账号
      </button>
      {!accounts.items.length && <p role="status">没有匹配的账号。</p>}
      <div className="admin-list">
        {accounts.items.map((account) => (
          <article className="admin-card" key={account.id}>
            <div className="admin-card-heading">
              <h2>{account.displayName}</h2>
              <span>{account.role === "operator" ? "管理员" : "学习者"}</span>
            </div>
            <p className="admin-actor">{account.email}</p>
            <button
              className="text-button"
              onClick={() => open("reset", account.email)}
            >
              生成密码重置链接
            </button>
          </article>
        ))}
      </div>
      {accounts.nextId && (
        <Link className="text-button" to={`/admin/accounts?${next}`}>
          下一页
        </Link>
      )}
      {query.has("afterId") && (
        <Link
          className="text-button"
          to={`/admin/accounts${query.get("q") ? `?q=${encodeURIComponent(query.get("q")!)}` : ""}`}
        >
          返回第一页
        </Link>
      )}
      <dialog
        ref={dialog}
        className="choice-dialog admin-dialog"
        onCancel={(event) => {
          if (busy.current) event.preventDefault();
        }}
        onClose={() => {
          setLink("");
          setEmail("");
          setReason("");
          setError("");
        }}
      >
        <h2>{kind === "invite" ? "邀请新账号" : "密码重置链接"}</h2>
        {link ? (
          <>
            <p>
              链接有效期{" "}
              {expires >= 60 ? `${expires / 60} 小时` : `${expires} 分钟`}
              ，仅本次显示。
            </p>
            <label htmlFor="account-link">一次性链接</label>
            <textarea
              id="account-link"
              autoFocus
              readOnly
              value={link}
              onFocus={(event) => event.currentTarget.select()}
            />
            <p>选中复制后，通过你选择的方式交给本人。</p>
          </>
        ) : (
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void submit();
            }}
          >
            <p>
              {kind === "invite"
                ? "邀请有效期为 48 小时。"
                : "重置链接有效期为 30 分钟，使用后该账号现有登录会话失效。"}
              重新生成会使之前的链接失效。
            </p>
            <label htmlFor="account-email">邮箱</label>
            <input
              id="account-email"
              type="email"
              required
              maxLength={254}
              value={email}
              readOnly={pending || kind === "reset"}
              onChange={(event) => setEmail(event.target.value)}
            />
            {kind === "invite" && (
              <label>
                <input
                  type="checkbox"
                  checked={operator}
                  disabled={pending}
                  onChange={(event) => setOperator(event.target.checked)}
                />
                邀请为管理员，可管理课程和账号
              </label>
            )}
            <label htmlFor="account-reason">操作理由</label>
            <textarea
              id="account-reason"
              required
              maxLength={300}
              value={reason}
              readOnly={pending}
              onChange={(event) => setReason(event.target.value)}
            />
            <p role="alert">{error}</p>
            <button
              className="primary"
              disabled={!reason.trim() || !email.trim()}
              aria-disabled={pending}
              aria-busy={pending}
            >
              {pending ? "正在生成" : "生成链接"}
            </button>
          </form>
        )}
        <button
          className="text-button"
          aria-disabled={pending}
          onClick={() => {
            if (!busy.current) dialog.current?.close();
          }}
        >
          关闭
        </button>
      </dialog>
    </section>
  );
}
