import { index, route, type RouteConfig } from "@react-router/dev/routes";
export default [
  index("routes/home.tsx"),
  route("lessons/:lessonId", "routes/lesson.tsx"),
  route("review/:lessonId", "routes/review.tsx"),
  route("profile", "routes/profile.tsx"),
  route("health", "routes/health.ts"),
] satisfies RouteConfig;
