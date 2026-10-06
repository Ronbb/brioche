import type { CsrfToken } from "@brioche/contracts/CsrfToken";
export async function adminWrite<T>(
  path: string,
  body: object,
  signal?: AbortSignal,
): Promise<T> {
  signal?.throwIfAborted();
  const bootstrap = await fetch("/api/v1/auth/csrf", {
    cache: "no-store",
    signal: signal
      ? AbortSignal.any([signal, AbortSignal.timeout(10000)])
      : AbortSignal.timeout(10000),
  });
  if (!bootstrap.ok) throw Error("管理员服务暂时不可用。");
  const { csrfToken } = (await bootstrap.json()) as CsrfToken;
  signal?.throwIfAborted();
  const response = await fetch(`/api/v1/operator/${path}`, {
    method: "POST",
    cache: "no-store",
    headers: { "Content-Type": "application/json", "X-CSRF-Token": csrfToken },
    body: JSON.stringify(body),
    signal: signal
      ? AbortSignal.any([signal, AbortSignal.timeout(30000)])
      : AbortSignal.timeout(30000),
  });
  if (!response.ok) {
    const messages: Record<number, string> = {
      400: "未通过检查，请核对审批、素材与填写的信息。",
      401: "登录已过期，请重新登录。",
      403: "没有管理权限或请求未通过验证。",
      404: "没有找到指定对象，请刷新后核对。",
      409: "状态已发生变化，请刷新后核对再操作。",
      410: "这个课程版本已撤回。",
    };
    throw Error(
      messages[response.status] ?? "操作未确认，请刷新核对状态后重试。",
    );
  }
  return response.json() as Promise<T>;
}
