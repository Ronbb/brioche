# 独立 HTTPS 网关接入

基础 compose.yaml 保留单独运行时的 HTTP30075 入口。2026-10-07 用户启用外层 HTTPS 网关并要求去掉不安全端口访问，当前生产叠加 compose.https.yaml，移除宿主HTTP端口映射，内部 Traefik8080 保留。

外层网关须加入该部署的 app 网络，并将请求转发至内部 Traefik。当前独立网关已具备这个连接；不要用此覆盖文件启动一个尚无外部入口的新环境。实际域名、证书与API允许来源保存在部署私有配置，HTTPS主地址决定Secure Cookie；每个浏览器主机分别登录。

维护时始终使用两个 Compose 文件，避免恢复HTTP映射：

```sh
docker compose --env-file /your/private/deployment.env --project-name brioche -f compose.yaml -f compose.https.yaml build server web
docker compose --env-file /your/private/deployment.env --project-name brioche -f compose.yaml -f compose.https.yaml up -d --wait
pnpm health:check --project brioche --origin https://your-app.example
docker port brioche-traefik-1
```

最后一条不应输出宿主HTTP映射。巡检必须显式传入实际HTTPS origin，不能沿用脚本的HTTP30075默认值。HTTPS转发、首页、API health/ready及会话CSRF来源都需实际核对。清除端口使用 Compose 的 [!reset 合并规则](https://docs.docker.com/reference/compose-file/merge/)，当前本机 Compose5.3 已实际执行成功。外部 DNS、证书与路由器由独立网关管理，本文件不自动改变它们。
