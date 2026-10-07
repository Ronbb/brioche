import { useEffect, useRef, useState } from "react";
import type { AdminAlignment } from "@brioche/contracts/AdminAlignment";
import type { AdminSpeechPackageRequest } from "@brioche/contracts/AdminSpeechPackageRequest";
import { adminArchive } from "../lib/admin.client";

export function SpeechPackage({
  alignment,
  lessonRevision,
  disabled,
  onPending,
}: {
  alignment: AdminAlignment;
  lessonRevision: number;
  disabled: boolean;
  onPending: (value: boolean) => void;
}) {
  const [revision, setRevision] = useState(String(lessonRevision + 1));
  const [gap, setGap] = useState("250");
  const [source, setSource] = useState("Qwen 法语语音合成（固定角色声音版本）");
  const [license, setLicense] = useState("");
  const [creator, setCreator] = useState("");
  const [credit, setCredit] = useState("AI 合成语音 · Brioche");
  const [reason, setReason] = useState("");
  const [rights, setRights] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const busy = useRef(false);
  const request = useRef<AbortController | null>(null);
  const download = useRef<string | null>(null);
  const ready =
    alignment.clips.length > 0 &&
    alignment.clips.every((clip) => clip.accepted === true);
  useEffect(
    () => () => {
      request.current?.abort();
      if (download.current) URL.revokeObjectURL(download.current);
    },
    [],
  );
  async function assemble() {
    if (busy.current || disabled || !ready) return;
    setError("");
    setNotice("");
    const version = Number(revision),
      gapMs = Number(gap);
    if (
      !Number.isInteger(version) ||
      version <= lessonRevision ||
      version > 2147483647 ||
      !Number.isInteger(gapMs) ||
      gapMs < 0 ||
      gapMs > 1000 ||
      !rights ||
      [source, license, creator, credit, reason].some((value) => !value.trim())
    ) {
      setError("请核对新版本、停顿、来源授权及填写的信息。");
      return;
    }
    const body: AdminSpeechPackageRequest = {
      expectedReportHash: alignment.reportHash,
      lessonRevision: version,
      gapMs,
      rightsConfirmed: rights,
      source,
      license,
      creator,
      creditZh: credit,
      reason,
    };
    busy.current = true;
    setPending(true);
    onPending(true);
    const controller = new AbortController();
    request.current = controller;
    try {
      const blob = await adminArchive(
        `speech-alignments/${alignment.id}/package`,
        body,
        controller.signal,
      );
      controller.signal.throwIfAborted();
      if (download.current) URL.revokeObjectURL(download.current);
      const url = URL.createObjectURL(blob);
      download.current = url;
      const link = document.createElement("a");
      link.href = url;
      link.download = `speech-package-${alignment.id}-v${version}.tar`;
      document.body.append(link);
      link.click();
      link.remove();
      setNotice(
        "录音课包已下载。登记录音、导入新草稿并完成最终试听后，再审批发布。",
      );
    } catch (error) {
      if (!controller.signal.aborted)
        setError(
          error instanceof Error ? error.message : "组装未完成，请重新核对。",
        );
    } finally {
      busy.current = false;
      if (!controller.signal.aborted) {
        setPending(false);
        onPending(false);
      }
    }
  }
  return (
    <form
      className="admin-editor"
      onSubmit={(event) => {
        event.preventDefault();
        void assemble();
      }}
    >
      <h2>组装录音课包</h2>
      <p className="muted">
        输出新课程草稿、录音登记清单和完整来源记录。下载不会登记或发布。
      </p>
      {!ready && (
        <p className="muted">全部片段通过审听与时间轴核对后，可以组装。</p>
      )}
      <fieldset disabled={disabled || pending || !ready}>
        <label>
          新课程版本
          <input
            type="number"
            min={lessonRevision + 1}
            max={2147483647}
            step="1"
            required
            value={revision}
            onChange={(e) => setRevision(e.target.value)}
          />
        </label>
        <label>
          句间停顿（毫秒）
          <input
            type="number"
            min="0"
            max="1000"
            step="1"
            required
            value={gap}
            onChange={(e) => setGap(e.target.value)}
          />
        </label>
        <label>
          录音来源
          <input
            required
            maxLength={2000}
            value={source}
            onChange={(e) => setSource(e.target.value)}
          />
        </label>
        <label>
          使用授权依据
          <textarea
            required
            maxLength={2000}
            value={license}
            onChange={(e) => setLicense(e.target.value)}
          />
        </label>
        <label>
          创作或授权主体
          <input
            required
            maxLength={2000}
            value={creator}
            onChange={(e) => setCreator(e.target.value)}
          />
        </label>
        <label>
          公开署名
          <input
            required
            maxLength={500}
            value={credit}
            onChange={(e) => setCredit(e.target.value)}
          />
        </label>
        <label>
          组装说明
          <textarea
            required
            maxLength={2000}
            value={reason}
            onChange={(e) => setReason(e.target.value)}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={rights}
            onChange={(e) => setRights(e.target.checked)}
          />
          我确认录音及角色声音可按以上授权使用。
        </label>
      </fieldset>
      <button
        className="primary"
        disabled={!ready || (disabled && !pending)}
        aria-disabled={pending}
        aria-busy={pending}
      >
        {pending ? "正在组装" : "下载录音课包"}
      </button>
      {error && <p role="alert">{error}</p>}
      {notice && <p role="status">{notice}</p>}
    </form>
  );
}
