import { before, after, test } from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile, writeFile, mkdtemp, unlink, rmdir } from "node:fs/promises";
import { tmpdir } from "node:os";
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
const uploadDirectory = await mkdtemp(
  resolve(tmpdir(), "brioche-admin-upload-"),
);
const lessonUpload = resolve(uploadDirectory, "lesson.json");
const releaseUpload = resolve(uploadDirectory, "release.json");
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
let operatorAccount = false;
let adminApproved = false;
let adminWrites = [];
let lessonStatus = 200;
const identityReads = [];
let identityProof = null;
const profile = (id) => ({
  id,
  email: `${id}@example.test`,
  displayName: id === "shell-a" ? "Alice" : "Bob",
  role: operatorAccount ? "operator" : "learner",
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
  if (request.url === "/api/v1/auth/csrf") {
    response.end(JSON.stringify({ csrfToken: "controlled-admin-csrf" }));
    return;
  }
  if (request.url === "/api/v1/operator/overview") {
    response.end(
      JSON.stringify({
        generation: "0",
        activeRelease: null,
        releases: [],
        lessons: [
          {
            id: lesson.id,
            revision: 1,
            title: "在面包店买早餐",
            level: "a1",
            unit: lesson.unitId,
            published: false,
            withdrawn: false,
            approved: adminApproved,
            reviewVersion: adminApproved ? 1 : 0,
            reviewNote: "隔离管理员界面测试",
          },
        ],
      }),
    );
    return;
  }
  if (request.url.endsWith("/review") && request.method === "POST") {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const decision = JSON.parse(body);
      adminWrites.push(decision);
      adminApproved = decision.approved;
      response.end(JSON.stringify({ ...decision, version: 1 }));
    });
    return;
  }
  if (
    [
      "/api/v1/operator/lessons/import",
      "/api/v1/operator/releases/stage",
    ].includes(request.url) &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const upload = JSON.parse(body);
      adminWrites.push({ operation: request.url, ...upload });
      response.end(
        JSON.stringify(
          request.url.endsWith("/import")
            ? { lessonId: lesson.id, revision: 1 }
            : "browser-release",
        ),
      );
    });
    return;
  }
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
    response.statusCode = lessonStatus;
    response.end(JSON.stringify(lessonStatus === 200 ? lesson : {}));
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
      let body = "";
      for await (const chunk of request) body += chunk;
      const proxied = await fetch(
        process.env.INTERNAL_API_URL + url.pathname + url.search,
        {
          method: request.method,
          ...(body ? { body } : {}),
          headers: {
            "X-Shell-Channel": "browser",
            ...(request.headers["content-type"]
              ? { "Content-Type": request.headers["content-type"] }
              : {}),
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

test("operator enters admin from profile and approves using the centered dialog", async () => {
  accounts = true;
  operatorAccount = true;
  adminApproved = false;
  adminWrites = [];
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("set", "viewport", "390", "844");
    await browser("open", origin + "/profile");
    await browser("wait", ".setting-link[href='/admin']");
    const gap = await evaluate(
      `(() => { const heading=[...document.querySelectorAll('.settings-page > h2')].find(el=>el.textContent==='阅读');return heading.getBoundingClientRect().top-heading.previousElementSibling.getBoundingClientRect().bottom; })()`,
    );
    assert.equal(gap, 32);
    await browser("click", ".setting-link[href='/admin']");
    await browser("wait", ".admin-card");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "批准课程",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    const geometry = await evaluate(
      `(() => { const r=document.querySelector('.admin-dialog').getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:innerWidth,height:innerHeight}; })()`,
    );
    assert.ok(
      geometry.left >= 0 &&
        geometry.right <= geometry.width &&
        geometry.top >= 0 &&
        geometry.bottom <= geometry.height,
    );
    await browser("fill", "#admin-reason", "界面协议测试批准");
    await browser("focus", ".admin-dialog .primary");
    await browser("press", "Enter");
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.admin-card-heading')?.textContent.includes('已批准')",
    );
    assert.deepEqual(adminWrites, [
      { version: 0, approved: true, reason: "界面协议测试批准" },
    ]);
    assert.equal(
      await evaluate("document.querySelectorAll('.admin-dialog[open]').length"),
      0,
    );
    assert.equal(
      await evaluate("document.documentElement.scrollWidth > innerWidth"),
      false,
    );
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "发布目录",
      "--exact",
    );
    await browser("wait", "--text", "还没有发布目录");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "创建发布目录",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    await browser("upload", "#admin-document", releaseUpload);
    await browser("fill", "#admin-reason", "界面目录导入测试");
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.admin-dialog .primary')?.disabled === false",
    );
    await browser("focus", ".admin-dialog .primary");
    await browser("press", "Enter");
    await browser("wait", "--text", "发布目录已通过检查，可以预览或切换。");
    assert.equal(adminWrites[1].operation, "/api/v1/operator/releases/stage");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "课程审批",
      "--exact",
    );
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "导入课程",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    await browser("upload", "#admin-document", lessonUpload);
    await browser("fill", "#admin-reason", "界面课程导入测试");
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.admin-dialog .primary')?.disabled === false",
    );
    await browser("focus", ".admin-dialog .primary");
    await browser("press", "Enter");
    await browser("wait", "--text", "课程 v1 已导入，可以预览和审批。");
    assert.equal(adminWrites[2].operation, "/api/v1/operator/lessons/import");
    assert.deepEqual(JSON.parse(adminWrites[2].document), source);
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "导入课程",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    assert.equal(
      await evaluate("document.querySelector('#admin-document').files.length"),
      0,
    );
    await browser("press", "Escape");
  } finally {
    accounts = false;
    operatorAccount = false;
    await browser("cookies", "clear");
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
  await writeFile(lessonUpload, JSON.stringify(source));
  await writeFile(
    releaseUpload,
    JSON.stringify({ id: "browser-release", schemaVersion: "1.0", levels: [] }),
  );
  await new Promise((resolve) => api.listen(0, "127.0.0.1", resolve));
  process.env.INTERNAL_API_URL = `http://127.0.0.1:${api.address().port}`;
  await new Promise((resolve) => web.listen(0, "127.0.0.1", resolve));
  origin = `http://127.0.0.1:${web.address().port}`;
});
after(async () => {
  await unlink(lessonUpload);
  await unlink(releaseUpload);
  await rmdir(uploadDirectory);
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

test("lesson errors recover through native keyboard reload and catalog navigation", async () => {
  try {
    accounts = false;
    lessonStatus = 200;
    opened = true;
    await browser("open", origin);
    await browser("wait", "a[href^='/lessons/']");
    lessonStatus = 503;
    await browser("focus", "a[href^='/lessons/']");
    await browser("press", "Enter");
    await browser(
      "wait",
      "--fn",
      "document.querySelector('main h1')?.textContent==='服务暂时不可用'",
    );
    assert.equal(
      await evaluate("document.querySelectorAll('.reading').length"),
      0,
    );
    lessonStatus = 200;
    await browser("focus", ".error-recovery button.primary");
    await browser("press", "Enter");
    await browser("wait", ".reading");
    assert.ok(
      (await evaluate("document.querySelector('main').textContent")).includes(
        lesson.title.fr,
      ),
    );
    for (const [status, title] of [
      [404, "没有找到这页内容"],
      [410, "课程已撤回"],
    ]) {
      lessonStatus = status;
      await browser("open", `${origin}/lessons/${lesson.id}`);
      await browser(
        "wait",
        "--fn",
        `document.querySelector('main h1')?.textContent===${JSON.stringify(title)}`,
      );
      await browser(
        "wait",
        "--fn",
        "Object.keys(document.querySelector('.error-recovery a')).some(key=>key.startsWith('__reactFiber$'))",
      );
      assert.equal(
        await evaluate("document.querySelectorAll('.reading').length"),
        0,
      );
      await browser("focus", ".error-recovery a[href='/courses']");
      await browser("press", "Enter");
      await browser(
        "wait",
        "--fn",
        "location.pathname==='/courses' && document.activeElement.matches('main h1')",
      );
      assert.equal(
        await evaluate(
          "document.activeElement.tagName + ':' + document.activeElement.textContent",
        ),
        "H1:课程",
      );
      await browser("focus", "a[href^='/lessons/']");
      await browser("press", "Enter");
      await browser(
        "wait",
        "--fn",
        `document.querySelector('main h1')?.textContent===${JSON.stringify(title)}`,
      );
      await browser("back");
      await browser(
        "wait",
        "--fn",
        "location.pathname==='/courses' && !!document.querySelector('.courses-page')",
      );
      await browser(
        "wait",
        "--fn",
        "location.pathname==='/courses' && document.activeElement.matches('main h1')",
      );
    }
    assert.deepEqual(serverErrors, []);
    const { errors } = await browser("errors");
    assert.deepEqual(errors, []);
  } finally {
    lessonStatus = 200;
  }
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
