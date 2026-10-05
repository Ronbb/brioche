import { useEffect, useRef, useState } from "react";
import { Link, useRouteLoaderData } from "react-router";
import type { loader } from "../root";
import type { LoginRequest } from "@brioche/contracts/LoginRequest";
import type { AcceptInviteRequest } from "@brioche/contracts/AcceptInviteRequest";
import type { ResetPasswordRequest } from "@brioche/contracts/ResetPasswordRequest";
import { authRequest } from "../lib/auth.client";
import { useLearning } from "./learning";
import { Icon } from "./icon";
export function Account({
  mode,
}: {
  mode: "login" | "invite" | "reset-password";
}) {
  const identity = useRouteLoaderData<typeof loader>("root");
  const learning = useLearning();
  const busy = useRef(false);
  const [email, setEmail] = useState(""),
    [password, setPassword] = useState(""),
    [name, setName] = useState(""),
    [token, setToken] = useState(""),
    [pending, setPending] = useState(false),
    [error, setError] = useState(""),
    [done, setDone] = useState(false);
  useEffect(() => {
    if (mode === "login") return;
    function readLink() {
      if (!window.location.hash) return;
      const params = new URLSearchParams(window.location.hash.slice(1));
      setToken(params.get("token") ?? "");
      setEmail(params.get("email") ?? "");
      window.history.replaceState(
        window.history.state,
        "",
        window.location.pathname,
      );
    }
    readLink();
    window.addEventListener("hashchange", readLink);
    return () => window.removeEventListener("hashchange", readLink);
  }, [mode]);
  const title =
    mode === "login"
      ? "欢迎回来"
      : mode === "invite"
        ? "开始你的法语日常"
        : "设置新密码";
  async function submit() {
    if (busy.current) return;
    busy.current = true;
    setPending(true);
    setError("");
    try {
      const body: LoginRequest | AcceptInviteRequest | ResetPasswordRequest =
        mode === "login"
          ? { email, password }
          : mode === "invite"
            ? { email, password, token, displayName: name }
            : { token, password };
      await authRequest(mode === "invite" ? "accept-invite" : mode, body);
      setPassword("");
      learning.stop();
      if (mode === "reset-password") {
        setDone(true);
        setToken("");
      } else {
        const next =
          mode === "login"
            ? new URLSearchParams(window.location.search).get("next")
            : null;
        window.location.assign(
          next &&
            (next === "/reviews" ||
              /^\/(?:learning|lessons)\/[a-zA-Z0-9_-]+$/.test(next))
            ? next
            : "/profile",
        );
      }
    } catch (e) {
      setError(
        e instanceof Error && !["TypeError", "TimeoutError"].includes(e.name)
          ? e.message
          : "请求未完成，请稍后重试。",
      );
    } finally {
      busy.current = false;
      setPending(false);
    }
  }
  return (
    <section className="account-page page-arrive">
      <h1>{done ? "密码已更新" : title}</h1>
      {done ? (
        <>
          <p>旧会话已退出，请用新密码登录。</p>
          <Link className="primary" to="/login">
            登录
            <Icon name="arrow" />
          </Link>
        </>
      ) : !identity?.enabled ? (
        <>
          <p>当前是访客试学。账号功能需要连接数据库后使用。</p>
          <Link className="text-button" to="/">
            返回课程
          </Link>
        </>
      ) : mode !== "login" && !token ? (
        <>
          <p>请通过管理员提供的完整链接打开此页面。</p>
          <Link className="text-button" to="/login">
            返回登录
          </Link>
        </>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void submit();
          }}
        >
          {mode !== "reset-password" && (
            <label>
              邮箱
              <input
                type="email"
                required
                autoComplete={mode === "login" ? "username" : "email"}
                maxLength={254}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                disabled={pending}
              />
            </label>
          )}
          {mode === "invite" && (
            <label>
              怎么称呼你
              <input
                required
                autoComplete="nickname"
                maxLength={80}
                value={name}
                onChange={(e) => setName(e.target.value)}
                disabled={pending}
              />
            </label>
          )}
          <label>
            {mode === "reset-password" ? "新密码" : "密码"}
            <input
              type="password"
              required
              minLength={mode === "login" ? undefined : 12}
              maxLength={128}
              autoComplete={
                mode === "login" ? "current-password" : "new-password"
              }
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={pending}
            />
          </label>
          {mode !== "login" && (
            <p className="profile-note">
              使用 12–128 个字符，建议设置容易记住的长密码。
            </p>
          )}
          {error && (
            <p className="error-message" role="alert">
              {error}
            </p>
          )}
          <button className="primary" disabled={pending}>
            {pending
              ? "正在确认"
              : mode === "login"
                ? "登录"
                : mode === "invite"
                  ? "创建账号"
                  : "更新密码"}
            <Icon name="arrow" />
          </button>
          {mode === "login" && (
            <p className="profile-note">
              需要账号或忘记密码时，请联系管理员获取邀请或恢复链接。
            </p>
          )}
        </form>
      )}
    </section>
  );
}
