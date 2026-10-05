import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useLocation } from "react-router";
import { Icon } from "./icon";
type SpeechUnit = { id: string; text: string; locale?: string };
type PlayerState = {
  status: "idle" | "playing" | "paused";
  id: string | null;
  progress: number;
};
type Learning = {
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
export function LearningProvider({ children }: { children: ReactNode }) {
  const [translation, setTranslation] = useState(false),
    [rate, setRateState] = useState(1),
    [player, setPlayer] = useState<PlayerState>({
      status: "idle",
      id: null,
      progress: 0,
    }),
    [message, setMessage] = useState("");
  const rateRef = useRef(1),
    state = useRef(player),
    queue = useRef<SpeechUnit[]>([]),
    index = useRef(0),
    generation = useRef(0),
    dialog = useRef<HTMLDialogElement>(null);
  const location = useLocation();
  const restartPaused = useRef(false);
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
      setMessage("当前浏览器没有可用的法语语音");
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
        setMessage("朗读暂时无法播放，请重试。");
      }
    };
    window.speechSynthesis.speak(utterance);
  }
  function play(units: SpeechUnit[]) {
    stop();
    if (!window.speechSynthesis || !window.SpeechSynthesisUtterance) {
      setMessage("当前浏览器不支持语音朗读");
      return;
    }
    queue.current = units;
    index.current = 0;
    speakCurrent();
  }
  function toggle(units: SpeechUnit[]) {
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
  function setRate(value: number) {
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
  useEffect(() => {
    stop();
    dialog.current?.close();
  }, [location.pathname]);
  useEffect(
    () => () => {
      generation.current++;
      window.speechSynthesis?.cancel();
    },
    [],
  );
  useEffect(() => {
    if (!message) return;
    const timer = setTimeout(() => setMessage(""), 5500);
    return () => clearTimeout(timer);
  }, [message]);
  return (
    <Context.Provider
      value={{
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
        toast: setMessage,
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
    </Context.Provider>
  );
}
export function Player({ units }: { units: SpeechUnit[] }) {
  const learning = useLearning(),
    hold = useRef<ReturnType<typeof setTimeout> | null>(null),
    long = useRef(false),
    start = useRef({ x: 0, y: 0 });
  const cancel = () => {
    if (hold.current) clearTimeout(hold.current);
    hold.current = null;
  };
  useEffect(() => cancel, []);
  return (
    <div className="reader-player">
      <button
        className="playback-line"
        aria-label={
          learning.player.status === "playing" ? "暂停朗读" : "播放全文"
        }
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
              width: learning.player.progress * 100 + "%",
            }}
          />
          <span
            className="play-marker"
            style={{
              left: `${Math.max(2, Math.min(98, learning.player.progress ? learning.player.progress * 100 : 50))}%`,
            }}
          >
            <Icon
              name={learning.player.status === "playing" ? "pause" : "play"}
            />
          </span>
        </span>
      </button>
    </div>
  );
}
