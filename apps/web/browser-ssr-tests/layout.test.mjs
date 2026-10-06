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
const api = createServer((request, response) => {
  response.setHeader("Content-Type", "application/json");
  if (request.url === "/api/v1/me") {
    response.writeHead(404).end("{}");
    return;
  }
  if (request.url.startsWith("/api/catalog")) {
    response.end(JSON.stringify(catalog));
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
