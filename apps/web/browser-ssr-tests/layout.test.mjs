import { before, after, test } from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve, sep, extname } from "node:path";
import { fileURLToPath } from "node:url";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { randomUUID } from "node:crypto";
import { createRequestHandler } from "react-router";
import * as build from "../build/server/index.js";

const execute = promisify(execFile);
const cli = fileURLToPath(
  new URL(
    "../../../node_modules/agent-browser/bin/agent-browser.js",
    import.meta.url,
  ),
);
const client = fileURLToPath(new URL("../build/client/", import.meta.url));
const session = "brioche-ssr-layout-" + randomUUID();
const source = JSON.parse(
  await readFile(
    new URL("../../../docs/examples/a1-bakery.lesson.json", import.meta.url),
    "utf8",
  ),
);
// Public projection only: the browser never receives grading rules or editorial data.
const publicFields = [
  "schemaVersion",
  "id",
  "revision",
  "levelId",
  "unitId",
  "title",
  "summaryZh",
  "estimatedMinutes",
  "objectivesZh",
  "knowledge",
  "blocks",
  "steps",
  "completion",
  "reviewItemIds",
  "cast",
];
const lesson = {
  ...Object.fromEntries(publicFields.map((key) => [key, source[key]])),
  media: [],
  audio: [],
  audioTracks: [],
};
const catalog = {
  developmentFixture: true,
  levels: [
    {
      id: "a1",
      label: "A1",
      units: [
        { id: lesson.unitId, titleZh: "早餐与面包店", lessons: [lesson] },
      ],
    },
  ],
};
const handler = createRequestHandler(build, "production");
const originalBase = process.env.INTERNAL_API_URL;
const serverErrors = [];
let origin,
  opened = false;
let accounts = false;
const identityReads = [];
let identityProof = null;
const profile = (id) => ({
  id,
  email: `${id}@example.test`,
  displayName: id === "shell-a" ? "Alice" : "Bob",
  role: "learner",
  version: 1,
  settings: {
    timeZone: "Asia/Shanghai",
    weeklyDays: 3,
    dailyMinutes: 10,
    showTranslation: false,
    speechRate: 1,
  },
});
const api = createServer((request, response) => {
  response.setHeader("Content-Type", "application/json");
  if (request.url === "/api/v1/me") {
    if (accounts) {
      const id = /(?:^|;\s*)brioche\.sid=(shell-[ab])(?:;|$)/.exec(
        request.headers.cookie ?? "",
      )?.[1];
      identityReads.push({
        id: id ?? null,
        channel: request.headers["x-shell-channel"] ?? "ssr",
      });
      if (id) response.end(JSON.stringify(profile(id)));
      else response.writeHead(401).end("{}");
      return;
    }
    response.writeHead(404).end("{}");
    return;
  }
  if (request.url.startsWith("/api/catalog")) {
    response.end(JSON.stringify({ ...catalog, developmentFixture: !accounts }));
    return;
  }
  if (request.url.startsWith("/api/lessons/")) {
    response.end(JSON.stringify(lesson));
    return;
  }
  response.writeHead(404).end("{}");
});
const types = {
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".webmanifest": "application/manifest+json",
};
const web = createServer(async (request, response) => {
  try {
    const url = new URL(request.url, origin);
    if (url.pathname === "/__identity-proof" && request.method === "POST") {
      let body = "";
      for await (const chunk of request) body += chunk;
      identityProof = JSON.parse(body);
      response.writeHead(204).end();
      return;
    }
    if (url.pathname.startsWith("/api/")) {
      const proxied = await fetch(
        process.env.INTERNAL_API_URL + url.pathname + url.search,
        {
          headers: {
            "X-Shell-Channel": "browser",
            ...(request.headers.cookie
              ? { cookie: request.headers.cookie }
              : {}),
          },
        },
      );
      response.writeHead(proxied.status, {
        "Content-Type": "application/json",
        "Cache-Control": "private, no-store",
      });
      response.end(Buffer.from(await proxied.arrayBuffer()));
      return;
    }
    if (
      url.pathname.startsWith("/assets/") ||
      url.pathname.startsWith("/icons/") ||
      ["/apple-touch-icon.png", "/manifest.webmanifest"].includes(url.pathname)
    ) {
      const path = resolve(client, "." + decodeURIComponent(url.pathname));
      if (!path.startsWith(resolve(client) + sep)) {
        response.writeHead(400).end();
        return;
      }
      try {
        const bytes = await readFile(path);
        response.setHeader(
          "Content-Type",
          types[extname(path)] ?? "application/octet-stream",
        );
        response.end(bytes);
      } catch {
        response.writeHead(404).end();
      }
      return;
    }
    const headers = new Headers();
    for (const [key, value] of Object.entries(request.headers)) {
      if (value !== undefined)
        headers.set(key, Array.isArray(value) ? value.join(", ") : value);
    }
    const result = await handler(
      new Request(url, { method: request.method, headers }),
    );
    response.writeHead(result.status, Object.fromEntries(result.headers));
    response.end(Buffer.from(await result.arrayBuffer()));
  } catch (error) {
    serverErrors.push(String(error));
    if (!response.headersSent) response.writeHead(500);
    response.end();
  }
});
async function browser(...args) {
  const { stdout } = await execute(
    process.execPath,
    [cli, "--session", session, "--json", ...args],
    { timeout: 30000, maxBuffer: 2 * 1024 * 1024 },
  );
  const result = JSON.parse(stdout);
  assert.equal(result.success, true, result.error);
  return result.data;
}
const evaluate = async (code) => (await browser("eval", code)).result;
before(async () => {
  await new Promise((resolve) => api.listen(0, "127.0.0.1", resolve));
  process.env.INTERNAL_API_URL = `http://127.0.0.1:${api.address().port}`;
  await new Promise((resolve) => web.listen(0, "127.0.0.1", resolve));
  origin = `http://127.0.0.1:${web.address().port}`;
});
after(async () => {
  try {
    if (opened) await browser("close");
  } finally {
    for (const server of [web, api]) {
      server.closeAllConnections();
      await new Promise((resolve) => server.close(resolve));
    }
    if (originalBase === undefined) delete process.env.INTERNAL_API_URL;
    else process.env.INTERNAL_API_URL = originalBase;
  }
});

test("production SSR hydrates its real shell, preserves mobile widths, routes focus and modal return", async () => {
  const initial = await fetch(origin);
  assert.equal(initial.status, 200);
  assert.match(await initial.text(), /<html[^>]*class="overlay-scroll"/);
  await browser("open", origin);
  opened = true;
  await browser(
    "wait",
    "--fn",
    "!!window.__reactRouterContext && !!document.querySelector('.profile-avatar')",
  );
  await evaluate("window.shellMarker='hydrated-route-test'");
  for (const width of [320, 390, 768, 1440]) {
    await browser("set", "viewport", String(width), "740");
    assert.deepEqual(
      await evaluate(
        "({width:document.documentElement.clientWidth,overflow:document.documentElement.scrollWidth})",
      ),
      { width, overflow: width },
    );
    assert.equal(
      await evaluate(
        "getComputedStyle(document.documentElement).scrollbarWidth",
      ),
      "none",
    );
    assert.equal(
      await evaluate(
        "(()=>{const r=document.querySelector('.topbar').getBoundingClientRect();return r.left>=0&&r.right<=innerWidth})()",
      ),
      true,
    );
  }
  await browser("set", "viewport", "390", "740");
  await browser("focus", ".skip-link");
  await browser("press", "Enter");
  assert.equal(await evaluate("document.activeElement.id"), "page-content");
  await browser("focus", ".profile-avatar");
  await browser("press", "Enter");
  await browser(
    "wait",
    "--fn",
    "location.pathname==='/profile' && document.activeElement.matches('main h1')",
  );
  assert.equal(await evaluate("window.shellMarker"), "hydrated-route-test");
  await browser("focus", "button[aria-label^='朗读速度']");
  await browser("press", "Enter");
  await browser("wait", "dialog[open]");
  assert.equal(
    await evaluate(
      "(()=>{const r=document.querySelector('dialog[open]').getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight})()",
    ),
    true,
  );
  assert.equal(
    await evaluate(
      "document.querySelector('dialog[open]').contains(document.activeElement)",
    ),
    true,
  );
  assert.equal(
    await evaluate(
      "getComputedStyle(document.querySelector('.page-scrollbar')).visibility",
    ),
    "hidden",
  );
  await browser("press", "Escape");
  assert.equal(
    await evaluate(
      "document.activeElement.matches(\"button[aria-label^='朗读速度']\")",
    ),
    true,
  );
  await browser("focus", ".brand");
  await browser("press", "Enter");
  await browser(
    "wait",
    "--fn",
    "location.pathname==='/' && document.activeElement.matches('main h1')",
  );
  await browser("focus", "a[href='/courses']");
  await browser("press", "Enter");
  await browser(
    "wait",
    "--fn",
    "location.pathname==='/courses' && document.activeElement.matches('main h1')",
  );
  assert.equal(await evaluate("window.shellMarker"), "hydrated-route-test");
  assert.deepEqual(serverErrors, []);
  const { errors } = await browser("errors");
  assert.deepEqual(errors, [], "hydration and route errors must fail");
});

test("server-authorized identity replacement discards the old private page before reload warnings", async () => {
  accounts = true;
  identityReads.length = 0;
  identityProof = null;
  try {
    opened = true;
    await browser("open", origin + "/profile");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("open", origin + "/profile");
    opened = true;
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.profile-summary h2')?.textContent==='Alice'",
    );
    await browser("focus", ".profile-edit");
    await browser("press", "Enter");
    await browser("wait", ".profile-dialog[open]");
    await browser("focus", ".profile-dialog input");
    await browser("press", "Control+a");
    await browser("keyboard", "inserttext", "Unsaved Alice");
    await evaluate(
      `window.addEventListener('beforeunload', e => {const proof=JSON.stringify({guarded:e.defaultPrevented,oldProfile:document.querySelector('.profile-summary h2')?.textContent==='Alice',dialogs:document.querySelectorAll('.profile-dialog[open]').length});sessionStorage.setItem('shell-invalidation-proof',proof);navigator.sendBeacon('/__identity-proof',proof)}, {once:true})`,
    );
    await browser("cookies", "set", "brioche.sid", "shell-b");
    assert.equal(await evaluate("document.visibilityState"), "visible");
    // Controlled focus notification exercises the real IdentitySync HTTP read.
    await evaluate("window.dispatchEvent(new Event('focus'))");
    for (let attempt = 0; attempt < 100 && !identityProof; attempt++)
      await new Promise((resolve) => setTimeout(resolve, 20));
    assert.ok(
      identityReads.some(
        (read) => read.id === "shell-b" && read.channel === "browser",
      ),
    );
    assert.deepEqual(identityProof, {
      guarded: false,
      oldProfile: false,
      dialogs: 0,
    });
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.profile-summary h2')?.textContent==='Bob'",
    );
    assert.deepEqual(
      await evaluate(
        "JSON.parse(sessionStorage.getItem('shell-invalidation-proof'))",
      ),
      { guarded: false, oldProfile: false, dialogs: 0 },
    );
    assert.equal(
      await evaluate(
        "document.querySelectorAll('.profile-dialog[open]').length",
      ),
      0,
    );
    assert.ok(
      !(await evaluate("document.querySelector('main').textContent")).includes(
        "Unsaved Alice",
      ),
    );
    await evaluate("sessionStorage.removeItem('shell-invalidation-proof')");
    assert.deepEqual(serverErrors, []);
    const { errors } = await browser("errors");
    assert.deepEqual(errors, []);
  } finally {
    accounts = false;
    await browser("cookies", "clear");
  }
});
