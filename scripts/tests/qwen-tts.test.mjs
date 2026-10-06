import test from "node:test";
import assert from "node:assert/strict";
import { audioUrl, requestFor, run, sample, synthesize } from "../qwen-tts.mjs";
const env = {
  DASHSCOPE_API_KEY: "private-test-key",
  QWEN_WORKSPACE_ID: "test-space",
};
const wav = Buffer.alloc(44);
wav.write("RIFF");
wav.write("WAVE", 8);
const success = () =>
  Response.json({
    output: {
      finish_reason: "stop",
      audio: {
        url: "http://dashscope-result-bj.oss-cn-beijing.aliyuncs.com/test.wav?Signature=private",
      },
    },
  });

test("French role voices, emotion, and AI provenance use the documented new API", () => {
  const a = requestFor(sample[0], env);
  const b = requestFor(sample[1], env);
  assert.match(
    a.endpoint,
    /^https:\/\/test-space\.cn-beijing\.maas\.aliyuncs\.com\//,
  );
  assert.deepEqual(a.body.input.language_hints, ["fr"]);
  assert.notEqual(a.body.input.voice, b.body.input.voice);
  assert.match(a.body.input.instruction, /Friendly greeting/);
  assert.equal(a.body.input.enable_aigc_tag, true);
  assert.equal(a.body.input.instructions, undefined);
});
test("plan is offline; generation fails before I/O without credentials", async () => {
  assert.match(await run(["--plan"], {}), /未请求 API/);
  await assert.rejects(run(["--generate"], {}), /DASHSCOPE_API_KEY/);
  assert.throws(() =>
    requestFor(sample[0], { ...env, QWEN_WORKSPACE_ID: "../evil" }),
  );
});
test("signed result URLs are upgraded to TLS and cannot redirect secrets elsewhere", () => {
  assert.equal(
    audioUrl("http://dashscope-result-bj.oss-cn-beijing.aliyuncs.com/a")
      .protocol,
    "https:",
  );
  for (const url of [
    "http://127.0.0.1/a",
    "https://dashscope-result-bj.oss-cn-beijing.aliyuncs.com.evil/a",
    "https://user:pass@dashscope-result-bj.oss-cn-beijing.aliyuncs.com/a",
  ])
    assert.throws(() => audioUrl(url));
});
test("API auth is not sent to audio bucket; receipts contain neither key nor signed URL", async () => {
  const calls = [];
  const result = await synthesize(sample[0], env, async (url, init) => {
    calls.push({ url: String(url), init });
    return calls.length === 1 ? success() : new Response(wav);
  });
  assert.equal(calls.length, 2);
  assert.equal(calls[0].init.headers.Authorization, "Bearer private-test-key");
  assert.equal(calls[1].init.headers, undefined);
  assert.equal(calls[1].init.redirect, "error");
  assert.doesNotMatch(JSON.stringify(result), /private-test-key|Signature/);
});
test("paid POST is never retried; raw provider failures are not printed", async () => {
  let calls = 0;
  await assert.rejects(
    synthesize(sample[0], env, async () => {
      calls++;
      throw new Error("private-test-key");
    }),
    /未自动重试/,
  );
  assert.equal(calls, 1);
  await assert.rejects(
    synthesize(
      sample[0],
      env,
      async () => new Response("private-test-key", { status: 401 }),
    ),
    /HTTP 401/,
  );
});
test("oversized API bodies and invalid audio are rejected", async () => {
  await assert.rejects(
    synthesize(sample[0], env, async () => new Response("x".repeat(256001))),
    /限额/,
  );
  let calls = 0;
  await assert.rejects(
    synthesize(sample[0], env, async () =>
      ++calls === 1 ? success() : new Response("not wav"),
    ),
    /不是 WAV/,
  );
});
