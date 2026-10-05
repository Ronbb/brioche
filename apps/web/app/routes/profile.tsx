import { useLearning } from "../components/learning";
import { Icon } from "../components/icon";
import { Link, useRouteLoaderData } from "react-router";
import type { loader } from "../root";
import { authRequest } from "../lib/auth.client";
import { clearLearningDrafts } from "../lib/learning-draft";
import { useEffect, useRef, useState } from "react";
import { ChoiceDialog, type Choice } from "../components/choice-dialog";
const commonZones: Choice[] = [
  { value: "Asia/Shanghai", label: "中国", detail: "Asia/Shanghai" },
  { value: "Asia/Hong_Kong", label: "香港", detail: "Asia/Hong_Kong" },
  { value: "Asia/Taipei", label: "台北", detail: "Asia/Taipei" },
  { value: "Europe/Paris", label: "法国 · 巴黎", detail: "Europe/Paris" },
  { value: "Europe/London", label: "英国 · 伦敦", detail: "Europe/London" },
  { value: "America/New_York", label: "纽约", detail: "America/New_York" },
  { value: "Asia/Tokyo", label: "东京", detail: "Asia/Tokyo" },
  { value: "UTC", label: "UTC" },
];
export default function Profile() {
  const learning = useLearning();
  const identity = useRouteLoaderData<typeof loader>("root");
  const [pending, setPending] = useState(false);
  const profile = learning.profile;
  const editor = useRef<HTMLDialogElement>(null);
  const editBusy = useRef(false);
  const [draftName, setDraftName] = useState(""),
    [zone, setZone] = useState("Asia/Shanghai"),
    [days, setDays] = useState(5),
    [minutes, setMinutes] = useState(10),
    [editing, setEditing] = useState(false),
    [zones, setZones] = useState(commonZones);
  useEffect(() => {
    const known = new Set(commonZones.map((choice) => choice.value));
    const all =
      typeof Intl.supportedValuesOf === "function"
        ? Intl.supportedValuesOf("timeZone")
        : [];
    setZones([
      ...commonZones,
      ...all
        .filter((value) => !known.has(value))
        .map((value) => ({ value, label: value })),
    ]);
  }, []);
  function openEditor() {
    if (!profile) return;
    setDraftName(profile.displayName);
    setZone(profile.settings.timeZone);
    setDays(profile.settings.weeklyDays);
    setMinutes(profile.settings.dailyMinutes);
    editor.current?.showModal();
  }
  async function save() {
    if (editBusy.current || !profile) return;
    const changes = {
      ...(draftName.trim() !== profile.displayName
        ? { displayName: draftName.trim() }
        : {}),
      ...(zone !== profile.settings.timeZone ? { timeZone: zone } : {}),
      ...(days !== profile.settings.weeklyDays ? { weeklyDays: days } : {}),
      ...(minutes !== profile.settings.dailyMinutes
        ? { dailyMinutes: minutes }
        : {}),
    };
    if (!Object.keys(changes).length) {
      editor.current?.close();
      return;
    }
    setEditing(true);
    editBusy.current = true;
    if (await learning.saveProfile(changes)) editor.current?.close();
    setEditing(false);
    editBusy.current = false;
  }
  async function logout() {
    if (pending) return;
    setPending(true);
    try {
      await authRequest("logout");
      if (learning.profile) clearLearningDrafts(learning.profile.id);
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
          <h2>{profile?.displayName ?? "法语学习者"}</h2>
          <p>{profile?.email ?? "一点法语，一点生活。"}</p>
          <span className="profile-level">A1–A2</span>
        </div>
        {profile && (
          <button
            className="icon-button profile-edit"
            aria-label="编辑个人资料与学习目标"
            onClick={openEditor}
          >
            <Icon name="chevron" />
          </button>
        )}
      </div>
      {profile && (
        <>
          <h2>学习日常</h2>
          <div className="settings-group">
            <Link className="setting-row setting-link" to="/reviews">
              <span>我的复习</span>
              <Icon name="chevron" />
            </Link>
            <Link className="setting-row setting-link" to="/library">
              <span>我的表达</span>
              <Icon name="chevron" />
            </Link>
            <Link className="setting-row setting-link" to="/review-history">
              <span>复习记录</span>
              <Icon name="chevron" />
            </Link>
            <button className="setting-row setting-link" onClick={openEditor}>
              <span>学习目标</span>
              <span>
                每周 {profile.settings.weeklyDays} 天 · 每天{" "}
                {profile.settings.dailyMinutes} 分钟
                <Icon name="chevron" />
              </span>
            </button>
            <button className="setting-row setting-link" onClick={openEditor}>
              <span>学习时区</span>
              <span>
                {zones.find(
                  (choice) => choice.value === profile.settings.timeZone,
                )?.label ?? profile.settings.timeZone}
                <Icon name="chevron" />
              </span>
            </button>
          </div>
        </>
      )}
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
      <p className="profile-note" role="status">
        {profile
          ? learning.saveStatus === "saving"
            ? "正在保存"
            : learning.saveStatus === "error"
              ? "保存未确认，请检查当前设置后重试。"
              : "设置已保存到账号。"
          : "阅读偏好暂时仅在本次浏览中保留。"}
      </p>
      {profile ? (
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
      <dialog
        ref={editor}
        className="profile-dialog"
        aria-labelledby="profile-edit-title"
        onCancel={(event) => {
          if (editing) event.preventDefault();
        }}
      >
        <div className="rate-heading">
          <h2 id="profile-edit-title">个人资料与学习日常</h2>
          <button
            type="button"
            className="icon-button"
            disabled={editing}
            aria-label="关闭个人资料"
            onClick={() => editor.current?.close()}
          >
            <Icon name="close" />
          </button>
        </div>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            void save();
          }}
        >
          <label className="profile-field">
            怎么称呼你
            <input
              required
              maxLength={80}
              autoComplete="nickname"
              value={draftName}
              disabled={editing}
              onChange={(event) => setDraftName(event.target.value)}
            />
          </label>
          <div className="setting-row">
            <span>学习时区</span>
            <ChoiceDialog
              title="学习时区"
              choices={zones}
              value={zone}
              onChange={setZone}
              searchable
              disabled={editing}
            />
          </div>
          <button
            className="text-button device-zone"
            type="button"
            disabled={editing}
            onClick={() =>
              setZone(Intl.DateTimeFormat().resolvedOptions().timeZone)
            }
          >
            使用设备时区
          </button>
          <div className="setting-row">
            <span>每周学习</span>
            <ChoiceDialog
              title="每周学习天数"
              choices={[3, 5, 7].map((value) => ({
                value: String(value),
                label: `${value} 天`,
              }))}
              value={String(days)}
              onChange={(value) => setDays(Number(value))}
              disabled={editing}
            />
          </div>
          <div className="setting-row">
            <span>每天学习</span>
            <ChoiceDialog
              title="每天学习时间"
              choices={[5, 10, 15].map((value) => ({
                value: String(value),
                label: `${value} 分钟`,
              }))}
              value={String(minutes)}
              onChange={(value) => setMinutes(Number(value))}
              disabled={editing}
            />
          </div>
          {learning.saveError && (
            <p className="error-message" role="alert">
              {learning.saveError}
            </p>
          )}
          <button className="primary" disabled={editing}>
            {editing ? "正在保存" : "保存"}
            <Icon name="check" />
          </button>
        </form>
      </dialog>
    </section>
  );
}
