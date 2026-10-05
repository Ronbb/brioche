import { useLearning } from "../components/learning";
import { Icon } from "../components/icon";
export default function Profile() {
  const learning = useLearning();
  return (
    <section className="settings-page page-arrive">
      <div className="settings-heading">
        <h1>我的</h1>
      </div>
      <div className="profile-summary">
        <img src="/assets/avatars/learner.svg" alt="" />
        <div>
          <h2>法语学习者</h2>
          <p>一点法语，一点生活。</p>
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
      <p className="profile-note">当前为访客试学，设置仅在本次浏览中保留。</p>
    </section>
  );
}
