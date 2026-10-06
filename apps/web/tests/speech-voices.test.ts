import test from "node:test";
import assert from "node:assert/strict";
import { withFrenchVoice } from "../app/lib/speech-voices.ts";
const voice = (lang: string) => ({ lang }) as SpeechSynthesisVoice;
function harness() {
  const listeners = new Set<() => void>();
  let voices: SpeechSynthesisVoice[] = [],
    expired = () => {},
    canceled = 0;
  const source = {
    getVoices: () => voices,
    addEventListener: (_: string, callback: () => void) =>
      listeners.add(callback),
    removeEventListener: (_: string, callback: () => void) =>
      listeners.delete(callback),
  };
  const after = (callback: () => void) => {
    expired = callback;
    return () => canceled++;
  };
  return {
    source,
    after,
    listeners,
    set: (value: SpeechSynthesisVoice[]) => (voices = value),
    emit: () => [...listeners].forEach((callback) => callback()),
    expire: () => expired(),
    canceled: () => canceled,
  };
}
test("ready French voices preserve synchronous gesture and prefer the exact locale", () => {
  const h = harness(),
    exact = voice("FR-fr");
  h.set([voice("fr-CA"), exact]);
  let result: SpeechSynthesisVoice | null | undefined;
  withFrenchVoice(
    h.source,
    "fr-FR",
    new AbortController().signal,
    (value) => (result = value),
    () => {
      throw Error("must not wait");
    },
  );
  assert.equal(result, exact);
  assert.equal(h.listeners.size, 0);
});
test("delayed French list resolves once and removes event and deadline", () => {
  const h = harness(),
    results: (SpeechSynthesisVoice | null)[] = [],
    selected = voice("fr-CA");
  withFrenchVoice(
    h.source,
    "fr-FR",
    new AbortController().signal,
    (value) => results.push(value),
    h.after,
  );
  h.set([voice("en-US")]);
  h.emit();
  assert.equal(results.length, 0);
  h.set([selected]);
  h.emit();
  h.emit();
  h.expire();
  assert.deepEqual(results, [selected]);
  assert.equal(h.listeners.size, 0);
  assert.equal(h.canceled(), 1);
});
test("canceling an old request cannot speak or fail after a newer request", () => {
  const h = harness(),
    old = new AbortController(),
    results: string[] = [];
  withFrenchVoice(
    h.source,
    "fr-FR",
    old.signal,
    () => results.push("old"),
    h.after,
  );
  const stale = [...h.listeners][0];
  old.abort();
  assert.equal(h.listeners.size, 0);
  assert.equal(h.canceled(), 1);
  withFrenchVoice(
    h.source,
    "fr-FR",
    new AbortController().signal,
    () => results.push("new"),
    h.after,
  );
  h.set([voice("fr-FR")]);
  stale();
  h.emit();
  h.expire();
  assert.deepEqual(results, ["new"]);
  assert.equal(h.listeners.size, 0);
});
test("unavailable French voice times out once and ignores later voices", () => {
  const h = harness(),
    results: (SpeechSynthesisVoice | null)[] = [];
  h.set([voice("en-US"), voice("fry-NL")]);
  withFrenchVoice(
    h.source,
    "fr-FR",
    new AbortController().signal,
    (value) => results.push(value),
    h.after,
  );
  h.expire();
  h.set([voice("fr-FR")]);
  h.emit();
  h.expire();
  assert.deepEqual(results, [null]);
  assert.equal(h.listeners.size, 0);
  assert.equal(h.canceled(), 1);
});
test("already canceled requests do not query voices or install listeners", () => {
  const h = harness(),
    controller = new AbortController();
  controller.abort();
  h.source.getVoices = () => {
    throw Error("must not query");
  };
  withFrenchVoice(
    h.source,
    "fr-FR",
    controller.signal,
    () => {
      throw Error("must not callback");
    },
    h.after,
  );
  assert.equal(h.listeners.size, 0);
  assert.equal(h.canceled(), 0);
});
test("a list appearing during registration is discovered without another event", () => {
  const h = harness(),
    selected = voice("fr-FR");
  let reads = 0,
    result;
  h.source.getVoices = () => (++reads === 1 ? [] : [selected]);
  withFrenchVoice(
    h.source,
    "fr-FR",
    new AbortController().signal,
    (value) => (result = value),
    h.after,
  );
  assert.equal(result, selected);
  assert.equal(h.listeners.size, 0);
  assert.equal(h.canceled(), 1);
});
