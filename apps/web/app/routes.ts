import { index, route, type RouteConfig } from "@react-router/dev/routes";
export default [
  index("routes/home.tsx"),
  route("lessons/:lessonId", "routes/lesson.tsx"),
  route("learning/:sessionId", "routes/learning.tsx"),
  route("practice/:lessonId", "routes/practice.tsx"),
  route("review/:lessonId", "routes/review.tsx"),
  route("reviews", "routes/reviews.tsx"),
  route("profile", "routes/profile.tsx"),
  route("login", "routes/login.tsx"),
  route("invite", "routes/invite.tsx"),
  route("reset-password", "routes/reset-password.tsx"),
  route("health", "routes/health.ts"),
] satisfies RouteConfig;
