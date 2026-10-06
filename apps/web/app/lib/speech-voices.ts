type VoiceSource = Pick<
  SpeechSynthesis,
  "getVoices" | "addEventListener" | "removeEventListener"
>;
type Delay = (callback: () => void) => () => void;
const deadline: Delay = (callback) => {
  const timer = setTimeout(callback, 4000);
  return () => clearTimeout(timer);
};

/** Immediate voices keep the user's click synchronous; delayed lists are cancellable. */
export function withFrenchVoice(
  source: VoiceSource,
  locale: string,
  signal: AbortSignal,
  ready: (voice: SpeechSynthesisVoice | null) => void,
  after: Delay = deadline,
) {
  if (signal.aborted) return;
  const choose = () => {
    const voices = source.getVoices();
    return (
      voices.find(
        (voice) => voice.lang.toLowerCase() === locale.toLowerCase(),
      ) ??
      voices.find((voice) => /^fr(?:-|$)/i.test(voice.lang)) ??
      null
    );
  };
  const immediate = choose();
  if (immediate) {
    ready(immediate);
    return;
  }
  let active = true;
  let cancelDeadline = () => {};
  const dispose = () => {
    if (!active) return;
    active = false;
    cancelDeadline();
    source.removeEventListener("voiceschanged", changed);
    signal.removeEventListener("abort", dispose);
  };
  const finish = (voice: SpeechSynthesisVoice | null) => {
    if (!active) return;
    dispose();
    ready(voice);
  };
  const changed = () => {
    const voice = choose();
    if (voice) finish(voice);
  };
  source.addEventListener("voiceschanged", changed);
  signal.addEventListener("abort", dispose, { once: true });
  cancelDeadline = after(() => finish(choose()));
  // Catch a list that became available between the first read and registration.
  changed();
}
