import { Link, data, useLocation } from "react-router";
import type { AdminHistory } from "@brioche/contracts/AdminHistory";
import { getIdentity, getPrivate } from "../lib/api.server";
import { usePageCursorFocus } from "../components/page-cursor-focus";
import type { Route } from "./+types/admin-history";

export async function loader({ request }: Route.LoaderArgs) {
  const { user } = await getIdentity(request);
  if (!user) throw new Response("请先登录。", { status: 401 });
  if (user.role !== "operator")
    throw new Response("仅管理员可以进入。", { status: 403 });
  const search = new URLSearchParams();
  const parameters = new URL(request.url).searchParams;
  for (const key of ["beforeTime", "beforeKey"]) {
    const value = parameters.get(key);
    if (value !== null) search.set(key, value);
  }
  return data(
    await getPrivate<AdminHistory>(
      request,
      `/api/v1/operator/history${search.size ? `?${search}` : ""}`,
    ),
    {
      headers: { "Cache-Control": "private, no-store", Vary: "Cookie" },
    },
  );
}
export function headers() {
  return { "Cache-Control": "private, no-store", Vary: "Cookie" };
}
const labels: Record<string, string> = {
  approve: "批准课程",
  reject: "退回课程",
  import: "导入课程",
  stage: "创建发布目录",
  activate: "切换发布目录",
  withdraw: "撤回课程版本",
  invite: "生成账号邀请",
  inviteOperator: "生成管理员邀请",
  reset: "生成密码重置链接",
};
export default function AdminHistoryPage({
  loaderData: history,
}: Route.ComponentProps) {
  const location = useLocation();
  const cursor = new URLSearchParams(location.search).get("beforeKey");
  const heading = usePageCursorFocus(cursor);
  const next =
    history.next &&
    new URLSearchParams({
      beforeTime: history.next.beforeTime,
      beforeKey: history.next.beforeKey,
    });
  return (
    <section className="admin-page page-arrive">
      <div className="admin-heading">
        <h1 ref={heading} tabIndex={-1}>
          审批与发布记录
        </h1>
        <Link className="text-button" to="/admin">
          管理员后台
        </Link>
      </div>
      {!history.items.length && (
        <p role="status">
          {cursor ? "这一页没有更早的记录。" : "还没有操作记录。"}
        </p>
      )}
      <ol className="admin-history">
        {history.items.map((item) => (
          <li className="admin-card" key={item.key}>
            <div className="admin-card-heading">
              <strong>{labels[item.action] ?? item.action}</strong>
              <time dateTime={item.createdAt}>
                {new Intl.DateTimeFormat("zh-CN", {
                  dateStyle: "medium",
                  timeStyle: "medium",
                  timeZone: "Asia/Shanghai",
                }).format(new Date(item.createdAt))}
                （中国时间）
              </time>
            </div>
            <h2>{item.target}</h2>
            <p className="admin-note">{item.reason}</p>
            <p className="admin-actor">操作来源：{item.actor}</p>
          </li>
        ))}
      </ol>
      {next && (
        <Link className="text-button" to={`/admin/history?${next}`}>
          更早的记录
        </Link>
      )}
      {cursor && (
        <Link className="text-button" to="/admin/history">
          最新记录
        </Link>
      )}
    </section>
  );
}
