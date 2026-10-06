import { useState } from "react";
import { createRoot } from "react-dom/client";
import { createMemoryRouter, RouterProvider } from "react-router";
import { StartLearning } from "../app/components/start-learning";
import { LearningProvider, useLearning } from "../app/components/learning";
import Lesson from "../app/routes/lesson";
import { lesson } from "./lesson";
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
const reading = new URL(location.href).searchParams.get("case") === "reading";
const router = createMemoryRouter([
  {
    path: "/",
    element: reading ? (
      <LearningProvider>
        <ReadingHarness />
      </LearningProvider>
    ) : (
      <StartHarness />
    ),
  },
  { path: "/learning/:id", element: <h1>已进入学习</h1> },
  { path: "/login", element: <h1>登录入口</h1> },
]);
router.subscribe((state) => {
  qa.route = state.location.pathname;
  qa.search = state.location.search;
});
createRoot(document.getElementById("root")!).render(
  <RouterProvider router={router} />,
);
qa.ready = true;
