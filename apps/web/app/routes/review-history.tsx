import { Link, redirect } from "react-router";
import type { ReviewHistoryPage } from "@brioche/contracts/ReviewHistoryPage";
import { getPrivate } from "../lib/api.server";
import { Icon } from "../components/icon";
import type { Route } from "./+types/review-history";
export async function loader({ request }: Route.LoaderArgs) {
  const cursor = new URL(request.url).searchParams.get("cursor");
  try {
    return await getPrivate<ReviewHistoryPage>(
      request,
      "/api/v1/me/review-history" +
        (cursor ? "?cursor=" + encodeURIComponent(cursor) : ""),
    );
  } catch (error) {
    if (error instanceof Response && error.status === 401)
      throw redirect("/login?next=/review-history");
    throw error;
  }
}
export default function History({ loaderData }: Route.ComponentProps) {
  const labels = { again: "还不熟", remembered: "有印象", familiar: "记住了" };
  return (
    <section className="settings-page page-arrive">
      <div className="settings-title-row">
        <h1>复习记录</h1>
        <Link className="text-button" to="/library?view=reviews">
          我的表达
        </Link>
      </div>
      {!loaderData.items.length && (
        <p className="profile-note">完成一次复习后，记录会显示在这里。</p>
      )}
      <ul className="review-result-list">
        {loaderData.items.map((item) => (
          <li key={item.id}>
            <div>
              <span
                className="result-expression"
                lang={item.withdrawn ? "zh-CN" : "fr"}
              >
                {item.vocabulary?.lemma ?? "来源内容已撤回"}
              </span>
              <small>
                {new Date(item.reviewedAt).toLocaleString("zh-CN", {
                  timeZone: item.timeZone,
                  year: "numeric",
                  month: "numeric",
                  day: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                })}{" "}
                · {labels[item.rating]}
              </small>
            </div>
            <span className="review-result-grade">
              下次{" "}
              {new Date(item.dueAt).toLocaleDateString("zh-CN", {
                timeZone: item.timeZone,
                month: "numeric",
                day: "numeric",
              })}
            </span>
          </li>
        ))}
      </ul>
      {loaderData.nextCursor && (
        <Link
          className="text-button"
          to={
            "/review-history?cursor=" +
            encodeURIComponent(loaderData.nextCursor)
          }
        >
          更早记录
          <Icon name="chevron" />
        </Link>
      )}
    </section>
  );
}
