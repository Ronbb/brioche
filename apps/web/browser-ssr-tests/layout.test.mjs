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
let managedRole = "learner";
let managedSessionRevoked = false;
let pendingTokenRevoked = false;
const voiceSeed = JSON.parse(
  await readFile(
    new URL("../../../docs/characters/voices.json", import.meta.url),
    "utf8",
  ),
);
let characterVoice = { ...voiceSeed.items[0], voiceRevision: 0, profile: null };
const characterAvatar = await readFile(
  new URL("../public/assets/avatars/camille.svg", import.meta.url),
);
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
  if (request.url.startsWith("/api/v1/operator/assets")) {
    if (request.method === "POST") {
      const chunks = [];
      request.on("data", (chunk) => chunks.push(chunk));
      request.on("end", async () => {
        try {
          const payload = await new Request("http://test", {
            method: "POST",
            headers: { "content-type": request.headers["content-type"] },
            body: Buffer.concat(chunks),
          }).formData();
          const document = JSON.parse(payload.get("document"));
          const file = payload.get("file");
          adminWrites.push({
            operation: request.url,
            ...document,
            fileBytes: file.size,
            csrf: request.headers["x-csrf-token"],
          });
          response.end(
            JSON.stringify({
              assetId: document.assetId,
              revision: document.revision,
            }),
          );
        } catch (e) {
          serverErrors.push(String(e));
          response.writeHead(400).end("{}");
        }
      });
      return;
    }
    if (request.url.endsWith("/file")) {
      response.setHeader("content-type", "image/svg+xml");
      response.end(characterAvatar);
    } else {
      const query = new URL(request.url, "http://test").searchParams;
      response.end(
        JSON.stringify({
          items:
            query.get("q") === "missing"
              ? []
              : [
                  {
                    asset: {
                      assetId: "avatar-camille-v1",
                      revision: 1,
                      sha256: "a".repeat(64),
                      mimeType: "image/svg+xml",
                      width: 96,
                      height: 96,
                      altZh: "Camille 头像",
                      creditZh: "隔离测试",
                      url: "/api/v1/operator/assets/avatar-camille-v1/1/file",
                    },
                    source: "test:original",
                    license: "LicenseRef-TestOnly",
                    creator: "test fixture",
                    rightsConfirmed: true,
                    byteSize: 1234,
                  },
                ],
          next: null,
        }),
      );
    }
    return;
  }
  if (request.url === "/api/v1/auth/csrf") {
    response.end(JSON.stringify({ csrfToken: "controlled-admin-csrf" }));
    return;
  }
  if (
    request.url === "/api/v1/operator/characters/revisions" &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const change = JSON.parse(body);
      adminWrites.push({ operation: request.url, ...change });
      characterVoice = {
        character: {
          characterId: change.characterId,
          revision: change.expectedRevision + 1,
          displayName: change.displayName,
          avatarId: change.avatarId,
          speechLocale: "fr-FR",
        },
        avatarRevision: change.avatarRevision,
        voiceRevision: 0,
        profile: null,
      };
      response.end(JSON.stringify(characterVoice));
    });
    return;
  }
  if (
    request.url === "/api/v1/operator/characters" &&
    request.method === "GET"
  ) {
    response.end(JSON.stringify({ items: [characterVoice], nextId: null }));
    return;
  }
  if (
    request.url === "/api/v1/operator/characters" &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const change = JSON.parse(body);
      adminWrites.push({ operation: request.url, ...change });
      characterVoice = {
        ...characterVoice,
        voiceRevision: characterVoice.voiceRevision + 1,
        profile: change.profile,
      };
      response.end(JSON.stringify(characterVoice));
    });
    return;
  }
  if (
    /^\/api\/v1\/operator\/characters\/character-camille\/1\/avatar$/.test(
      request.url,
    )
  ) {
    response.setHeader("content-type", "image/svg+xml");
    response.end(characterAvatar);
    return;
  }
  if (
    request.url.startsWith("/api/v1/operator/accounts/pending-tokens") &&
    request.method === "GET"
  ) {
    response.end(
      JSON.stringify({
        items: pendingTokenRevoked
          ? []
          : [
              {
                id: "b".repeat(64),
                email: "pending@example.test",
                kind: "invite",
                role: "learner",
                expiresAt: "2027-01-01T00:00:00Z",
              },
            ],
        nextId: null,
      }),
    );
    return;
  }
  if (
    request.url ===
      `/api/v1/operator/accounts/pending-tokens/${"b".repeat(64)}/revoke` &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      adminWrites.push({ operation: request.url, ...JSON.parse(body) });
      pendingTokenRevoked = true;
      response.end("true");
    });
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
  if (
    request.url === "/api/v1/operator/accounts/101/sessions" &&
    request.method === "GET"
  ) {
    response.end(
      JSON.stringify({
        account: {
          id: "101",
          email: "learner@example.test",
          displayName: "测试账号",
          role: managedRole,
        },
        items: managedSessionRevoked
          ? []
          : [
              {
                id: "a".repeat(64),
                expiresAt: "2027-01-01T00:00:00Z",
                current: false,
              },
            ],
        nextId: null,
      }),
    );
    return;
  }
  if (
    request.url ===
      "/api/v1/operator/accounts/101/sessions/" + "a".repeat(64) + "/revoke" &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      adminWrites.push({ operation: request.url, ...JSON.parse(body) });
      managedSessionRevoked = true;
      response.end(JSON.stringify({ current: false }));
    });
    return;
  }
  if (
    request.url.startsWith("/api/v1/operator/accounts") &&
    request.method === "GET"
  ) {
    const older = new URL(
      request.url,
      "http://controlled.test",
    ).searchParams.has("afterId");
    response.end(
      JSON.stringify({
        items: [
          {
            id: older ? "102" : "101",
            email: older ? "older@example.test" : "learner@example.test",
            displayName: older ? "较早账号" : "测试账号",
            role: older ? "learner" : managedRole,
          },
        ],
        nextId: older ? null : "101",
      }),
    );
    return;
  }
  if (
    request.url === "/api/v1/operator/accounts/101/role" &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const change = JSON.parse(body);
      adminWrites.push({ operation: request.url, ...change });
      managedRole = change.role;
      response.end(
        JSON.stringify({
          id: "101",
          email: "learner@example.test",
          displayName: "测试账号",
          role: managedRole,
        }),
      );
    });
    return;
  }
  if (
    request.url === "/api/v1/operator/accounts/token" &&
    request.method === "POST"
  ) {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => {
      const issuance = JSON.parse(body);
      adminWrites.push({ operation: request.url, ...issuance });
      setTimeout(
        () =>
          response.end(
            JSON.stringify({
              token: "f".repeat(64),
              email: issuance.email,
              kind: issuance.kind,
              expiresInSeconds: issuance.kind === "invite" ? 172800 : 1800,
            }),
          ),
        300,
      );
    });
    return;
  }
  if (request.url.startsWith("/api/v1/operator/history")) {
    const older = new URL(
      request.url,
      "http://controlled.test",
    ).searchParams.has("beforeKey");
    response.end(
      JSON.stringify({
        items: [
          {
            key: older ? "content:1" : "review:lesson:1:1",
            action: older ? "stage" : "approve",
            target: "在面包店买早餐 v1",
            actor: older ? "local-author-cli" : "user:101",
            reason: older ? "较早的目录检查" : "界面协议测试批准",
            createdAt: "2026-10-07T01:02:03.123456Z",
          },
        ],
        next: older
          ? null
          : {
              beforeTime: "2026-10-07T01:02:03.123456Z",
              beforeKey: "review:lesson:1:1",
            },
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
      const chunks = [];
      for await (const chunk of request) chunks.push(chunk);
      const body = Buffer.concat(chunks);
      const proxied = await fetch(
        process.env.INTERNAL_API_URL + url.pathname + url.search,
        {
          method: request.method,
          ...(body.length ? { body } : {}),
          headers: {
            "X-Shell-Channel": "browser",
            ...(request.headers["x-csrf-token"]
              ? { "x-csrf-token": request.headers["x-csrf-token"] }
              : {}),
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
        "Content-Type":
          proxied.headers.get("content-type") ?? "application/json",
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

test("visual registry renders private images and searches at mobile widths", async () => {
  accounts = true;
  operatorAccount = true;
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("open", origin + "/admin/assets");
    await browser("wait", ".admin-asset img");
    await browser(
      "wait",
      "--fn",
      "document.querySelector('.admin-asset img')?.naturalWidth > 0",
    );
    for (const width of [320, 390, 678, 1024]) {
      await browser("set", "viewport", String(width), "844");
      assert.equal(
        await evaluate("document.documentElement.scrollWidth<=innerWidth"),
        true,
      );
    }
    assert.equal(
      await evaluate(
        "document.querySelector('.admin-asset').textContent.includes('LicenseRef-TestOnly')",
      ),
      true,
    );
    await browser("fill", "input[name=q]", "missing");
    await browser("press", "Enter");
    await browser("wait", "--text", "没有符合条件的素材。");
    assert.equal(
      await evaluate("new URL(location.href).searchParams.get('q')"),
      "missing",
    );
  } finally {
    accounts = false;
    operatorAccount = false;
  }
});

test("operator uploads an actual SVG and supplies provenance in the mobile dialog", async () => {
  accounts = true;
  operatorAccount = true;
  adminWrites = [];
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("set", "viewport", "390", "844");
    await browser("open", origin + "/admin/assets");
    await browser("wait", ".admin-asset");
    const snapshot = await browser("snapshot", "-i");
    const ref = Object.entries(snapshot.refs).find(
      ([, item]) => item.role === "button" && item.name === "上传图片",
    )?.[0];
    assert.ok(ref, JSON.stringify(snapshot));
    await browser("click", "@" + ref);
    await browser("wait", ".admin-dialog[open]");
    for (const width of [320, 390]) {
      await browser("set", "viewport", String(width), "844");
      assert.equal(
        await evaluate(
          "document.querySelector('.admin-dialog').scrollWidth <= document.querySelector('.admin-dialog').clientWidth",
        ),
        true,
      );
    }
    await browser(
      "upload",
      "input[name=file]",
      fileURLToPath(
        new URL("../public/assets/avatars/camille.svg", import.meta.url),
      ),
    );
    for (const [name, value] of Object.entries({
      assetId: "browser-upload",
      altZh: "浏览器上传测试",
      source: "test:original",
      license: "LicenseRef-TestOnly",
      creator: "test fixture",
      creditZh: "仅隔离测试",
      reason: "验证实际文件上传",
    }))
      await browser("fill", `.admin-dialog [name=${name}]`, value);
    await browser("check", "input[name=rightsConfirmed]");
    await browser("fill", "textarea[name=reason]", "验证实际文件上传");
    await browser("press", "Tab");
    await browser("press", "Tab");
    await browser("press", "Enter");
    await browser("wait", "--text", "素材 browser-upload · 版本 1 已登记。");
    assert.equal(adminWrites.length, 1);
    assert.equal(adminWrites[0].assetId, "browser-upload");
    assert.equal(adminWrites[0].mimeType, "image/svg+xml");
    assert.equal(adminWrites[0].rightsConfirmed, true);
    assert.equal(adminWrites[0].csrf, "controlled-admin-csrf");
    assert.equal(adminWrites[0].fileBytes, characterAvatar.length);
    assert.equal(
      await evaluate("document.querySelector('.admin-dialog[open]') === null"),
      true,
    );
    assert.equal(
      await evaluate("document.documentElement.scrollWidth<=innerWidth"),
      true,
    );
  } finally {
    accounts = false;
    operatorAccount = false;
  }
});

test("operator revokes a pending invitation using the mobile admin page", async () => {
  accounts = true;
  operatorAccount = true;
  pendingTokenRevoked = false;
  adminWrites = [];
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("set", "viewport", "390", "844");
    await browser("open", origin + "/admin/tokens");
    await browser("wait", ".admin-card");
    await browser("click", ".admin-card button");
    await browser("wait", ".admin-dialog[open]");
    await browser("fill", "#token-reason", "隔离邀请撤销测试");
    await browser("press", "Tab");
    await browser("press", "Enter");
    await browser("wait", "--text", "链接已撤销。");
    assert.equal(adminWrites[0].reason, "隔离邀请撤销测试");
    assert.match(adminWrites[0].operation, /pending-tokens\/[b]+\/revoke$/);
    assert.equal(
      await evaluate("document.querySelector('.admin-dialog').open"),
      false,
    );
    await browser("wait", "--text", "没有待使用的链接。");
    assert.equal(
      await evaluate("document.documentElement.scrollWidth<=innerWidth"),
      true,
    );
  } finally {
    accounts = false;
    operatorAccount = false;
  }
});

test("operator creates a character using a private avatar picker", async () => {
  accounts = true;
  operatorAccount = true;
  adminWrites = [];
  characterVoice = { ...voiceSeed.items[0], voiceRevision: 0, profile: null };
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("set", "viewport", "390", "844");
    await browser("open", origin + "/admin/characters");
    await browser("wait", ".character-profile");
    const snapshot = await browser("snapshot", "-i");
    const ref = Object.entries(snapshot.refs).find(
      ([, item]) => item.role === "button" && item.name === "新建角色",
    )?.[0];
    assert.ok(ref);
    await browser("click", "@" + ref);
    await browser("wait", ".avatar-picker button");
    await browser(
      "fill",
      ".admin-dialog[open] input[name=characterId]",
      "character-browser",
    );
    await browser(
      "fill",
      ".admin-dialog[open] input[name=displayName]",
      "Émile",
    );
    await browser("click", ".avatar-picker button");
    await browser(
      "fill",
      ".admin-dialog[open] textarea[name=reason]",
      "隔离角色登记测试",
    );
    for (const width of [320, 390]) {
      await browser("set", "viewport", String(width), "844");
      assert.equal(
        await evaluate(
          "document.querySelector('.admin-dialog[open]').scrollWidth<=document.querySelector('.admin-dialog[open]').clientWidth",
        ),
        true,
      );
    }
    await browser("focus", ".admin-dialog[open] .primary");
    await browser("press", "Enter");
    await browser(
      "wait",
      "--fn",
      "!document.querySelector('.admin-dialog[open]')",
    );
    await browser("wait", "--text", "Émile");
    assert.equal(adminWrites.length, 1);
    assert.equal(adminWrites[0].expectedRevision, 0);
    assert.equal(adminWrites[0].avatarId, "avatar-camille-v1");
    assert.equal(adminWrites[0].avatarRevision, 1);
    assert.equal(adminWrites[0].displayName, "Émile");
    assert.equal(adminWrites[0].reason, "隔离角色登记测试");
  } finally {
    accounts = false;
    operatorAccount = false;
    characterVoice = { ...voiceSeed.items[0], voiceRevision: 0, profile: null };
  }
});

test("operator versions a character voice profile through the real mobile page", async () => {
  accounts = true;
  operatorAccount = true;
  adminWrites = [];
  characterVoice = { ...voiceSeed.items[0], voiceRevision: 0, profile: null };
  try {
    await browser("open", origin + "/");
    await browser("cookies", "set", "brioche.sid", "shell-a");
    await browser("set", "viewport", "390", "844");
    await browser("open", origin + "/admin/characters");
    await browser("wait", ".character-profile");
    assert.equal(
      await evaluate("document.documentElement.scrollWidth<=innerWidth"),
      true,
    );
    const voiceSnapshot = await browser("snapshot", "-i");
    const voiceRef = Object.entries(voiceSnapshot.refs).find(
      ([, item]) => item.role === "button" && item.name === "配置声音档案",
    )?.[0];
    assert.ok(voiceRef);
    await browser("click", "@" + voiceRef);
    await browser("wait", ".admin-dialog[open]");
    await browser(
      "fill",
      ".admin-dialog[open] form > label:first-of-type textarea",
      "Warm, curious and politely reserved.",
    );
    await browser(
      "fill",
      ".admin-dialog[open] form > label:last-of-type input",
      "隔离声音档案测试",
    );
    await browser("press", "Tab");
    assert.equal(await evaluate("document.activeElement.type"), "submit");
    await browser("press", "Enter");
    await browser("wait", "--text", "声音档案已保存为新版本。");
    assert.equal(
      await evaluate("document.querySelector('.admin-dialog').open"),
      false,
    );
    assert.equal(adminWrites[0].expectedVoiceRevision, 0);
    assert.equal(
      adminWrites[0].profile.personality,
      "Warm, curious and politely reserved.",
    );
    assert.equal(adminWrites[0].reason, "隔离声音档案测试");
    assert.equal(
      await evaluate(
        "document.querySelector('.character-profile-head img').naturalWidth>0",
      ),
      true,
    );
  } finally {
    accounts = false;
    operatorAccount = false;
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
    await browser("click", "a[href='/admin/history']");
    await browser("wait", ".admin-history");
    assert.equal(
      await evaluate("document.querySelector('h1').textContent"),
      "审批与发布记录",
    );
    assert.equal(
      await evaluate("document.querySelector('time').dateTime"),
      "2026-10-07T01:02:03.123456Z",
    );
    assert.match(
      await evaluate("document.querySelector('.admin-actor').textContent"),
      /user:101/,
    );
    await browser("click", "a[href^='/admin/history?']");
    await browser("wait", "--text", "较早的目录检查");
    assert.match(
      await evaluate("document.querySelector('.admin-actor').textContent"),
      /local-author-cli/,
    );
    assert.equal(
      await evaluate(
        "document.querySelectorAll(\"a[href^='/admin/history?']\").length",
      ),
      0,
    );
    await browser("back");
    await browser("wait", "a[href^='/admin/history?']");
    assert.equal(await evaluate("document.activeElement.tagName"), "H1");
    await browser("click", "a[href='/admin']");
    await browser("wait", "a[href='/admin/accounts']");
    await browser("click", "a[href='/admin/accounts']");
    await browser("wait", "#account-search");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "邀请新账号",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    await browser("fill", "#account-email", "invited@example.test");
    await browser("fill", "#account-reason", "隔离邀请界面测试");
    await browser("focus", ".admin-dialog .primary");
    await browser("press", "Enter");
    await browser("press", "Enter");
    await browser("wait", "#account-link");
    assert.equal(await evaluate("document.activeElement.id"), "account-link");
    assert.equal(
      adminWrites.filter(
        (item) => item.operation === "/api/v1/operator/accounts/token",
      ).length,
      1,
    );
    assert.equal(
      await evaluate(
        "document.querySelector('#account-link').value.startsWith(location.origin+'/invite#token=')",
      ),
      true,
    );
    assert.equal(
      await evaluate(
        "Object.keys(localStorage).some(key=>localStorage.getItem(key).includes('ffffffffffffffff'))",
      ),
      false,
    );
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "关闭",
      "--exact",
    );
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "生成密码重置链接",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    assert.equal(
      await evaluate("document.querySelector('#account-link')===null"),
      true,
    );
    assert.equal(
      await evaluate("document.querySelector('#account-email').readOnly"),
      true,
    );
    await browser("press", "Escape");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "设为管理员",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    await browser("fill", "#account-reason", "隔离权限界面测试");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "确认修改权限",
      "--exact",
    );
    await browser("wait", "--text", "改为学习者");
    assert.equal(
      await evaluate("document.querySelector('.admin-dialog').open"),
      false,
    );
    const change = adminWrites.find(
      (item) => item.operation === "/api/v1/operator/accounts/101/role",
    );
    assert.equal(change.expectedRole, "learner");
    assert.equal(change.role, "operator");
    assert.equal(change.reason, "隔离权限界面测试");
    managedRole = "learner";
    assert.equal(
      await evaluate(
        "document.querySelector('a[href=\"/admin/accounts/101/sessions\"]')?.textContent",
      ),
      "登录会话",
    );
    const sessionsSnapshot = await browser("snapshot", "-i");
    assert.match(JSON.stringify(sessionsSnapshot), /登录会话/);
    const sessionsRef = Object.entries(sessionsSnapshot.refs).find(
      ([, item]) => item.role === "link" && item.name === "登录会话",
    )?.[0];
    assert.ok(sessionsRef);
    await browser("click", `@${sessionsRef}`);
    await browser("wait", "--text", "撤销此会话");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "撤销此会话",
      "--exact",
    );
    await browser("wait", ".admin-dialog[open]");
    await browser("fill", "#session-reason", "隔离会话撤销测试");
    await browser(
      "find",
      "role",
      "button",
      "click",
      "--name",
      "确认撤销",
      "--exact",
    );
    await browser("wait", "--text", "没有有效的登录会话。");
    assert.equal(
      await evaluate("document.querySelector('.admin-dialog').open"),
      false,
    );
    assert.equal(
      adminWrites.find((item) => item.operation?.endsWith("/revoke")).reason,
      "隔离会话撤销测试",
    );
    const accountSnapshot = await browser("snapshot", "-i");
    const accountRef = Object.entries(accountSnapshot.refs).find(
      ([, item]) => item.role === "link" && item.name === "账号管理",
    )?.[0];
    assert.ok(accountRef);
    await browser("click", `@${accountRef}`);
    await browser("wait", "#account-search");
    managedSessionRevoked = false;
    await browser("click", "a[href^='/admin/accounts?']");
    await browser("wait", "--text", "较早账号");
    assert.equal(await evaluate("document.activeElement.tagName"), "H1");
    await browser("back");
    await browser("wait", "--text", "测试账号");
    assert.equal(await evaluate("document.activeElement.tagName"), "H1");
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

test("medium reading uses an animated modal with retained content and keyboard dismissal", async () => {
  const originalBlocks = lesson.blocks;
  const body = lesson.blocks.find((block) => block.type === "dialogue");
  lesson.blocks = originalBlocks.filter(
    (block) => !["dialogue", "article"].includes(block.type) || block === body,
  );
  try {
    await browser("set", "media", "light");
    await browser("set", "viewport", "678", "884");
    await browser("open", `${origin}/lessons/${lesson.id}`);
    opened = true;
    await browser(
      "wait",
      "--fn",
      "!!document.querySelector('.word.known') && !!window.__reactRouterContext && getComputedStyle(document.documentElement).getPropertyValue('--surface').trim()==='#fffdf7'",
    );
    assert.equal(
      await evaluate("document.querySelectorAll('[role=tab]').length"),
      0,
    );
    assert.equal(
      await evaluate(
        "getComputedStyle(document.querySelector('aside.knowledge')).display",
      ),
      "none",
    );
    assert.equal(
      await evaluate("document.documentElement.scrollWidth <= innerWidth"),
      true,
    );
    await browser("focus", ".word.known");
    await browser("press", "Enter");
    await browser("wait", "dialog.knowledge-sheet[open]");
    const motion = await evaluate(
      "(()=>{const d=document.querySelector('dialog.knowledge-sheet');return {card:getComputedStyle(d).animationName,backdrop:getComputedStyle(d,'::backdrop').animationName,inside:d.contains(document.activeElement),text:d.querySelector('h2')?.textContent}})()",
    );
    assert.equal(motion.card, "knowledge-enter");
    assert.equal(motion.backdrop, "knowledge-backdrop-enter");
    assert.equal(motion.inside, true);
    const closing = await evaluate(
      "(()=>{const d=document.querySelector('dialog.knowledge-sheet');d.dispatchEvent(new Event('cancel',{cancelable:true}));return new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve({open:d.open,closing:d.hasAttribute('data-closing'),text:d.querySelector('h2')?.textContent,animation:getComputedStyle(d).animationName}))))})()",
    );
    assert.equal(closing.open, true);
    assert.equal(closing.closing, true);
    assert.equal(closing.text, motion.text);
    assert.equal(closing.animation, "knowledge-leave");
    await browser(
      "wait",
      "--fn",
      "!document.querySelector('dialog.knowledge-sheet').open",
    );
    assert.equal(
      await evaluate("document.activeElement.matches('.word.known')"),
      true,
    );
    await browser("press", "Enter");
    await browser("wait", "dialog.knowledge-sheet[open]");
    await browser("press", "Escape");
    await browser(
      "wait",
      "--fn",
      "!document.querySelector('dialog.knowledge-sheet').open",
    );
    assert.equal(
      await evaluate("document.activeElement.matches('.word.known')"),
      true,
    );
    await browser("set", "media", "light", "reduced-motion");
    await browser("press", "Enter");
    await browser("wait", "dialog.knowledge-sheet[open]");
    assert.equal(
      await evaluate(
        "getComputedStyle(document.querySelector('dialog.knowledge-sheet')).animationName",
      ),
      "none",
    );
    assert.equal(
      await evaluate(
        "getComputedStyle(document.querySelector('dialog.knowledge-sheet'),'::backdrop').animationName",
      ),
      "none",
    );
    await browser("press", "Escape");
    assert.equal(
      await evaluate("document.querySelector('dialog.knowledge-sheet').open"),
      false,
    );
  } finally {
    lesson.blocks = originalBlocks;
    await browser("set", "media", "light");
  }
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
