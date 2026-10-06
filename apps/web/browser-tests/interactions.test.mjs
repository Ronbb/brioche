import { before, after, test } from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { randomUUID } from "node:crypto";
import { createServer } from "vite";
import tailwindcss from "@tailwindcss/vite";

const execute = promisify(execFile);
const cli = fileURLToPath(
  new URL(
    "../../../node_modules/agent-browser/bin/agent-browser.js",
    import.meta.url,
  ),
);
const session = "brioche-regression-" + randomUUID();
let server, origin;
async function browser(...args) {
  const { stdout } = await execute(
    process.execPath,
    [cli, "--session", session, "--json", ...args],
    { timeout: 30000, maxBuffer: 2 * 1024 * 1024 },
  );
  const result = JSON.parse(stdout);
  assert.equal(result.success, true, result.error ?? "browser command failed");
  return result.data;
}
async function evaluate(code) {
  return (await browser("eval", code)).result;
}
const press = (key) => browser("press", key);
async function open(kind = "start") {
  await browser("open", origin + "/?case=" + kind);
  await browser(
    "wait",
    "--fn",
    "!!window.qa?.ready && !!document.querySelector('main')",
  );
}
before(async () => {
  server = await createServer({
    configFile: false,
    root: fileURLToPath(new URL(".", import.meta.url)),
    plugins: [tailwindcss()],
    logLevel: "error",
    server: { host: "127.0.0.1", port: 0 },
  });
  await server.listen();
  origin = "http://127.0.0.1:" + server.httpServer.address().port;
});
after(async () => {
  try {
    await browser("close");
  } finally {
    await server?.close();
  }
});

test("learning entry keeps focus, deduplicates keyboard submits, and returns through login", async () => {
  await open();
  await browser("focus", ".start-learning button");
  await press("Enter");
  await press("Enter");
  await press("Enter");
  assert.deepEqual(
    await evaluate(
      "({focus:document.activeElement.tagName,count:qa.writes.length,busy:document.activeElement.getAttribute('aria-busy')})",
    ),
    { focus: "BUTTON", count: 1, busy: "true" },
  );
  await evaluate("qa.release[0](401)");
  await browser("wait", ".start-learning a");
  await press("Tab");
  await press("Enter");
  assert.deepEqual(await evaluate("({route:qa.route,search:qa.search})"), {
    route: "/login",
    search: "?next=%2Flessons%2Fcourse-a",
  });
});

test("a replaced lesson ignores its late response while current retries keep their exact body", async () => {
  await open();
  await browser("focus", ".start-learning button");
  await press("Enter");
  await press("Tab");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('h1').textContent==='course-b'",
  );
  await browser("focus", ".start-learning button");
  await press("Enter");
  await evaluate("qa.release[0](200)");
  assert.deepEqual(
    await evaluate(
      "({route:qa.route,pending:document.querySelector('.primary').getAttribute('aria-busy'),lessons:qa.writes.map(w=>w.lessonId),separate:qa.writes[0].idempotencyKey!==qa.writes[1].idempotencyKey})",
    ),
    {
      route: "/",
      pending: "true",
      lessons: ["course-a", "course-b"],
      separate: true,
    },
  );
  await evaluate("qa.release[1](500)");
  await browser("wait", "[role=alert]");
  await press("Enter");
  assert.equal(
    await evaluate(
      "JSON.stringify(qa.writes[1])===JSON.stringify(qa.writes[2])",
    ),
    true,
  );
  await evaluate("qa.release[2](200)");
  await browser("wait", "--fn", "qa.route==='/learning/session-course-b'");
});

test("multiple reading bodies support keyboard scrolling, independent translations, and playback cancellation", async () => {
  await open("reading");
  await browser("set", "viewport", "320", "740");
  assert.equal(
    await evaluate("document.querySelectorAll('[role=tab]').length"),
    3,
  );
  await browser("focus", ".speaker");
  await press("Enter");
  await browser("wait", "--fn", "qa.playback==='playing'");
  assert.deepEqual(
    await evaluate(
      "({translation:document.querySelector('.translation')?.textContent ?? null,selected:document.querySelector('[aria-selected=true]').textContent.trim(),spoken:qa.spoken.at(-1)})",
    ),
    { translation: "早上好！", selected: "早晨的问候", spoken: "Bonjour !" },
  );
  await browser("focus", "[role=tab]");
  await press("End");
  assert.deepEqual(
    await evaluate(
      "({first:document.querySelector('.sentence').textContent,playback:qa.playback,translation:!!document.querySelector('.translation'),visible:document.activeElement.getBoundingClientRect().right<=document.querySelector('.reading-tabs').getBoundingClientRect().right+1})",
    ),
    { first: "Bonsoir !", playback: "idle", translation: false, visible: true },
  );
  await browser("focus", ".speaker");
  await press("Enter");
  await browser("wait", "--fn", "qa.playback==='playing'");
  assert.equal(await evaluate("qa.spoken.at(-1)"), "Bonsoir !");
  await browser("focus", "[aria-selected=true]");
  await press("ArrowLeft");
  assert.deepEqual(
    await evaluate(
      "({first:document.querySelector('.sentence').textContent,avatars:document.querySelectorAll('.speaker').length,playback:qa.playback})",
    ),
    { first: "Camille va à la boulangerie.", avatars: 0, playback: "idle" },
  );
  await press("Home");
  assert.deepEqual(
    await evaluate(
      "({translation:document.querySelector('.translation')?.textContent ?? null,selected:document.querySelector('[aria-selected=true]').textContent.trim(),focus:document.activeElement.getAttribute('role')})",
    ),
    { translation: "早上好！", selected: "早晨的问候", focus: "tab" },
  );
  for (const width of [320, 390, 900]) {
    await browser("set", "viewport", String(width), "844");
    assert.equal(
      await evaluate(
        "document.querySelector('.reading-tabs').getBoundingClientRect().right<=innerWidth",
      ),
      true,
    );
  }
});
