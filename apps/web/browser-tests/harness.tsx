import { useState } from "react";
import { createRoot } from "react-dom/client";
import { createMemoryRouter, RouterProvider } from "react-router";
import { StartLearning } from "../app/components/start-learning";
import { LearningProvider, useLearning } from "../app/components/learning";
import Lesson from "../app/routes/lesson";
import { lesson } from "./lesson";
import { ChoiceDialog } from "../app/components/choice-dialog";
import Profile from "../app/routes/profile";
import type { UserProfile } from "@brioche/contracts/UserProfile";
import "../app/styles/app.css";

const qa = {
  ready: false,
  writes: [] as { lessonId: string; idempotencyKey: string }[],
  release: [] as ((status: number) => void)[],
  spoken: [] as string[],
  lastUtterance: null as ControlledUtterance | null,
  oldUtterance: null as ControlledUtterance | null,
  playback: "idle",
  playbackId: null as string | null,
  route: "/",
  search: "",
  changeUser: null as (() => void) | null,
  profileWrites: [] as Record<string, unknown>[],
  profileRelease: [] as ((profile: UserProfile) => void)[],
  navigate: null as ((destination: string | number) => void) | null,
};
Object.assign(window, { qa });
class ControlledUtterance extends EventTarget {
  onstart: (() => void) | null = null;
  onend: (() => void) | null = null;
  onerror: ((event: { error: string }) => void) | null = null;
  constructor(public text: string) {
    super();
  }
}
Object.defineProperty(window, "SpeechSynthesisUtterance", {
  configurable: true,
  value: ControlledUtterance,
});
Object.defineProperty(window, "speechSynthesis", {
  configurable: true,
  value: {
    getVoices: () => [{ lang: "fr-FR", name: "controlled" }],
    speak: (utterance: ControlledUtterance) => {
      qa.spoken.push(utterance.text);
      qa.lastUtterance = utterance;
      utterance.onstart?.();
    },
    cancel() {},
    pause() {},
    resume() {},
    addEventListener() {},
    removeEventListener() {},
  },
});
const originalFetch = window.fetch;
window.fetch = async (input, init) => {
  if (String(input) === "/api/v1/auth/csrf")
    return Response.json({ csrfToken: "controlled" });
  if (String(input) === "/api/v1/me/settings") {
    const index = qa.profileWrites.push(JSON.parse(String(init?.body))) - 1;
    return new Promise<Response>((resolve) => {
      qa.profileRelease[index] = (profile) => resolve(Response.json(profile));
    });
  }
  if (String(input) !== "/api/v1/learning-sessions")
    return originalFetch(input, init);
  const body = JSON.parse(String(init?.body)) as {
    lessonId: string;
    idempotencyKey: string;
  };
  const index = qa.writes.push(body) - 1;
  return new Promise<Response>((resolve) => {
    qa.release[index] = (status) =>
      resolve(
        status === 200
          ? Response.json({ progress: { id: "session-" + body.lessonId } })
          : new Response("", { status }),
      );
  });
};
function StartHarness() {
  const [lessonId, setLessonId] = useState("course-a");
  return (
    <main>
      <h1>{lessonId}</h1>
      <StartLearning lessonId={lessonId}>学习 {lessonId}</StartLearning>
      <button
        id="switch-course"
        onClick={() =>
          setLessonId(lessonId === "course-a" ? "course-b" : "course-a")
        }
      >
        切换推荐课程
      </button>
    </main>
  );
}
function ReadingHarness() {
  const { player } = useLearning();
  qa.playback = player.status;
  qa.playbackId = player.id;
  return (
    <div className="app">
      <main>
        <Lesson
          loaderData={{ lesson, demo: true }}
          params={{ lessonId: lesson.id }}
          matches={[
            {
              id: "root",
              params: {},
              pathname: "/",
              loaderData: { user: null, enabled: false },
              handle: undefined,
            },
            {
              id: "routes/lesson",
              params: { lessonId: lesson.id },
              pathname: "/",
              loaderData: { lesson, demo: true },
              handle: undefined,
            },
          ]}
        />
      </main>
    </div>
  );
}
function ChoicesHarness() {
  const [rate, setRate] = useState("1"),
    [zone, setZone] = useState("Asia/Shanghai");
  return (
    <main>
      <h1>个人设置</h1>
      <ChoiceDialog
        title="朗读速度"
        value={rate}
        onChange={setRate}
        choices={["0.75", "1", "1.25", "1.5"].map((value) => ({
          value,
          label: value + "×",
        }))}
      />
      <ChoiceDialog
        title="时区"
        value={zone}
        onChange={setZone}
        searchable
        choices={[
          { value: "Asia/Shanghai", label: "上海" },
          { value: "Europe/Paris", label: "巴黎" },
        ]}
      />
    </main>
  );
}
function ProfileHarness() {
  const [user, setUser] = useState<UserProfile>({
    id: "account-a",
    email: "a@example.test",
    displayName: "Alice",
    role: "learner",
    version: 1,
    settings: {
      timeZone: "Asia/Shanghai",
      weeklyDays: 5,
      dailyMinutes: 10,
      showTranslation: false,
      speechRate: 1,
    },
  });
  qa.changeUser = () =>
    setUser({
      ...user,
      id: "account-b",
      email: "b@example.test",
      displayName: "Bob",
      version: 10,
    });
  return (
    <LearningProvider user={user}>
      <main>
        <Profile />
      </main>
    </LearningProvider>
  );
}
const kind = new URL(location.href).searchParams.get("case");
const reading = kind === "reading";
const router = createMemoryRouter(
  [
    {
      path: "/",
      element: reading ? (
        <LearningProvider>
          <ReadingHarness />
        </LearningProvider>
      ) : kind === "choices" ? (
        <ChoicesHarness />
      ) : kind === "profile" ? (
        <ProfileHarness />
      ) : (
        <StartHarness />
      ),
    },
    { path: "/learning/:id", element: <h1>已进入学习</h1> },
    { path: "/login", element: <h1>登录入口</h1> },
    { path: "/previous", element: <h1>上一页</h1> },
  ],
  { initialEntries: ["/previous", "/"], initialIndex: 1 },
);
qa.navigate = (destination) => {
  if (typeof destination === "number") void router.navigate(destination);
  else void router.navigate(destination);
};
router.subscribe((state) => {
  qa.route = state.location.pathname;
  qa.search = state.location.search;
});
createRoot(document.getElementById("root")!).render(
  <RouterProvider router={router} />,
);
qa.ready = true;
