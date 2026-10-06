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

test("learning navigation keeps the exact pending request across leaving and returning", async () => {
  await open("session");
  await browser("set", "viewport", "320", "700");
  await browser("focus", ".speaker");
  await press("Enter");
  assert.equal(
    await evaluate("document.activeElement.matches('.speaker')"),
    true,
  );
  await browser("focus", ".learning-actions .primary");
  await press("Enter");
  await browser("wait", "--fn", "qa.learningWrites.length===1");
  const original = await evaluate("qa.learningWrites[0]");
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    true,
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  assert.equal(
    await evaluate(
      "(()=>{const r=document.querySelector('.pending-navigation[open]').getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight})()",
    ),
    true,
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "这次提交尚未确认",
  );
  await press("Escape");
  assert.equal(
    await evaluate(
      "document.activeElement.matches('.learning-step-heading h2')",
    ),
    true,
  );
  await evaluate("qa.learningRelease[0](503)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.learning-actions .primary')?.textContent.includes('重试保存')",
  );
  await evaluate("qa.navigate(-1)");
  await browser("wait", ".pending-navigation[open]");
  await browser("focus", ".pending-navigation .text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.route==='/previous'");
  assert.equal(await evaluate("qa.learningWrites.length"), 1);
  assert.deepEqual(
    await evaluate(
      "JSON.parse(sessionStorage.getItem('brioche.learning.v1:qa-account:qa-session:1:pending')).body",
    ),
    original,
  );
  await evaluate("qa.navigate('/')");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.learning-actions .primary')?.textContent.includes('重试保存')",
  );
  await browser("focus", ".learning-actions .primary");
  await press("Enter");
  await browser("wait", "--fn", "qa.learningWrites.length===2");
  assert.deepEqual(await evaluate("qa.learningWrites[1]"), original);
  await evaluate(
    "qa.learningRelease[1]({id:'qa-session',lessonId:'reading-protocol',revision:1,version:2,lastStepId:'read',confirmedStepIds:['read'],hintedExerciseIds:[],attempts:[],completedAt:null,firstCompletedAt:null})",
  );
  await browser(
    "wait",
    "--fn",
    "!sessionStorage.getItem('brioche.learning.v1:qa-account:qa-session:1:pending')",
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", "--fn", "qa.route==='/login'");

  await open("session");
  await browser("focus", ".learning-actions .primary");
  await press("Enter");
  await browser("wait", "--fn", "qa.learningWrites.length===1");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await evaluate(
    "qa.learningRelease[0]({id:'qa-session',lessonId:'reading-protocol',revision:1,version:2,lastStepId:'read',confirmedStepIds:['read'],hintedExerciseIds:[],attempts:[],completedAt:null,firstCompletedAt:null})",
  );
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.pending-navigation[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(
    await evaluate(
      "document.activeElement.matches('.learning-step-heading h2')",
    ),
    true,
  );
  assert.equal(await evaluate("qa.learningWrites.length"), 1);
  await evaluate("qa.navigate('/login')");
  await browser("wait", "--fn", "qa.route==='/login'");
});

test("review navigation preserves the original rating and does not treat queue reads as pending writes", async () => {
  await open("reviews");
  await browser("focus", ".review-flashcard");
  await press("Enter");
  await browser("focus", '.review-ratings button[data-grade="2"]');
  await press("Enter");
  await browser("wait", "--fn", "qa.reviewWrites.length===1");
  const original = await evaluate("qa.reviewWrites[0]");
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    true,
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await press("Escape");
  assert.equal(
    await evaluate(
      "document.activeElement.matches('.review-session-header h1')",
    ),
    true,
  );
  await evaluate("qa.reviewRelease[0](503)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.empty-state h2')?.textContent==='确认上次复习'",
  );
  await evaluate("qa.navigate(-1)");
  await browser("wait", ".pending-navigation[open]");
  await browser("focus", ".pending-navigation .text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.route==='/previous'");
  assert.deepEqual(
    await evaluate(
      "JSON.parse(sessionStorage.getItem('brioche.learning.v1:qa-account:reviews:1:pending')).body",
    ),
    original,
  );
  await evaluate("qa.navigate('/')");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.empty-state h2')?.textContent==='确认上次复习'",
  );
  await browser("focus", ".review-page > button.primary");
  await press("Enter");
  await browser("wait", "--fn", "qa.reviewWrites.length===2");
  assert.deepEqual(await evaluate("qa.reviewWrites[1]"), original);
  await evaluate(
    "qa.reviewRelease[1]({card:{id:'qa-card',knowledgeId:'qa-word',sourceLessonId:'reading-protocol',sourceRevision:1,vocabulary:{id:'qa-word',lemma:'bonjour',partOfSpeech:'phrase',gender:null,meaningZh:'你好',noteZh:'日常问候'},stage:1,dueAt:'2026-10-07T00:00:00Z',version:2,suspended:false},reviewedAt:'2026-10-06T00:00:00Z',timeZone:'Asia/Shanghai'})",
  );
  await browser("wait", "--fn", "qa.queueReads.length===1");
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    false,
  );
  assert.equal(
    await evaluate(
      "sessionStorage.getItem('brioche.learning.v1:qa-account:reviews:1:pending')",
    ),
    null,
  );
  await evaluate("qa.navigate('/login')");
  await browser(
    "wait",
    "--fn",
    "qa.route==='/login'||!!document.querySelector('.pending-navigation[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/login");
  assert.equal(await evaluate("qa.reviewWrites.length"), 2);
  await evaluate(
    "qa.queueReads[0]({items:[],dueCount:0,nextDueAt:null,localDate:'2026-10-06',timeZone:'Asia/Shanghai'})",
  );
  assert.equal(await evaluate("qa.route"), "/login");
});

test("revoked reviews close the leave prompt, focus recovery, and never revive an unavailable card", async () => {
  await open("reviews");
  await browser("focus", ".review-flashcard");
  await press("Enter");
  const cancellations = await evaluate("qa.cancellations");
  await browser("focus", '.review-ratings button[data-grade="0"]');
  await press("Enter");
  await browser("wait", "--fn", "qa.reviewWrites.length===1");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await evaluate("qa.reviewRelease[0](410)");
  await browser(
    "wait",
    "--fn",
    "qa.queueReads.length===1 && !document.querySelector('.pending-navigation[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.review-flashcard,.review-context,.review-ratings').length",
    ),
    0,
  );
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "需要确认复习队列",
  );
  assert.equal(await evaluate("qa.cancellations>" + cancellations), true);
  assert.equal(
    await evaluate(
      "sessionStorage.getItem('brioche.learning.v1:qa-account:reviews:1:pending')",
    ),
    null,
  );
  await evaluate("qa.queueReads[0](503)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.review-page > button.text-button')?.getAttribute('aria-busy')==='false'",
  );
  await browser("focus", ".review-page > button.text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.queueReads.length===2");
  await evaluate("qa.queueReads[1](qa.reviewFixture)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.review-page > button.text-button')?.getAttribute('aria-busy')==='false'",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.review-flashcard,.review-context,.review-ratings').length",
    ),
    0,
  );
  await browser("focus", ".review-page > button.text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.queueReads.length===3");
  await evaluate(
    "qa.queueReads[2]({items:[],dueCount:0,nextDueAt:null,localDate:'2026-10-06',timeZone:'Asia/Shanghai'})",
  );
  await browser("wait", "--fn", "!!document.querySelector('.review-summary')");
  assert.equal(
    await evaluate(
      "document.activeElement.matches('.review-session-header h1')",
    ),
    true,
  );
  assert.equal(await evaluate("qa.reviewWrites.length"), 1);
});

test("collapsed library cards retain all pending writes and retry the exact original after navigation", async () => {
  await open("library");
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  await browser("focus", ".bookmark-action");
  await press("Enter");
  await browser(
    "focus",
    ".knowledge-actions:not(:has(.bookmark-action)) button",
  );
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===2");
  const original = await evaluate("qa.ownedWrites[0]");
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  assert.equal(
    await evaluate("!!document.querySelector('.library-entry-body')"),
    false,
  );
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    true,
  );
  // One acknowledged request cannot clear another pending operation.
  await evaluate("qa.ownedRelease[1](qa.reviewFixture.items[0])");
  await browser(
    "wait",
    "--fn",
    "Object.keys(sessionStorage).filter(k=>k.includes(':owned:')).length===1",
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await press("Escape");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "我的表达",
  );
  await evaluate("qa.ownedRelease[0](503)");
  await evaluate("qa.navigate(-1)");
  await browser("wait", ".pending-navigation[open]");
  await browser("focus", ".pending-navigation .text-button");
  await press("Enter");
  await browser("wait", "--fn", "qa.route==='/previous'");
  assert.deepEqual(
    await evaluate(
      "JSON.parse(sessionStorage.getItem(Object.keys(sessionStorage).find(k=>k.includes(':owned:'))))",
    ),
    { ...original, method: "PUT" },
  );
  await evaluate("qa.navigate('/')");
  await browser("wait", ".library-entry-heading");
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "!!document.querySelector('.bookmark-action:disabled') && !!Array.from(document.querySelectorAll('button')).find(b=>b.textContent==='重试保存')",
  );
  await browser(
    "focus",
    ".knowledge-actions:has(.bookmark-action) button:last-child",
  );
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===3");
  assert.deepEqual(await evaluate("qa.ownedWrites[2]"), original);
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  // A late acknowledgement clears storage even when its writer is unmounted.
  await evaluate(
    "qa.ownedRelease[2]({...qa.savedFixture,saved:false,version:2})",
  );
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.pending-navigation[open]') && !Object.keys(sessionStorage).some(k=>k.includes(':owned:'))",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "我的表达",
  );
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    false,
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", "--fn", "qa.route==='/login'");
  assert.equal(await evaluate("qa.ownedWrites.length"), 3);
});

test("pending save recovery keeps keyboard focus, prevents duplicate requests, and observes external confirmation", async () => {
  await open("pending");
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===2",
  );
  await browser("focus", ".library-entry:first-child button");
  await press("Enter");
  await press("Enter");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===1");
  assert.deepEqual(
    await evaluate(
      "({tag:document.activeElement.tagName,busy:document.activeElement.getAttribute('aria-busy'),count:qa.ownedWrites.length})",
    ),
    { tag: "BUTTON", busy: "true", count: 1 },
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await press("Escape");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "未确认保存",
  );
  await evaluate("qa.ownedRelease[0](503)");
  await browser("wait", ".error-message");
  await browser("focus", ".library-entry:first-child button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===2");
  assert.deepEqual(
    await evaluate("qa.ownedWrites[1]"),
    await evaluate("qa.ownedWrites[0]"),
  );
  await evaluate("qa.ownedRelease[1](qa.savedFixture)");
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===1",
  );
  assert.equal(
    await evaluate("document.activeElement.matches('.library-entry button')"),
    true,
  );
  // A previously mounted writer can confirm while this recovery list is open.
  await evaluate(
    "qa.confirmExternal(qa.ownedWrites[0].path.endsWith('0') ? 1 : 0)",
  );
  assert.deepEqual(
    await evaluate(
      "({rows:document.querySelectorAll('.library-entry').length, keys:Object.keys(sessionStorage).filter(k=>k.includes(':qa-account:')),focus:document.activeElement.textContent})",
    ),
    { rows: 0, keys: [], focus: "没有待确认的保存。" },
  );
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===0",
  );
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "没有待确认的保存。",
  );
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    false,
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", "--fn", "qa.route==='/login'");
  assert.equal(await evaluate("qa.ownedWrites.length"), 2);
});

test("an old account recovery response cannot replace the new account pending list or lock its writes", async () => {
  await open("pending");
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===2",
  );
  await browser("focus", ".library-entry:first-child button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===1");
  await evaluate("qa.changeUser()");
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===1",
  );
  await browser("focus", ".library-entry button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===2");
  assert.equal(
    await evaluate("qa.ownedWrites[1].path"),
    "/api/v1/me/saved-items/pending-word-2",
  );
  await evaluate("qa.ownedRelease[0](qa.savedFixture)");
  assert.deepEqual(
    await evaluate(
      "({rows:document.querySelectorAll('.library-entry').length,busy:document.querySelector('.library-entry button')?.getAttribute('aria-busy')})",
    ),
    { rows: 1, busy: "true" },
  );
  await evaluate("qa.ownedRelease[1](qa.savedFixture)");
  await browser(
    "wait",
    "--fn",
    "document.querySelectorAll('.library-entry').length===0",
  );
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "没有待确认的保存。",
  );
});

test("unavailable learning retains independent expression writes and only releases navigation after all confirmations", async () => {
  await open("session-revoked");
  await browser("focus", ".learning-actions button.primary");
  await press("Enter");
  await browser("wait", "--fn", "qa.learningWrites.length===1");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await evaluate("qa.learningRelease[0](410)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.lesson-header h1')?.textContent==='课程已撤回'",
  );
  assert.equal(
    await evaluate("!!document.querySelector('.pending-navigation[open]')"),
    true,
  );
  assert.equal(await evaluate("qa.route"), "/");
  await press("Escape");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "课程已撤回",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.learning-stage,.learning-actions').length",
    ),
    0,
  );
  assert.equal(
    await evaluate(
      "Object.keys(sessionStorage).filter(k=>k.includes(':qa-account:owned:')).length",
    ),
    2,
  );
  await evaluate("qa.navigate('/login')");
  await browser(
    "wait",
    "--fn",
    "qa.route==='/login'||!!document.querySelector('.pending-navigation[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  await press("Escape");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "课程已撤回",
  );
  await evaluate("qa.confirmExternal(0)");
  await evaluate("qa.navigate('/login')");
  await browser("wait", ".pending-navigation[open]");
  await evaluate("qa.confirmExternal(1)");
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.pending-navigation[open]')",
  );
  assert.equal(await evaluate("qa.route"), "/");
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "课程已撤回",
  );
  assert.equal(
    await evaluate(
      "(()=>{const e=new Event('beforeunload',{cancelable:true});window.dispatchEvent(e);return e.defaultPrevented})()",
    ),
    false,
  );
  await evaluate("qa.navigate('/login')");
  await browser("wait", "--fn", "qa.route==='/login'");
  assert.equal(await evaluate("qa.learningWrites.length"), 1);
  assert.equal(await evaluate("qa.ownedWrites.length"), 0);
});

test("optional fill-blank hints omit empty actions and reveal meaningful hints once", async () => {
  await open("text-no-hint");
  assert.equal(
    await evaluate("document.querySelectorAll('.practice-hint').length"),
    0,
  );
  await browser("focus", ".exercise-sheet input");
  await browser("keyboard", "inserttext", "bonjour");
  await browser("focus", ".exercise-sheet button.primary");
  await press("Enter");
  assert.deepEqual(await evaluate("qa.textAnswers[0]"), {
    kind: "text",
    text: "bonjour",
  });
  assert.equal(await evaluate("qa.hintRequests"), 0);
  await open("text-limit");
  await browser("focus", ".exercise-sheet .practice-hint");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('.exercise-sheet .profile-note')?.textContent==='边界测试'",
  );
  assert.equal(await evaluate("qa.hintRequests"), 1);
  assert.equal(
    await evaluate("document.activeElement.textContent"),
    "边界测试",
  );
  assert.equal(
    await evaluate("document.querySelectorAll('.practice-hint').length"),
    0,
  );
});

test("native fill-blank input bounds UTF-16 units and submits the exact text without removing accents", async () => {
  await open("text-limit");
  await browser("focus", ".exercise-sheet input");
  assert.equal(await evaluate("document.activeElement.maxLength"), 1024);
  await browser("keyboard", "inserttext", "a".repeat(1025));
  assert.equal(await evaluate("document.activeElement.value.length"), 1024);
  await browser("focus", ".exercise-sheet button.primary");
  await press("Enter");
  assert.deepEqual(await evaluate("qa.textAnswers[0]"), {
    kind: "text",
    text: "a".repeat(1024),
  });
  await browser("focus", ".exercise-sheet input");
  await press("Control+a");
  await browser("keyboard", "inserttext", "😀".repeat(513));
  assert.equal(await evaluate("document.activeElement.value.length"), 1024);
  await browser("focus", ".exercise-sheet button.primary");
  await press("Enter");
  assert.deepEqual(await evaluate("qa.textAnswers[1]"), {
    kind: "text",
    text: "😀".repeat(512),
  });
  await browser("focus", ".exercise-sheet input");
  await press("Control+a");
  await browser("keyboard", "inserttext", "e\u0301");
  await browser("focus", ".exercise-sheet button.primary");
  await press("Enter");
  assert.deepEqual(await evaluate("qa.textAnswers[2]"), {
    kind: "text",
    text: "e\u0301",
  });
});

test("responsive content keeps long words and controls inside the viewport", async () => {
  const selectors =
    ".lesson-header h1,.sentence .word,.practice-option span,.practice-sentence,.order-bank button,.order-answer button,.learning-step-heading h2,.review-expression,.profile-summary h2,.library-entry-heading,.library-entry-heading strong,.lesson-label small,.lesson-label b,.resume-learning strong,.review .fr,.course-search-field";
  for (const kind of [
    "reading",
    "session",
    "reviews",
    "profile",
    "library",
    "home",
    "courses",
    "text-limit",
  ]) {
    await open(kind + "&stress=1");
    await browser(
      "wait",
      kind === "profile"
        ? ".profile-summary h2"
        : kind === "reviews"
          ? ".review-expression"
          : kind === "text-limit"
            ? ".order-bank button"
            : kind === "library"
              ? ".library-entry-heading"
              : kind === "home" || kind === "courses"
                ? ".lesson-row"
                : ".sentence .word",
    );
    for (const width of [320, 390, 768, 1440]) {
      await browser("set", "viewport", String(width), "844");
      await browser(
        "wait",
        "--fn",
        "document.getAnimations().every(a=>a.playState!=='running'||!Number.isFinite(a.effect.getComputedTiming().endTime))",
      );
      const overflow = await evaluate(`(() => {
        const nodes = [...document.querySelectorAll(${JSON.stringify(selectors)})];
        return nodes.filter(e => e.getClientRects().length).flatMap(e => {
          const r = e.getBoundingClientRect();
          const range = document.createRange();
          range.selectNodeContents(e);
          const text = range.getBoundingClientRect();
          const parent = e.parentElement.getBoundingClientRect();
          return Math.min(r.left,text.left) < Math.max(0,parent.left) - 1 || Math.max(r.right,text.right) > Math.min(innerWidth,parent.right) + 1 ? [{text:e.textContent,left:r.left,right:r.right,textRight:text.right,parentLeft:parent.left,parentRight:parent.right}] : [];
        });
      })()`);
      assert.deepEqual(overflow, [], kind + " at " + width + "px");
      const documentBounds = await evaluate(
        `({width:document.documentElement.scrollWidth,viewport:innerWidth,overflow:[...document.querySelectorAll('main *')].filter(e=>e.getClientRects().length && e.getBoundingClientRect().right > innerWidth + 1).map(e=>({tag:e.tagName,cls:e.className,text:e.textContent.slice(0,100),right:e.getBoundingClientRect().right})).slice(0,12)})`,
      );
      assert.ok(
        documentBounds.width <= documentBounds.viewport,
        kind +
          " document at " +
          width +
          "px: " +
          JSON.stringify(documentBounds),
      );
      if (kind === "reading") {
        await browser("focus", "[aria-selected=true]");
        await press("ArrowRight");
        assert.equal(
          await evaluate("document.querySelectorAll('.speaker').length"),
          0,
        );
        assert.equal(
          await evaluate(
            "[...document.querySelectorAll('.sentence .word')].every(e=>{const r=e.getBoundingClientRect(),p=e.parentElement.getBoundingClientRect();return r.left>=p.left-1&&r.right<=p.right+1&&r.right<=innerWidth+1})",
          ),
          true,
          "article at " + width + "px",
        );
        await press("Home");
      }
    }
    if (kind === "reading") {
      await browser("focus", ".sentence .word");
      await press("Enter");
      await browser("wait", "--fn", "qa.playback==='playing'");
      assert.equal(
        await evaluate("qa.spoken.at(-1)"),
        "anticonstitutionnellement",
      );
    }
    if (kind === "library") {
      await browser("focus", ".library-entry-heading");
      await press("Enter");
      await browser("wait", ".library-entry-body");
      await browser("wait", "--fn", "qa.spoken.length>0");
      assert.equal(
        await evaluate("qa.spoken.at(-1)"),
        "anticonstitutionnellement",
      );
      assert.equal(
        await evaluate("document.activeElement.className"),
        "library-entry-heading",
      );
      await press("Enter");
      assert.equal(
        await evaluate(
          "document.querySelectorAll('.library-entry-body').length",
        ),
        0,
      );
    }
  }
  await browser("set", "viewport", "320", "844");
  await browser("focus", ".order-bank button");
  await press("Enter");
  assert.equal(
    await evaluate(
      "document.querySelector('.order-answer button').textContent",
    ),
    "anticonstitutionnellement",
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.order-answer button').getBoundingClientRect().right<=innerWidth",
    ),
    true,
  );
});

test("reduced motion disables card expansion animations", async () => {
  await browser("set", "media", "light", "reduced-motion");
  try {
    await open("reviews");
    assert.equal(
      await evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches"),
      true,
    );
    await browser("focus", ".review-flashcard");
    await press("Enter");
    await browser("wait", ".review-ratings");
    assert.equal(await evaluate("document.getAnimations().length"), 0);
    assert.equal(
      await evaluate(
        "getComputedStyle(document.querySelector('.review-flashcard')).transitionDuration",
      ),
      "0s",
    );
    await open("home");
    await browser("focus", ".home-review button.review");
    await press("Enter");
    assert.equal(
      await evaluate("document.activeElement.getAttribute('aria-expanded')"),
      "true",
    );
    assert.equal(await evaluate("document.getAnimations().length"), 0);
  } finally {
    await browser("set", "media", "light");
  }
});

test("managed review recovery preserves keyboard focus and deduplicates reads and retries", async () => {
  await open("managed-library");
  await browser("set", "viewport", "320", "844");
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  await browser("focus", ".library-entry-body button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===1");
  await evaluate("qa.ownedRelease[0](503)");
  await browser("wait", "--text", "重试保存");
  await browser("focus", ".library-entry-body button:nth-of-type(2)");
  await press("Enter");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===2");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "重试保存",
  );
  assert.equal(
    await evaluate(
      "JSON.stringify(qa.ownedWrites[0])===JSON.stringify(qa.ownedWrites[1])",
    ),
    true,
  );
  await evaluate(
    "qa.ownedRelease[1]({...qa.reviewFixture.items[0],version:2,suspended:true})",
  );
  await browser("wait", "--text", "恢复复习");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "恢复复习",
  );
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===3");
  await evaluate("qa.ownedRelease[2](409)");
  await browser("wait", "--fn", "qa.cardReads.length===1");
  await evaluate("qa.cardReads[0](503)");
  await browser("wait", "--text", "重新读取记录");
  await browser("focus", ".library-entry-body button:nth-of-type(2)");
  await press("Enter");
  await press("Enter");
  await press("Enter");
  await browser("wait", "--fn", "qa.cardReads.length>=2");
  assert.equal(await evaluate("qa.cardReads.length"), 2);
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "正在读取",
  );
  await evaluate("qa.cardReads[1](503)");
  await browser("wait", "--text", "重新读取记录");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "重新读取记录",
  );
  await press("Enter");
  await browser("wait", "--fn", "qa.cardReads.length===3");
  await evaluate(
    "qa.cardReads[2]({...qa.reviewFixture.items[0],version:3,suspended:false})",
  );
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.library-entry-body')?.textContent.includes('重新读取记录')",
  );
  assert.equal(
    await evaluate("document.activeElement.className"),
    "library-entry-heading",
  );
  assert.equal(await evaluate("qa.ownedWrites.length"), 3);
  await browser("focus", ".library-entry-body button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===4");
  await evaluate("qa.ownedRelease[3](409)");
  await browser("wait", "--fn", "qa.cardReads.length===4");
  await evaluate("qa.cardReads[3](503)");
  await browser("wait", "--text", "重新读取记录");
  await browser("focus", ".library-entry-body button:nth-of-type(2)");
  await press("Enter");
  await browser("wait", "--fn", "qa.cardReads.length===5");
  await browser("focus", ".library-entry-body a");
  await evaluate(
    "qa.cardReads[4]({...qa.reviewFixture.items[0],version:4,suspended:false})",
  );
  await browser(
    "wait",
    "--fn",
    "!document.querySelector('.library-entry-body')?.textContent.includes('重新读取记录')",
  );
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "回看来源课程",
  );
  await open("managed-library");
  await browser("focus", ".library-entry-heading");
  await press("Enter");
  await browser("focus", ".library-entry-body button");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===1");
  await evaluate("qa.ownedRelease[0](503)");
  await browser("wait", "--text", "重试保存");
  await browser("focus", ".library-entry-body button:nth-of-type(2)");
  await press("Enter");
  await browser("wait", "--fn", "qa.ownedWrites.length===2");
  await evaluate("qa.ownedRelease[1](422)");
  await browser(
    "wait",
    "--fn",
    "document.activeElement.getAttribute('role')==='alert'",
  );
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "请检查填写的信息。",
  );
  assert.equal(await evaluate("qa.cardReads.length"), 0);
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.library-entry-body button').length",
    ),
    1,
  );
});

test("homepage omits an empty expression card while keeping review and continuation entries", async () => {
  await open("home-no-expression");
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.home-review button.review').length",
    ),
    0,
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.resume-learning').getAttribute('href')",
    ),
    "/learning/qa-home-session",
  );
  await browser("focus", ".home-review-link");
  await press("Enter");
  await browser("wait", "--fn", "qa.route==='/reviews'");
});

test("homepage rapid card toggles keep a single height animation and immediate expanded state", async () => {
  await open("home&hold-animation=1");
  await browser("focus", ".home-review button.review");
  await press("Enter");
  assert.equal(
    await evaluate("document.activeElement.getAttribute('aria-expanded')"),
    "true",
  );
  await press("Space");
  assert.equal(
    await evaluate("document.activeElement.getAttribute('aria-expanded')"),
    "false",
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.home-review button.review').getAnimations().filter(a=>a.effect.target===document.querySelector('.home-review button.review')).length<=1",
    ),
    true,
  );
});

test("course search retains focus while pending and shows explicit empty results and reset", async () => {
  await open("courses");
  await browser("focus", "#course-query");
  await browser("keyboard", "inserttext", "introuvable");
  await browser("focus", ".course-search button");
  await press("Enter");
  await browser("wait", "--fn", "qa.catalogReads.length===1");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "搜索",
  );
  await press("Enter");
  assert.equal(await evaluate("qa.catalogReads.length"), 1);
  assert.equal(
    await evaluate(
      "document.querySelector('.courses-page [role=status]').textContent",
    ),
    "正在查找…",
  );
  await evaluate(
    "qa.catalogReads[0].release({levels:[],developmentFixture:false})",
  );
  await browser("wait", "--text", "还没有找到这个场景");
  assert.equal(
    await evaluate("document.querySelector('#course-query').value"),
    "introuvable",
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.courses-page [role=status]').textContent",
    ),
    "找到 0 堂课程",
  );
  await browser("focus", ".empty-state a");
  await press("Enter");
  await browser("wait", ".lesson-row");
  assert.equal(
    await evaluate("document.querySelector('#course-query').value"),
    "",
  );
  await browser("focus", "#course-query");
  await browser("keyboard", "inserttext", "bonjour");
  await press("Enter");
  await browser("wait", "--fn", "qa.catalogReads.length===2");
  await evaluate("qa.catalogReads[1].release(qa.catalogFixture)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('[role=status]').textContent==='找到 1 堂课程'",
  );
  assert.equal(await evaluate("document.activeElement.id"), "course-query");
  await press("Enter");
  await browser("wait", "--fn", "qa.catalogReads.length===3");
  await browser("keyboard", "inserttext", " demain");
  await evaluate("qa.catalogReads[2].release(qa.catalogFixture)");
  await browser(
    "wait",
    "--fn",
    "document.querySelector('[role=status]').textContent==='找到 1 堂课程'",
  );
  assert.equal(await evaluate("document.activeElement.id"), "course-query");
  assert.equal(
    await evaluate("document.querySelector('#course-query').value"),
    "bonjour demain",
  );
});

test("review history pagination focuses results, distinguishes empty continuations, and returns to latest records", async () => {
  await open("history");
  await browser("wait", ".review-result-list li");
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.review-result-list li').length",
    ),
    2,
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.result-expression')[1].textContent",
    ),
    "来源内容已撤回",
  );
  assert.equal(
    await evaluate("document.querySelectorAll('.result-expression')[1].lang"),
    "zh-CN",
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.review-result-list small').textContent.includes('2026/10/6 07:30')",
    ),
    true,
  );
  await browser("focus", "a[href*='cursor=']");
  await press("Enter");
  await browser(
    "wait",
    "--fn",
    "new URLSearchParams(qa.search).get('cursor')==='older/qa?+'",
  );
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "复习记录",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.review-result-list li').length",
    ),
    1,
  );
  assert.equal(
    await evaluate(
      "document.querySelector('.review-result-list small').textContent.includes('还不熟')",
    ),
    true,
  );
  await browser("focus", "a[href*='cursor=end']");
  await press("Enter");
  await browser("wait", "--fn", "qa.search==='?cursor=end'");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "复习记录",
  );
  assert.equal(
    await evaluate("document.querySelector('.profile-note').textContent"),
    "这一页没有更早的复习记录。",
  );
  await browser("focus", "a[href='/review-history']");
  await press("Enter");
  await browser("wait", ".review-result-list li");
  assert.equal(await evaluate("qa.search"), "");
  assert.equal(
    await evaluate("document.activeElement.textContent.trim()"),
    "复习记录",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('.review-result-list li').length",
    ),
    2,
  );
  await open("history-empty");
  await browser("wait", ".profile-note");
  assert.equal(
    await evaluate("document.querySelector('.profile-note').textContent"),
    "完成一次复习后，记录会显示在这里。",
  );
  assert.equal(
    await evaluate(
      "document.querySelectorAll('a[href*=cursor],a[href=\"/review-history\"]').length",
    ),
    0,
  );
});

test("review history keeps long expressions and recorded dates inside four viewport widths", async () => {
  await open("history&stress=1");
  await browser("wait", ".review-result-list li");
  for (const width of [320, 390, 768, 1440]) {
    await browser("set", "viewport", String(width), "844");
    await browser(
      "wait",
      "--fn",
      "document.getAnimations().every(a=>a.playState!=='running'||!Number.isFinite(a.effect.getComputedTiming().endTime))",
    );
    const overflow = await evaluate(`(() => {
      return [...document.querySelectorAll('.result-expression,.review-result-list small,.review-result-grade')].flatMap(e => {
        const r=e.getBoundingClientRect(),p=e.parentElement.getBoundingClientRect();
        const range=document.createRange();range.selectNodeContents(e);const t=range.getBoundingClientRect();
        return Math.min(r.left,t.left)<Math.max(0,p.left)-1 || Math.max(r.right,t.right)>Math.min(innerWidth,p.right)+1 ? [{text:e.textContent,right:r.right,textRight:t.right,parentRight:p.right}] : [];
      });
    })()`);
    assert.deepEqual(overflow, [], "history at " + width + "px");
    assert.equal(
      await evaluate("document.documentElement.scrollWidth<=innerWidth"),
      true,
    );
    assert.equal(
      await evaluate(
        "document.querySelector('.result-expression').textContent",
      ),
      "anticonstitutionnellement",
    );
  }
});
