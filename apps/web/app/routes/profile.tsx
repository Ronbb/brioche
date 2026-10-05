import { useLearning } from "../components/learning";
import { Icon } from "../components/icon";
import { Link, useRouteLoaderData } from "react-router";
import type { loader } from "../root";
import { authRequest } from "../lib/auth.client";
import { useState } from "react";
export default function Profile() {
  const learning = useLearning();
  const identity = useRouteLoaderData<typeof loader>("root");
  const [pending, setPending] = useState(false);
  async function logout() {
    if (pending) return;
    setPending(true);
    try {
      await authRequest("logout");
      learning.stop();
      window.location.assign("/");
    } catch {
      learning.toast("退出未完成，请重试。");
      setPending(false);
    }
  }
  return (
    <section className="settings-page page-arrive">
      <div className="settings-heading">
        <h1>我的</h1>
      </div>
      <div className="profile-summary">
        <img src="/assets/avatars/learner.svg" alt="" />
        <div>
          <h2>{identity?.user?.displayName ?? "法语学习者"}</h2>
          <p>{identity?.user?.email ?? "一点法语，一点生活。"}</p>
          <span className="profile-level">A1–A2</span>
        </div>
      </div>
      <h2>阅读</h2>
      <div className="settings-group">
        <div className="setting-row">
          <span id="translation-label">默认显示中文译文</span>
          <button
            className="translation-switch"
            role="switch"
            aria-checked={learning.translation}
            aria-labelledby="translation-label"
            onClick={() => learning.setTranslation(!learning.translation)}
          >
            <span className="switch-track" aria-hidden="true" />
          </button>
        </div>
        <div className="setting-row">
          <span>朗读速度</span>
          <button className="speed-trigger" onClick={learning.openRate}>
            {learning.rate}×<Icon name="chevron" />
          </button>
        </div>
      </div>
      <p className="profile-note">阅读偏好暂时仅在本次浏览中保留。</p>
      {identity?.user ? (
        <button
          className="text-button"
          disabled={pending}
          onClick={() => void logout()}
        >
          {pending ? "正在退出" : "退出登录"}
        </button>
      ) : (
        identity?.enabled && (
          <Link className="primary" to="/login">
            登录账号
            <Icon name="arrow" />
          </Link>
        )
      )}
    </section>
  );
}
