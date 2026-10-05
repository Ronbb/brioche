import type { Catalog } from "@brioche/contracts/Catalog";
import type { PublicLesson } from "@brioche/contracts/PublicLesson";
import type { UserProfile } from "@brioche/contracts/UserProfile";
const base = () => process.env.INTERNAL_API_URL ?? "http://127.0.0.1:3001";
async function api<T>(path: string): Promise<T> {
  let response: Response;
  try {
    response = await fetch(base() + path, {
      signal: AbortSignal.timeout(5000),
    });
  } catch {
    throw new Response("课程服务暂时无法连接，请稍后重试。", { status: 503 });
  }
  if (!response.ok)
    throw new Response(
      response.status === 404 ? "没有找到这堂课程。" : "课程服务暂时不可用。",
      { status: response.status },
    );
  return response.json() as Promise<T>;
}
export const getCatalog = () => api<Catalog>("/api/catalog");
export const getLesson = (id: string) =>
  api<PublicLesson>("/api/lessons/" + encodeURIComponent(id));
export async function getIdentity(
  request: Request,
): Promise<{ user: UserProfile | null; enabled: boolean }> {
  const cookie = (request.headers.get("cookie") ?? "")
    .split(";")
    .map((v) => v.trim())
    .filter((v) => /^(?:__Host-brioche\.sid|brioche\.sid)=/.test(v))
    .join("; ");
  let response: Response;
  try {
    response = await fetch(base() + "/api/v1/me", {
      headers: cookie ? { cookie } : {},
      signal: AbortSignal.timeout(5000),
    });
  } catch {
    throw new Response("账号服务暂时无法连接。", { status: 503 });
  }
  if (response.status === 401) return { user: null, enabled: true };
  if (response.status === 404) {
    const catalog = await getCatalog();
    if (catalog.developmentFixture) return { user: null, enabled: false };
  }
  if (!response.ok) throw new Response("账号服务暂时不可用。", { status: 503 });
  return { user: (await response.json()) as UserProfile, enabled: true };
}
