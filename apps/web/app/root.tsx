import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  Link,
  isRouteErrorResponse,
  useRouteError,
  data,
  useRouteLoaderData,
} from "react-router";
import { LearningProvider } from "./components/learning";
import { IdentitySync } from "./components/identity-sync";
import "./styles/app.css";
import { Scrollbar } from "./components/scrollbar";
import { RouteFocus } from "./components/route-focus";
import { getIdentity } from "./lib/api.server";
import type { Route } from "./+types/root";
import { useCallback, useState } from "react";
export async function loader({ request }: Route.LoaderArgs) {
  return data(await getIdentity(request), {
    headers: { "Cache-Control": "private, no-store", Vary: "Cookie" },
  });
}
export function headers() {
  return { "Cache-Control": "private, no-store", Vary: "Cookie" };
}
export function Layout({ children }: { children: React.ReactNode }) {
  const identity = useRouteLoaderData<typeof loader>("root");
  const [identityInvalidated, setIdentityInvalidated] = useState(false);
  const invalidateIdentity = useCallback(
    () => setIdentityInvalidated(true),
    [],
  );
  return (
    <html lang="zh-CN" className="overlay-scroll">
      <head>
        <meta charSet="utf-8" />
        <meta
          name="viewport"
          content="width=device-width,initial-scale=1,viewport-fit=cover"
        />
        <meta name="theme-color" content="#fffaef" />
        <meta name="application-name" content="Brioche" />
        <meta name="mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="apple-mobile-web-app-title" content="Brioche" />
        <meta name="apple-mobile-web-app-status-bar-style" content="default" />
        <link rel="manifest" href="/manifest.webmanifest" />
        <link rel="icon" href="/icons/app.svg" type="image/svg+xml" />
        <link
          rel="icon"
          href="/icons/favicon-32.png"
          type="image/png"
          sizes="32x32"
        />
        <link
          rel="apple-touch-icon"
          href="/apple-touch-icon.png"
          sizes="180x180"
        />
        <title>Brioche · 一点法语，一点生活</title>
        <Meta />
        <Links />
      </head>
      <body>
        <a className="skip-link" href="#page-content">
          跳到正文
        </a>
        <LearningProvider
          key={identityInvalidated ? "invalidated" : "active"}
          user={identityInvalidated ? null : (identity?.user ?? null)}
        >
          <IdentitySync
            user={identityInvalidated ? null : (identity?.user ?? null)}
            enabled={!identityInvalidated && (identity?.enabled ?? false)}
            onInvalidate={invalidateIdentity}
          />
          <div className="app">
            <header className="topbar">
              <Link className="brand" to="/" aria-label="Brioche 首页">
                <span className="brand-mark" aria-hidden="true">
                  <svg viewBox="0 0 32 32" width="27" height="27">
                    <path
                      d="M6 15c-4-7 5-12 9-6 4-8 14-3 11 5 4 2 3 7-1 8H8c-5-1-6-5-2-7Z"
                      fill="currentColor"
                    />
                  </svg>
                </span>
                <span>brioche.</span>
              </Link>
              <Link
                className="profile-settings profile-avatar"
                to="/profile"
                aria-label="个人信息与设置"
              >
                <img
                  src="/assets/avatars/learner.svg"
                  alt=""
                  width="44"
                  height="44"
                />
              </Link>
            </header>
            <main id="page-content" tabIndex={-1}>
              {identityInvalidated ? (
                <section className="home">
                  <h1 tabIndex={-1}>账号状态已更新</h1>
                  <p role="status">正在重新确认账号。</p>
                  <button
                    className="primary"
                    onClick={() => window.location.reload()}
                  >
                    重新加载
                  </button>
                </section>
              ) : (
                children
              )}
            </main>
          </div>
          <Scrollbar />
          <RouteFocus />
        </LearningProvider>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}
export default function App() {
  return <Outlet />;
}
export function ErrorBoundary() {
  const error = useRouteError();
  return (
    <section className="home">
      <h1>暂时无法打开</h1>
      <p className="error-message">
        {isRouteErrorResponse(error)
          ? String(error.data)
          : "页面暂时遇到问题，请刷新重试。"}
      </p>
      <Link className="primary" to="/">
        返回首页
      </Link>
    </section>
  );
}
