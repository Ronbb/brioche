import { useState } from "react";
import { createRoot } from "react-dom/client";
import { createMemoryRouter, RouterProvider } from "react-router";
import { StartLearning } from "../app/components/start-learning";
import { LearningProvider, useLearning } from "../app/components/learning";
import Lesson from "../app/routes/lesson";
import { lesson } from "./lesson";
import { ChoiceDialog } from "../app/components/choice-dialog";
import Profile from "../app/routes/profile";
import Learning from "../app/routes/learning";
import Reviews from "../app/routes/reviews";
import Library from "../app/routes/library";
import PendingSaves from "../app/routes/pending-saves";
import { clearPending, draftScope, saveDraft } from "../app/lib/learning-draft";
import { ownedTargetKey } from "../app/lib/owned-draft";
import type { SavedItem } from "@brioche/contracts/SavedItem";
import type { ReviewCard } from "@brioche/contracts/ReviewCard";
import type { ReviewQueue } from "@brioche/contracts/ReviewQueue";
import type { ReviewAttemptResult } from "@brioche/contracts/ReviewAttemptResult";
import type { LearningState } from "@brioche/contracts/LearningState";
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
  profileRelease: [] as ((profile: UserProfile | number) => void)[],
  profileReads: [] as ((profile: UserProfile | number) => void)[],
  navigate: null as ((destination: string | number) => void) | null,
  learningWrites: [] as Record<string, unknown>[],
  learningRelease: [] as ((value: LearningState | number) => void)[],
  reviewWrites: [] as Record<string, unknown>[],
  reviewRelease: [] as ((value: ReviewAttemptResult | number) => void)[],
  queueReads: [] as ((value: ReviewQueue | number) => void)[],
  reviewFixture: null as ReviewQueue | null,
  cancellations: 0,
  ownedWrites: [] as { path: string; body: Record<string, unknown> }[],
  ownedRelease: [] as ((value: SavedItem | ReviewCard | number) => void)[],
  savedFixture: null as SavedItem | null,
  confirmExternal: null as ((index: number) => void) | null,
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
    cancel() {
      qa.cancellations++;
    },
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
  if (
    (String(input).startsWith("/api/v1/me/saved-items/") &&
      init?.method === "PUT") ||
    String(input) === "/api/v1/me/review-enrollments"
  ) {
    const index =
      qa.ownedWrites.push({
        path: String(input),
        body: JSON.parse(String(init?.body)),
      }) - 1;
    return new Promise<Response>((resolve) => {
      qa.ownedRelease[index] = (value) =>
        resolve(
          typeof value === "number"
            ? new Response("", { status: value })
            : Response.json(value),
        );
    });
  }
  if (String(input) === "/api/v1/me/reviews/qa-card/attempts") {
    const index = qa.reviewWrites.push(JSON.parse(String(init?.body))) - 1;
    return new Promise<Response>((resolve) => {
      qa.reviewRelease[index] = (value) =>
        resolve(
          typeof value === "number"
            ? new Response("", { status: value })
            : Response.json(value),
        );
    });
  }
  if (String(input) === "/api/v1/me/reviews") {
    return new Promise<Response>((resolve) => {
      qa.queueReads.push((value) =>
        resolve(
          typeof value === "number"
            ? new Response("", { status: value })
            : Response.json(value),
        ),
      );
    });
  }
  if (String(input).startsWith("/api/v1/learning-sessions/qa-session/")) {
    const index = qa.learningWrites.push(JSON.parse(String(init?.body))) - 1;
    return new Promise<Response>((resolve) => {
      qa.learningRelease[index] = (value) =>
        resolve(
          typeof value === "number"
            ? new Response("", { status: value })
            : Response.json(value),
        );
    });
  }
  if (String(input) === "/api/v1/me/settings") {
    const index = qa.profileWrites.push(JSON.parse(String(init?.body))) - 1;
    return new Promise<Response>((resolve) => {
      qa.profileRelease[index] = (profile) =>
        resolve(
          typeof profile === "number"
            ? new Response("", { status: profile })
            : Response.json(profile),
        );
    });
  }
  if (String(input) === "/api/v1/me") {
    return new Promise<Response>((resolve) => {
      qa.profileReads.push((profile) =>
        resolve(
          typeof profile === "number"
            ? new Response("", { status: profile })
            : Response.json(profile),
        ),
      );
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
const progress: LearningState = {
  id: "qa-session",
  lessonId: lesson.id,
  revision: 1,
  version: 1,
  lastStepId: "read",
  confirmedStepIds: [],
  hintedExerciseIds: [],
  attempts: [],
  completedAt: null,
  firstCompletedAt: null,
};
function SessionHarness() {
  const session = { lesson, progress };
  return (
    <LearningProvider>
      <main>
        <Learning
          loaderData={{ session, ownerId: "qa-account" }}
          params={{ sessionId: "qa-session" }}
          matches={[
            {
              id: "root",
              params: {},
              pathname: "/",
              loaderData: { user: null, enabled: false },
              handle: undefined,
            },
            {
              id: "routes/learning",
              params: { sessionId: "qa-session" },
              pathname: "/",
              loaderData: { session, ownerId: "qa-account" },
              handle: undefined,
            },
          ]}
        />
      </main>
    </LearningProvider>
  );
}
const kind = new URL(location.href).searchParams.get("case");
const reviewQueue: ReviewQueue = {
  items: [
    {
      id: "qa-card",
      knowledgeId: "qa-word",
      sourceLessonId: lesson.id,
      sourceRevision: 1,
      vocabulary: {
        id: "qa-word",
        lemma: "bonjour",
        partOfSpeech: "phrase",
        gender: null,
        meaningZh: "你好",
        noteZh: "日常问候",
      },
      stage: 0,
      dueAt: "2026-10-06T00:00:00Z",
      version: 1,
      suspended: false,
    },
  ],
  dueCount: 1,
  nextDueAt: null,
  localDate: "2026-10-06",
  timeZone: "Asia/Shanghai",
};
qa.reviewFixture = reviewQueue;
const reviewUser: UserProfile = {
  id: "qa-account",
  email: "qa@example.test",
  displayName: "QA",
  role: "learner",
  version: 1,
  settings: {
    timeZone: "Asia/Shanghai",
    weeklyDays: 5,
    dailyMinutes: 10,
    showTranslation: false,
    speechRate: 1,
  },
};
function ReviewsHarness() {
  return (
    <LearningProvider user={reviewUser}>
      <main>
        <Reviews
          loaderData={reviewQueue}
          params={{}}
          matches={[
            {
              id: "root",
              params: {},
              pathname: "/",
              loaderData: { user: reviewUser, enabled: true },
              handle: undefined,
            },
            {
              id: "routes/reviews",
              params: {},
              pathname: "/",
              loaderData: reviewQueue,
              handle: undefined,
            },
          ]}
        />
      </main>
    </LearningProvider>
  );
}
const savedItem: SavedItem = {
  id: "qa-saved",
  knowledgeId: "qa-word",
  sourceLessonId: lesson.id,
  sourceRevision: 1,
  vocabulary: reviewQueue.items[0].vocabulary,
  saved: true,
  withdrawn: false,
  version: 1,
  createdAt: "2026-10-06T00:00:00Z",
};
qa.savedFixture = savedItem;
function LibraryHarness() {
  const loaderData = {
    view: "saved" as const,
    page: { items: [savedItem], nextCursor: null },
    cursor: null,
  };
  return (
    <LearningProvider user={reviewUser}>
      <main>
        <Library
          loaderData={loaderData}
          params={{}}
          matches={[
            {
              id: "root",
              params: {},
              pathname: "/",
              loaderData: { user: reviewUser, enabled: true },
              handle: undefined,
            },
            {
              id: "routes/library",
              params: {},
              pathname: "/",
              loaderData,
              handle: undefined,
            },
          ]}
        />
      </main>
    </LearningProvider>
  );
}
const pendingJobs = ["qa-account", "qa-account", "qa-next"].map(
  (userId, index) => {
    const knowledgeId = "pending-word-" + index;
    return {
      key:
        draftScope(userId, "owned", 1) +
        ":" +
        ownedTargetKey({
          kind: "bookmark",
          knowledgeId,
          lessonId: lesson.id,
          revision: 1,
        }),
      job: {
        path: "/api/v1/me/saved-items/" + knowledgeId,
        method: "PUT" as const,
        body: {
          sourceLessonId: lesson.id,
          sourceRevision: 1,
          saved: true,
          version: 0,
          idempotencyKey: "pending-recovery-operation-" + index,
        },
      },
    };
  },
);
if (kind === "pending") {
  for (const entry of pendingJobs) saveDraft(entry.key, entry.job);
  qa.confirmExternal = (index) =>
    clearPending(
      pendingJobs[index].key,
      pendingJobs[index].job.body.idempotencyKey,
    );
}
function PendingHarness() {
  const [user, setUser] = useState(reviewUser);
  qa.changeUser = () =>
    setUser({ ...reviewUser, id: "qa-next", displayName: "Next" });
  const loaderData = { userId: user.id };
  return (
    <LearningProvider user={user}>
      <main>
        <PendingSaves
          loaderData={loaderData}
          params={{}}
          matches={[
            {
              id: "root",
              params: {},
              pathname: "/",
              loaderData: { user, enabled: true },
              handle: undefined,
            },
            {
              id: "routes/pending-saves",
              params: {},
              pathname: "/",
              loaderData,
              handle: undefined,
            },
          ]}
        />
      </main>
    </LearningProvider>
  );
}
const reading = kind === "reading";
const router = createMemoryRouter(
  [
    {
      id: "root",
      path: "/",
      loader: () => ({ user: null, enabled: kind === "profile" }),
      element: reading ? (
        <LearningProvider>
          <ReadingHarness />
        </LearningProvider>
      ) : kind === "choices" ? (
        <ChoicesHarness />
      ) : kind === "profile" ? (
        <ProfileHarness />
      ) : kind === "session" ? (
        <SessionHarness />
      ) : kind === "reviews" ? (
        <ReviewsHarness />
      ) : kind === "library" ? (
        <LibraryHarness />
      ) : kind === "pending" ? (
        <PendingHarness />
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
