import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { createRequestHandler } from "react-router";
import * as build from "../build/server/index.js";

const source = JSON.parse(
  await readFile(
    new URL("../../../docs/examples/a1-bakery.lesson.json", import.meta.url),
    "utf8",
  ),
);
const fields = [
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
  ...Object.fromEntries(fields.map((key) => [key, source[key]])),
  media: [],
  audio: [],
  audioTracks: [],
};
let fixture = false;
let authenticated = false;
const profile = {
  id: "00000000-0000-0000-0000-000000000001",
  email: "learner@example.test",
  displayName: "测试学习者",
  role: "learner",
  settings: {
    timeZone: "Asia/Shanghai",
    weeklyDays: 3,
    dailyMinutes: 10,
    showTranslation: false,
    speechRate: 1,
  },
  version: 1,
};
const requests = [];
const originalBase = process.env.INTERNAL_API_URL;
const handler = createRequestHandler(build, "production");
const server = createServer((request, response) => {
  requests.push({
    method: request.method,
    path: request.url,
    cookie: request.headers.cookie,
  });
  response.setHeader("Content-Type", "application/json");
  if (request.url === "/api/v1/me") {
    response.statusCode = authenticated ? 200 : fixture ? 404 : 401;
    response.end(JSON.stringify(authenticated ? profile : {}));
  } else if (request.url === "/api/v1/me/reviews" && authenticated) {
    response.end(
      JSON.stringify({
        items: [],
        dueCount: 0,
        nextDueAt: null,
        localDate: "2026-10-06",
        timeZone: "Asia/Shanghai",
      }),
    );
  } else if (request.url === "/api/catalog") {
    response.end(
      JSON.stringify({
        developmentFixture: fixture,
        levels: [
          {
            id: "a1",
            label: "A1",
            units: [
              { id: lesson.unitId, titleZh: "早餐与面包店", lessons: [lesson] },
            ],
          },
        ],
      }),
    );
  } else if (request.url.startsWith("/api/lessons/")) {
    response.end(JSON.stringify(lesson));
  } else {
    response.statusCode = 401;
    response.end("{}");
  }
});
before(async () => {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  process.env.INTERNAL_API_URL = `http://127.0.0.1:${server.address().port}`;
});
after(async () => {
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  if (originalBase === undefined) delete process.env.INTERNAL_API_URL;
  else process.env.INTERNAL_API_URL = originalBase;
});
const request = (path) =>
  handler(
    new Request("http://brioche.test" + path, {
      headers: authenticated
        ? { cookie: "brioche.sid=controlled-ssr-session; unrelated=omit" }
        : {},
    }),
  );

test("production legacy entries reach real learning and authenticated review without writes", async () => {
  fixture = false;
  requests.length = 0;
  for (const [path, destination] of [
    [`/practice/${lesson.id}`, `/lessons/${lesson.id}`],
    [`/review/${lesson.id}`, "/reviews"],
    ["/reviews", "/login?next=/reviews"],
  ]) {
    const response = await request(path);
    assert.equal(response.status, 302, path);
    assert.equal(response.headers.get("Location"), destination);
  }
  assert.ok(requests.every((entry) => entry.method === "GET"));
});

test("fixture practice and review remain available, with demo homepage entry", async () => {
  fixture = true;
  for (const path of [`/practice/${lesson.id}`, `/review/${lesson.id}`]) {
    const response = await request(path);
    assert.equal(response.status, 200, path);
    assert.match(await response.text(), /je voudrais/i);
  }
  const response = await request("/");
  assert.equal(response.status, 200);
  assert.match(
    await response.text(),
    new RegExp(`href="/review/${lesson.id}"`),
  );
});

test("anonymous production homepage leads to account review login", async () => {
  fixture = false;
  const response = await request("/");
  assert.equal(response.status, 200);
  const html = await response.text();
  assert.match(html, /href="\/login\?next=\/reviews"/);
  assert.doesNotMatch(html, /href="\/review\//);
});

test("signed-in legacy review redirects to the private queue and forwards only its session cookie", async () => {
  fixture = false;
  authenticated = true;
  requests.length = 0;
  try {
    const legacy = await request(`/review/${lesson.id}`);
    assert.equal(legacy.status, 302);
    assert.equal(legacy.headers.get("Location"), "/reviews");
    const queue = await request("/reviews");
    assert.equal(queue.status, 200);
    assert.ok((await queue.text()).includes("复习"));
    const privateRead = requests.find(
      (entry) => entry.path === "/api/v1/me/reviews",
    );
    assert.equal(privateRead?.cookie, "brioche.sid=controlled-ssr-session");
    assert.ok(requests.every((entry) => entry.method === "GET"));
  } finally {
    authenticated = false;
  }
});
