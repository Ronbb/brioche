import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useLocation } from "react-router";
import { createPortal } from "react-dom";
import { Icon } from "./icon";
import type { UserProfile } from "@brioche/contracts/UserProfile";
import type { UpdateProfileRequest } from "@brioche/contracts/UpdateProfileRequest";
import { ApiRequestError, privateRequest } from "../lib/api.client";
export type ProfileChanges = Partial<Omit<UpdateProfileRequest, "version">>;
type SpeechUnit = { id: string; text: string; locale?: string };
type PlayerState = {
  status: "idle" | "playing" | "paused";
  id: string | null;
  progress: number;
};
type Learning = {
  profile: UserProfile | null;
  saveProfile: (changes: ProfileChanges) => Promise<boolean>;
  saveStatus: "idle" | "saving" | "saved" | "error";
  saveError: string;
  translation: boolean;
  setTranslation: (v: boolean) => void;
  rate: number;
  setRate: (v: number) => void;
  openRate: () => void;
  play: (units: SpeechUnit[]) => void;
  toggle: (units: SpeechUnit[]) => void;
  stop: () => void;
  player: PlayerState;
  toast: (message: string) => void;
};
const Context = createContext<Learning | null>(null);
export function useLearning() {
  const value = useContext(Context);
  if (!value) throw Error("LearningProvider required");
  return value;
}
export function LearningProvider({
  children,
  user = null,
}: {
  children: ReactNode;
  user?: UserProfile | null;
}) {
  const [profile, setProfile] = useState<UserProfile | null>(user),
    [saveStatus, setSaveStatus] = useState<Learning["saveStatus"]>("idle"),
    [saveError, setSaveError] = useState(""),
    [translation, setTranslationState] = useState(
      user?.settings.showTranslation ?? false,
    ),
    [rate, setRateState] = useState(user?.settings.speechRate ?? 1),
    [player, setPlayer] = useState<PlayerState>({
      status: "idle",
      id: null,
      progress: 0,
    }),
    [message, setMessage] = useState(""),
    [messageSequence, setMessageSequence] = useState(0),
    [toastHost, setToastHost] = useState<HTMLDialogElement | null>(null);
  function notify(value: string) {
    setMessage(value);
    setMessageSequence((sequence) => sequence + 1);
  }
  const savedProfile = useRef(user),
    pendingChanges = useRef(new Map<symbol, ProfileChanges>()),
    saves = useRef<Promise<boolean>>(Promise.resolve(true)),
    saveGeneration = useRef(0),
    saveCount = useRef(0);
  const rateRef = useRef(user?.settings.speechRate ?? 1),
    state = useRef(player),
    queue = useRef<SpeechUnit[]>([]),
    index = useRef(0),
    generation = useRef(0),
    dialog = useRef<HTMLDialogElement>(null);
  const location = useLocation();
  const restartPaused = useRef(false);
  function acceptProfile(value: UserProfile | null) {
    savedProfile.current = value;
    setProfile(value);
    let show = value?.settings.showTranslation ?? false,
      speed = value?.settings.speechRate ?? 1;
    for (const changes of pendingChanges.current.values()) {
      if (changes.showTranslation != null) show = changes.showTranslation;
      if (changes.speechRate != null) speed = changes.speechRate;
    }
    setTranslationState(show);
    applyRate(speed);
  }
  useEffect(() => {
    if (
      savedProfile.current?.id === user?.id &&
      (savedProfile.current?.version ?? 0) > (user?.version ?? 0)
    )
      return;
    saveGeneration.current++;
    pendingChanges.current.clear();
    setSaveStatus("idle");
    setSaveError("");
    acceptProfile(user);
  }, [user?.id, user?.version]);
  function saveProfile(changes: ProfileChanges): Promise<boolean> {
    if (!savedProfile.current) return Promise.resolve(false);
    const gen = saveGeneration.current;
    const job = Symbol();
    pendingChanges.current.set(job, changes);
    saveCount.current++;
    setSaveStatus("saving");
    setSaveError("");
    const next = saves.current.then(async () => {
      if (gen !== saveGeneration.current || !savedProfile.current) return false;
      try {
        const value = await privateRequest<UserProfile>(
          "/api/v1/me/settings",
          "PATCH",
          { ...changes, version: savedProfile.current.version },
        );
        if (gen !== saveGeneration.current) return false;
        pendingChanges.current.delete(job);
        acceptProfile(value);
        return true;
      } catch (error) {
        if (gen !== saveGeneration.current) return false;
        saveGeneration.current++;
        pendingChanges.current.clear();
        if (error instanceof ApiRequestError && error.status === 401)
          acceptProfile(null);
        else {
          // A timed-out response may already have saved. Read the current state; never resend a write automatically.
          try {
            acceptProfile(
              await privateRequest<UserProfile>("/api/v1/me", "GET"),
            );
          } catch {
            acceptProfile(savedProfile.current);
          }
        }
        setSaveStatus("error");
        const message =
          error instanceof ApiRequestError
            ? error.message
            : "保存未确认，请检查当前设置后重试。";
        setSaveError(message);
        notify(message);
        return false;
      }
    });
    saves.current = next;
    void next.then((ok) => {
      pendingChanges.current.delete(job);
      saveCount.current--;
      if (!saveCount.current && ok) setSaveStatus("saved");
    });
    return next;
  }
  function setTranslation(value: boolean) {
    setTranslationState(value);
    if (savedProfile.current) void saveProfile({ showTranslation: value });
  }
  function update(value: PlayerState) {
    state.current = value;
    setPlayer(value);
  }
  function stop() {
    restartPaused.current = false;
    generation.current++;
    if (typeof window !== "undefined") window.speechSynthesis?.cancel();
    queue.current = [];
    update({ status: "idle", id: null, progress: 0 });
  }
  function speakCurrent() {
    const unit = queue.current[index.current];
    if (!unit) {
      update({ status: "idle", id: null, progress: 1 });
      return;
    }
    const gen = generation.current,
      voices = window.speechSynthesis.getVoices(),
      voice =
        voices.find((v) => v.lang === (unit.locale ?? "fr-FR")) ??
        voices.find((v) => v.lang.startsWith("fr"));
    if (!voice) {
      stop();
      notify("当前浏览器没有可用的法语语音");
      return;
    }
    const utterance = new SpeechSynthesisUtterance(unit.text);
    utterance.lang = unit.locale ?? "fr-FR";
    utterance.voice = voice;
    utterance.rate = rateRef.current;
    utterance.onstart = () => {
      if (gen === generation.current)
        update({
          status: "playing",
          id: unit.id,
          progress: index.current / queue.current.length,
        });
    };
    utterance.onboundary = (event) => {
      if (gen === generation.current)
        update({
          status: "playing",
          id: unit.id,
          progress:
            (index.current + event.charIndex / Math.max(1, unit.text.length)) /
            queue.current.length,
        });
    };
    utterance.onend = () => {
      if (gen === generation.current) {
        index.current++;
        speakCurrent();
      }
    };
    utterance.onerror = (event) => {
      if (
        gen === generation.current &&
        event.error !== "canceled" &&
        event.error !== "interrupted"
      ) {
        stop();
        notify("朗读暂时无法播放，请重试。");
      }
    };
    window.speechSynthesis.speak(utterance);
  }
  function play(units: SpeechUnit[]) {
    stop();
    if (!window.speechSynthesis || !window.SpeechSynthesisUtterance) {
      notify("当前浏览器不支持语音朗读");
      return;
    }
    queue.current = units;
    index.current = 0;
    speakCurrent();
  }
  function toggle(units: SpeechUnit[]) {
    if (
      state.current.id &&
      !units.some((unit) => unit.id === state.current.id)
    ) {
      play(units);
      return;
    }
    if (state.current.status === "playing") {
      window.speechSynthesis.pause();
      update({ ...state.current, status: "paused" });
    } else if (state.current.status === "paused") {
      if (restartPaused.current) {
        restartPaused.current = false;
        speakCurrent();
      } else {
        window.speechSynthesis.resume();
        update({ ...state.current, status: "playing" });
      }
    } else play(units);
  }
  function applyRate(value: number) {
    if (rateRef.current === value) return;
    rateRef.current = value;
    setRateState(value);
    if (state.current.status === "playing") {
      generation.current++;
      window.speechSynthesis.cancel();
      speakCurrent();
    } else if (state.current.status === "paused") {
      generation.current++;
      window.speechSynthesis.cancel();
      restartPaused.current = true;
    }
  }
  function setRate(value: number) {
    applyRate(value);
    if (savedProfile.current) void saveProfile({ speechRate: value });
  }
  useEffect(() => {
    stop();
    dialog.current?.close();
    setMessage("");
    setToastHost(null);
  }, [location.pathname]);
  useEffect(
    () => () => {
      generation.current++;
      window.speechSynthesis?.cancel();
    },
    [],
  );
  useEffect(() => {
    if (!message) {
      setToastHost(null);
      return;
    }
    const updateHost = () =>
      setToastHost(
        Array.from(
          document.querySelectorAll<HTMLDialogElement>("dialog[open]"),
        ).at(-1) ?? null,
      );
    updateHost();
    const observer = new MutationObserver(updateHost);
    observer.observe(document.body, {
      subtree: true,
      attributes: true,
      attributeFilter: ["open"],
    });
    const timer = setTimeout(() => setMessage(""), 5500);
    return () => {
      clearTimeout(timer);
      observer.disconnect();
    };
  }, [message, messageSequence]);
  const toast = (
    <div className="toast" hidden={!message}>
      <span role="status" aria-live="polite">
        {message}
      </span>
      <button
        className="toast-close"
        aria-label="关闭提示"
        onClick={() => setMessage("")}
      >
        <Icon name="close" />
      </button>
    </div>
  );
  return (
    <Context.Provider
      value={{
        profile,
        saveProfile,
        saveStatus,
        saveError,
        translation,
        setTranslation,
        rate,
        setRate,
        openRate: () => {
          dialog.current?.showModal();
          dialog.current
            ?.querySelector<HTMLButtonElement>('[aria-checked="true"]')
            ?.focus();
        },
        play,
        toggle,
        stop,
        player,
        toast: notify,
      }}
    >
      {children}
      <dialog
        ref={dialog}
        className="rate-dialog"
        aria-labelledby="speed-title"
        onClick={(e) => {
          if (e.target === dialog.current) {
            const r = dialog.current.getBoundingClientRect();
            if (
              e.clientX < r.left ||
              e.clientX > r.right ||
              e.clientY < r.top ||
              e.clientY > r.bottom
            )
              dialog.current.close();
          }
        }}
      >
        <div className="rate-heading">
          <h2 id="speed-title">朗读速度</h2>
          <button
            className="icon-button"
            aria-label="关闭速度设置"
            onClick={() => dialog.current?.close()}
          >
            <Icon name="close" />
          </button>
        </div>
        <div className="rate-options" role="radiogroup" aria-label="朗读速度">
          {[0.75, 1, 1.25, 1.5].map((value) => (
            <button
              key={value}
              className="rate-option"
              role="radio"
              aria-checked={rate === value}
              tabIndex={rate === value ? 0 : -1}
              onKeyDown={(event) => {
                const values = [0.75, 1, 1.25, 1.5];
                let next: number | undefined;
                if (event.key === "ArrowDown" || event.key === "ArrowRight")
                  next = values[(values.indexOf(value) + 1) % values.length];
                if (event.key === "ArrowUp" || event.key === "ArrowLeft")
                  next =
                    values[
                      (values.indexOf(value) + values.length - 1) %
                        values.length
                    ];
                if (event.key === "Home") next = values[0];
                if (event.key === "End") next = values.at(-1);
                if (next !== undefined) {
                  event.preventDefault();
                  setRate(next);
                  const buttons =
                    dialog.current?.querySelectorAll<HTMLButtonElement>(
                      '[role="radio"]',
                    );
                  buttons?.[values.indexOf(next)]?.focus();
                }
              }}
              onClick={() => {
                setRate(value);
                dialog.current?.close();
              }}
            >
              {value}×<Icon name="check" />
            </button>
          ))}
        </div>
      </dialog>
      {toastHost ? createPortal(toast, toastHost) : toast}
    </Context.Provider>
  );
}
export function Player({ units }: { units: SpeechUnit[] }) {
  const learning = useLearning(),
    hold = useRef<ReturnType<typeof setTimeout> | null>(null),
    long = useRef(false),
    start = useRef({ x: 0, y: 0 });
  const ownsPlayback = units.some((unit) => unit.id === learning.player.id);
  const progress = ownsPlayback ? learning.player.progress : 0;
  const playing = ownsPlayback && learning.player.status === "playing";
  const cancel = () => {
    if (hold.current) clearTimeout(hold.current);
    hold.current = null;
  };
  useEffect(() => cancel, []);
  return (
    <div className="reader-player">
      <button
        className="playback-line"
        aria-label={playing ? "暂停朗读" : "播放全文"}
        onPointerDown={(e) => {
          long.current = false;
          start.current = { x: e.clientX, y: e.clientY };
          cancel();
          hold.current = setTimeout(() => {
            long.current = true;
            learning.openRate();
          }, 550);
        }}
        onPointerMove={(e) => {
          if (
            Math.hypot(
              e.clientX - start.current.x,
              e.clientY - start.current.y,
            ) > 10
          )
            cancel();
        }}
        onPointerUp={cancel}
        onPointerCancel={cancel}
        onPointerLeave={cancel}
        onContextMenu={(e) => {
          e.preventDefault();
          cancel();
          long.current = true;
          learning.openRate();
        }}
        onKeyDown={(e) => {
          if (e.shiftKey && e.key === "F10") {
            e.preventDefault();
            learning.openRate();
          }
        }}
        onClick={() => {
          cancel();
          if (long.current) {
            long.current = false;
            return;
          }
          learning.toggle(units);
        }}
      >
        <span className="playback-track">
          <span
            style={{
              display: "block",
              height: 2,
              background: "var(--accent)",
              width: progress * 100 + "%",
            }}
          />
          <span
            className="play-marker"
            style={{
              left: `${Math.max(2, Math.min(98, progress ? progress * 100 : 50))}%`,
            }}
          >
            <Icon name={playing ? "pause" : "play"} />
          </span>
        </span>
      </button>
    </div>
  );
}
