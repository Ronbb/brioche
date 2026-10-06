import { before, after, afterEach, test } from "node:test";
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
let browserOpened = false;
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
  browserOpened = true;
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
    if (browserOpened) await browser("close");
  } finally {
    await server?.close();
  }
});
afterEach(async () => {
  if (!browserOpened) return;
  const { errors } = await browser("errors");
  await browser("errors", "--clear");
  assert.deepEqual(
    errors,
    [],
    "uncaught page errors must fail the interaction regression",
  );
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

test("late speech callbacks cannot interrupt newer playback, while current failures stop and allow retry", async () => {
  await open("reading");
  await browser("focus", ".speaker");
  await press("Enter");
  await browser("wait", "--fn", "qa.playback==='playing'");
  await evaluate("qa.oldUtterance=qa.lastUtterance");
  await browser("focus", "[role=tab]");
  await press("End");
  await browser("focus", ".speaker");
  await press("Enter");
  await browser("wait", "--fn", "qa.playback==='playing'");
  const currentId = await evaluate("qa.playbackId");
  await evaluate(
    "qa.oldUtterance.onstart();qa.oldUtterance.onerror({error:'canceled'});qa.oldUtterance.onend()",
  );
  assert.deepEqual(
    await evaluate(
      "({status:qa.playback,id:qa.playbackId,count:qa.spoken.length,toast:document.querySelector('.toast [role=status]').textContent})",
    ),
    { status: "playing", id: currentId, count: 2, toast: "" },
  );
  await evaluate("qa.lastUtterance.onerror({error:'interrupted'})");
  await browser("wait", "--fn", "qa.playback==='idle'");
  assert.equal(
    await evaluate(
      "document.querySelector('.toast [role=status]').textContent",
    ),
    "朗读已中断，请再次点击播放。",
  );
  await press("Enter");
  await browser("wait", "--fn", "qa.playback==='playing'");
  assert.deepEqual(
    await evaluate(
      "({id:qa.playbackId,last:qa.spoken.at(-1),count:qa.spoken.length})",
    ),
    { id: currentId, last: "Bonsoir !", count: 3 },
  );
});

test("custom settings dialogs remain in the viewport and restore keyboard focus and search", async () => {
  await open("choices");
  const rate = 'button[aria-label^="朗读速度："]';
  for (const width of [320, 390, 900]) {
    await browser("set", "viewport", String(width), "700");
    await browser("focus", rate);
    await press("Enter");
    await browser("wait", "dialog[open]");
    assert.equal(
      await evaluate("document.activeElement.getAttribute('aria-checked')"),
      "true",
    );
    assert.equal(
      await evaluate(
        "(()=>{const r=document.querySelector('dialog[open]').getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight})()",
      ),
      true,
    );
    await press("End");
    assert.equal(
      await evaluate("document.activeElement.textContent.trim()"),
      "1.5×",
    );
    await press("Enter");
    assert.equal(
      await evaluate("document.querySelectorAll('dialog[open]').length"),
      0,
    );
    assert.equal(
      await evaluate(
        "document.activeElement.matches('button[aria-label^=\"朗读速度：\"]')",
      ),
      true,
    );
    await press("Enter");
    await press("Escape");
    assert.equal(
      await evaluate(
        "document.activeElement.matches('button[aria-label^=\"朗读速度：\"]')",
      ),
      true,
    );
  }
  const zone = 'button[aria-label^="时区："]';
  await browser("focus", zone);
  await press("Enter");
  await browser("fill", "dialog[open] input", "no-matching-city");
  assert.equal(
    await evaluate(
      "document.querySelectorAll('dialog[open] [role=radio]').length",
    ),
    0,
  );
  await press("Escape");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('dialog[open] input')?.value===''",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('dialog[open] [role=radio]').length",
    ),
    2,
  );
  assert.equal(
    await evaluate("document.activeElement.matches('input[type=search]')"),
    true,
  );
  await browser("fill", "dialog[open] input", "Paris");
  await press("Tab");
  await press("Enter");
  assert.equal(
    await evaluate("document.activeElement.getAttribute('aria-label')"),
    "时区：巴黎",
  );
});

test("a profile identity change discards the previous editor and permits independent saves", async () => {
  await open("profile");
  const edit = 'button[aria-label="编辑个人资料与学习目标"]';
  await browser("focus", edit);
  await press("Enter");
  await browser("fill", ".profile-dialog input", "Alice edited");
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===1");
  await evaluate("qa.changeUser()");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.profile-summary h2')?.textContent==='Bob'",
  );
  assert.equal(
    await evaluate("document.querySelectorAll('.profile-dialog[open]').length"),
    0,
  );
  await browser("focus", edit);
  await press("Enter");
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Bob",
  );
  await browser("fill", ".profile-dialog input", "Bob edited");
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===2");
  assert.equal(await evaluate("qa.profileWrites[1].version"), 10);
  await evaluate(
    "qa.profileRelease[0]({id:'account-a',email:'a@example.test',displayName:'Alice edited',role:'learner',version:2,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
  );
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Bob edited",
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.profile-dialog button[type=submit]').getAttribute('aria-busy')",
    ),
    "true",
  );
  await evaluate(
    "qa.profileRelease[1]({id:'account-b',email:'b@example.test',displayName:'Bob edited',role:'learner',version:11,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
  );
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.profile-summary h2')?.textContent==='Bob edited' && !document.querySelector('.profile-dialog[open]')",
  );
});

test("closing a changed profile asks before discarding and preserves the draft when retained", async () => {
  await open("profile");
  const edit = 'button[aria-label="编辑个人资料与学习目标"]';
  await browser("focus", edit);
  await press("Enter");
  await browser("fill", ".profile-dialog input", "Unsaved name");
  await press("Escape");
  assert.equal(
    await evaluate("document.querySelectorAll('.profile-dialog[open]').length"),
    1,
  );
  await browser("wait", ".profile-discard");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "放弃这些修改？",
  );
  await browser("focus", ".profile-discard .primary");
  await press("Enter");
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Unsaved name",
  );
  assert.equal(
    await evaluate("document.activeElement.matches('.profile-dialog input')"),
    true,
  );
  assert.equal(await evaluate("qa.profileWrites.length"), 0);
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    true,
  );
  await browser("focus", 'button[aria-label="关闭个人资料"]');
  await press("Enter");
  await browser("focus", ".profile-discard .text-button");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.profile-dialog[open]')",
  );
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    false,
  );
  await browser("focus", edit);
  await press("Enter");
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Alice",
  );
  await press("Escape");
  assert.equal(
    await evaluate("document.querySelectorAll('.profile-dialog[open]').length"),
    0,
  );
});

test("profile drafts guard route pushes and history pops, including saves already in flight", async () => {
  await open("profile");
  const edit = 'button[aria-label="编辑个人资料与学习目标"]';
  await browser("focus", edit);
  await press("Enter");
  await browser("fill", ".profile-dialog input", "Keep this draft");
  await evaluate("qa.navigate('/login')");
  await browser(
    "wait",
    "--fn",
    "qa.route==='/login'||!!document.querySelector('.profile-discard')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  await browser("focus", ".profile-discard .primary");
  await press("Enter");
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Keep this draft",
  );
  await evaluate("qa.navigate(-1)");
  await browser("wait", ".profile-discard");
  await browser("focus", ".profile-discard .text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.route==='/previous'");
  assert.equal(await evaluate("qa.profileWrites.length"), 0);

  await open("profile");
  await browser("focus", edit);
  await press("Enter");
  await browser("fill", ".profile-dialog input", "Saved before leaving");
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===1");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".profile-leave-status");
  assert.equal(
    await evaluate(
      "!!document.querySelector('.profile-dialog[open] .profile-leave-status')",
    ),
    true,
  );
  assert.equal(await evaluate("qa.route"), "/");
  await evaluate(
    "qa.profileRelease[0]({id:'account-a',email:'a@example.test',displayName:'Saved before leaving',role:'learner',version:2,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
  );
  await browser("wait", "--fn", "qa.route==='/login'");
  assert.equal(await evaluate("qa.profileWrites.length"), 1);
});

test("failed profile saves retain blocked drafts and require explicit version-aware recovery", async () => {
  const edit = 'button[aria-label="编辑个人资料与学习目标"]';
  for (const status of [409, 503]) {
    await open("profile");
    await browser("focus", edit);
    await press("Enter");
    await browser("fill", ".profile-dialog input", "My draft");
    await browser("focus", ".profile-dialog button[type=submit]");
    await press("Enter");
    await browser("wait", "--fn", "qa.profileWrites.length===1");
    await evaluate("qa.navigate('/login')");
    await browser("wait", ".profile-dialog .profile-leave-status");
    await evaluate("qa.profileRelease[0](" + status + ")");
    await browser("wait", "--fn", "qa.profileReads.length===1");
    assert.equal(await evaluate("qa.route"), "/");
    await evaluate(
      "qa.profileReads[0]({id:'account-a',email:'a@example.test',displayName:" +
        JSON.stringify(status === 409 ? "Other device" : "My draft") +
        ",role:'learner',version:7,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
    );
    await browser("wait", ".profile-discard");
    assert.equal(
      await evaluate("document.activeElement.textContent"),
      "放弃这些修改？",
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.profile-discard p').textContent",
      ),
      "上次保存未获确认。离开将丢弃当前编辑草稿。",
    );
    assert.equal(await evaluate("qa.profileWrites.length"), 1);
    await press("Escape");
    assert.equal(
      await evaluate("document.querySelector('.profile-dialog input').value"),
      "My draft",
    );
    await browser("focus", ".profile-dialog button[type=submit]");
    await press("Enter");
    if (status === 409) {
      await browser("wait", "--fn", "qa.profileWrites.length===2");
      assert.deepEqual(await evaluate("qa.profileWrites[1]"), {
        displayName: "My draft",
        version: 7,
      });
      await evaluate(
        "qa.profileRelease[1]({id:'account-a',email:'a@example.test',displayName:'My draft',role:'learner',version:8,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
      );
    }
    await browser(
      "wait",
      "--fn",
      "!document.querySelector('.profile-dialog[open]')",
    );
    assert.equal(await evaluate("qa.route"), "/");
    assert.equal(
      await evaluate("qa.profileWrites.length"),
      status === 409 ? 2 : 1,
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.profile-summary h2').textContent",
      ),
      "My draft",
    );
  }
});

test("unauthenticated recovery removes the old profile and releases its navigation guard", async () => {
  for (const status of [503, 401]) {
    await open("profile");
    await browser("focus", 'button[aria-label="编辑个人资料与学习目标"]');
    await press("Enter");
    await browser("fill", ".profile-dialog input", "Private draft");
    await browser("focus", ".profile-dialog button[type=submit]");
    await press("Enter");
    await browser("wait", "--fn", "qa.profileWrites.length===1");
    await evaluate("qa.navigate('/login')");
    await browser("wait", ".profile-dialog .profile-leave-status");
    await evaluate("qa.profileRelease[0](" + status + ")");
    if (status === 503) {
      await browser("wait", "--fn", "qa.profileReads.length===1");
      await evaluate("qa.profileReads[0](401)");
    }
    await browser(
      "wait",
      "--fn",
      "!!document.querySelector('.profile-discard')||document.querySelector('.profile-summary h2')?.textContent==='法语学习者'",
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.profile-summary h2').textContent",
      ),
      "法语学习者",
    );
    assert.equal(
      await evaluate(
        "document.querySelectorAll('.profile-dialog[open]').length",
      ),
      0,
    );
    assert.equal(
      await evaluate("document.querySelectorAll('.profile-edit').length"),
      0,
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.profile-summary').textContent.includes('a@example.test')",
      ),
      false,
    );
    assert.equal(
      await evaluate("qa.profileReads.length"),
      status === 503 ? 1 : 0,
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.toast [role=status]').textContent",
      ),
      "登录已过期，请重新登录。",
    );
    await browser("focus", 'a.primary[href="/login"]');
    await press("Enter");
    await browser("wait", "--fn", "qa.route==='/login'");
    assert.equal(await evaluate("qa.profileWrites.length"), 1);
  }
});

test("failed recovery reads retain the profile draft and let the server resolve the old version on explicit retry", async () => {
  await open("profile");
  await browser("focus", 'button[aria-label="编辑个人资料与学习目标"]');
  await press("Enter");
  await browser("fill", ".profile-dialog input", "Retained draft");
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===1");
  await evaluate("qa.navigate('/login');qa.profileRelease[0](503)");
  await browser("wait", "--fn", "qa.profileReads.length===1");
  await evaluate("qa.profileReads[0](503)");
  await browser("wait", ".profile-discard");
  assert.equal(
    await evaluate("document.querySelector('.profile-summary h2').textContent"),
    "Alice",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(await evaluate("qa.profileWrites.length"), 1);
  await press("Escape");
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Retained draft",
  );
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===2");
  assert.equal(await evaluate("qa.profileWrites[1].version"), 1);
  await evaluate("qa.profileRelease[1](409)");
  await browser("wait", "--fn", "qa.profileReads.length===2");
  await evaluate(
    "qa.profileReads[1]({id:'account-a',email:'a@example.test',displayName:'Another device',role:'learner',version:7,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
  );
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.profile-dialog button[type=submit]')?.getAttribute('aria-busy')==='false'",
  );
  assert.equal(
    await evaluate("document.activeElement.matches('.error-message')"),
    true,
  );
  assert.equal(
    await evaluate("document.querySelector('.profile-dialog input').value"),
    "Retained draft",
  );
  assert.equal(await evaluate("qa.profileWrites.length"), 2);
  await browser("focus", ".profile-dialog button[type=submit]");
  await press("Enter");
  await browser("wait", "--fn", "qa.profileWrites.length===3");
  assert.equal(await evaluate("qa.profileWrites[2].version"), 7);
  await evaluate(
    "qa.profileRelease[2]({id:'account-a',email:'a@example.test',displayName:'Retained draft',role:'learner',version:8,settings:{timeZone:'Asia/Shanghai',weeklyDays:5,dailyMinutes:10,showTranslation:false,speechRate:1}})",
  );
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.profile-dialog[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(await evaluate("qa.profileWrites.length"), 3);
});
