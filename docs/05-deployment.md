# 部署与运维设计

## 前提与默认拓扑

用户已确定生产运行在本机 Docker Compose，域名由 Cloudflare DNS 管理，路由器做静态端口映射。开发时 Web、API 和数据库在本机直接运行，不要求 Docker。工程已提供 `compose.yaml` 与 `infra/` 镜像配置；实际命令和当前限制见 [工程说明](08-development.md)。

本设计没有修改 DNS、路由器、防火墙，也没有启动生产服务。实际域名、公网地址、生产机器/系统和可映射外网端口仍待部署前补齐。

2026-10-06 用户确认：入口网关使用 Traefik，Compose 对外仅映射 HTTP `30075`，HTTPS 由用户在外部处理。本项目不配置证书、ACME、HTTPS 端口或 HTTP→HTTPS 跳转。

```text
浏览器 → 用户管理的外部入口 / HTTPS
       ↓
用户入口 / 路由器 → 生产机器 HTTP :30075
       ↓
Traefik（宿主机 30075 → 容器 8080）
  ├─ /api/*   → Rust server:3001（保留路径前缀）
  └─ 其他     → React Router SSR web:3000

server → postgres:5432（内部网络）
```

生产机器使用固定 DHCP 租约/静态 LAN 地址。静态端口映射不代表公网 IP 固定：公网 IP 变化时需要 DDNS 更新 A/AAAA。若处于 CGNAT 或运营商封锁入口，端口映射不会生效，需改变接入方式；Cloudflare Tunnel 只是可重新评估的备选，不在默认拓扑中。

## 开发环境

目标启动方式：本机 Rust stable（rustfmt/clippy）、Node.js LTS、pnpm、PostgreSQL，分别跑 Vite web/Axum API，统一通过 `http://localhost:5173` 使用。数据库只能本机访问；支持用户已经安装的 PostgreSQL，若尚未安装再确定安装方式。Vite 固定端口，代理 `/api` 至 3001；SSR loader 直接连接内部 API。

计划环境变量（未来 `.env.example` 只放占位值）：

| 变量 | 开发示例/用途 |
| --- | --- |
| DATABASE_URL | 指向本机 PostgreSQL 的专用开发数据库；禁止复制生产密码 |
| PUBLIC_APP_URL | `http://localhost:5173`；生产需含实际端口 |
| INTERNAL_API_URL | 开发 `http://127.0.0.1:3001`；生产 `http://server:3001` |
| SESSION_COOKIE_KEY | 每个环境独立随机秘密，供选定的 cookie 保护配置使用；绝不进入 VITE_ 公开变量 |
| MEDIA_ROOT | 本机已发布媒体目录，生产为持久卷路径 |
| CONTENT_SOURCE_ROOT | 内容源路径，仅 CLI 读取，不暴露给 Web |
| REGISTRATION_MODE | 默认 invite；dev fixture 初始化属于独立工具 |

`PUBLIC_APP_URL` 作为统一 origin 的项目配置，Rust 的 Origin/CSRF 校验和前端 SSR 据此配置；避免多个配置各写一套域名。变量名称是项目提案，并不假定认证库自动识别。若 cookie store 配置无需独立密钥，移除相应变量，不保留无实际用途的 secrets。

当前开发命令为 `pnpm dev:api`、`pnpm dev:web`；迁移使用 `cargo run -p brioche-server -- migrate`，课程导入使用 `import <file>` 子命令。离线 `check <lesson.json>` 与 `check-release <manifest.json>` 已实现；完整内容 Schema、staging 预览与媒体发布流程仍需补齐，详见 [工程说明](08-development.md)。

## Docker 服务与网络

| 服务 | 职责 | 持久化/端口 |
| --- | --- | --- |
| traefik | HTTP 同源路由 | 固定 `30075:8080`；只读 file provider 配置 |
| web | React Router SSR Node 服务 | Vite client/server 构建包含在镜像；内部 3000 |
| server | Rust Axum 二进制与同镜像内容 CLI | 媒体持久卷；内部 3001，DB 连接 |
| postgres | 数据库 | named volume；内部 5432 |
| migrate / content-cli | 一次性维护任务 | 与当前 release 镜像一致；正常运行后退出 |

Traefik/Web/API 共享应用内部网络，API/PostgreSQL 共享数据库网络，Web 不直接连接数据库。仅 Traefik 入口发布宿主端口，数据库和业务进程保留在内部网络。Traefik 使用 file provider，不挂 Docker socket；dashboard/API 未启用，健康检查仅监听容器 loopback `8082`。

基础镜像已固定 registry index digest；实际核对与更新约定见 [容器镜像说明](../infra/images.md)。Linux/amd64 的完整 Compose 构建、迁移/健康顺序与 HTTP 30075 的账号/SSR/学习/媒体链路已在隔离项目验证；这不是实际域名、用户数据或公网生产验收。

使用 Linux 容器。如果生产宿主机是 Windows，Docker Desktop/WSL2 的服务自启、网络转发和磁盘权限需要单独验证；不能把 `restart: unless-stopped` 当作 Docker 引擎本身会在开机后启动。媒体卷与 DB 卷路径必须确定，备份目录与 live volume 分开。

多阶段构建、非 root 应用进程、生产依赖、frozen lockfile、healthcheck 和日志轮转已配置；应用已有 body/pool/hash/media 并发限制，Compose 另有可调 CPU、内存与进程上限。在 Compose 中 readiness 和迁移顺序显式配置，不能只靠 depends_on 的启动顺序猜测数据库已就绪。应用支持 SIGTERM 优雅停止。

### 容器初始资源预算

| 服务 | CPU 上限 | 内存上限 | 进程/线程上限 | `.env` 前缀 |
| --- | --- | --- | --- | --- |
| PostgreSQL | 1.0 | 512 MiB | 128 | POSTGRES |
| 一次性迁移 | 1.0 | 256 MiB | 128 | MIGRATE |
| Rust API | 1.0 | 512 MiB | 256 | API |
| Web SSR | 1.0 | 384 MiB | 128 | WEB |
| Traefik | 0.5 | 256 MiB | 128 | TRAEFIK |

每项通过 `<前缀>_CPUS`、`<前缀>_MEMORY_LIMIT`、`<前缀>_PIDS_LIMIT` 调整，默认值与 `infra/production.env.example` 一致。使用服务级 [cpus](https://docs.docker.com/reference/compose-file/services/#cpus)、[mem_limit](https://docs.docker.com/reference/compose-file/services/#mem_limit)、[pids_limit](https://docs.docker.com/reference/compose-file/services/#pids_limit)，适用于本机 Compose；没有固定 CPU 核编号。CPU 配额是上限，不是保留宿主机核心；内存 swap 沿用运行时设置，未把这些值宣称为含 swap 的总内存预算。镜像构建由 BuildKit 执行，不受这些运行容器配额限制。

这是初始约束，不是生产容量承诺。四个常驻容器的内存上限合计 1664 MiB，迁移在 API 启动前退出；宿主机仍需为 Docker、操作系统、页面缓存、构建、备份和其他应用预留资源。不要只按容器上限之和判断宿主机容量。

准备上线时，先 `docker compose config --quiet` 验证最终参数，再启动隔离项目并通过 inspect 核对实际 Memory/NanoCpus/PidsLimit；同时测真实课程、登录哈希、并发 SSR/学习提交、媒体读取、备份和恢复负载。记录延迟分布、健康检查、OOMKilled、重启次数、CPU 节流和内存压力。达到预算时先查瓶颈，再在 `.env` 调整并重新创建对应服务，不关闭应用本身的并发/文件限制；上线负载和 RPO/RTO 仍需单独验收。

Web 使用 React Router 官方 Node 部署方式运行 Vite 生成的 client/server bundle，正确复制 assets 和生产依赖；不使用 `vite preview` 作为生产服务器。Rust 单独多阶段构建 release 二进制，固定 target/libc/TLS 配套环境，避免在 Windows 直接构建的 exe 放进 Linux 容器。Cargo.lock 和 pnpm-lock.yaml 均保留；SeaORM 迁移随版本构建为一次性工具，应用启动不自动 schema sync。

域名、session 配置和内部 API 地址是 runtime 配置；不得把生产秘密烘进镜像。VITE_ 变量会进入浏览器 bundle，只能保存公开值，优先同源相对 `/api`，不把内部 API URL 暴露给浏览器。SSR 关闭时可切换静态文件服务模式，但本提案默认保留 Web SSR 进程。

## 外部入口与 HTTPS 对接

Compose 内仅提供 HTTP，入口固定为宿主机 `30075`。用户管理的外部网关把请求转发至 `http://<生产机器>:30075`，并处理 HTTPS、Cloudflare 与路由器接入。本项目不需要 DNS API token 或证书卷。

应用的公开 origin 应配置为浏览器实际访问的地址，包含实际协议与外网端口；它与内部 HTTP 端口独立。生产 cookie 的 Secure 标记按外部 HTTPS origin 配置，CSRF 用明确的允许 origin 校验，不用不受信的转发 header 推断。若后续确需使用外部代理 header，仅配置用户实际入口的 trusted IP/CIDR，不全网信任。

认证/API/个人页面绕过共享缓存；含 cookie 的个性化 HTML/loader data 不缓存。Vite 内容哈希资源与已发布媒体可长期缓存。正式接入外部入口时用两个账号验证进度和会话隔离。
## 发布流程

1. 本机开发完成类型检查、相关测试、生产构建和内容校验。
2. 构建并标记 web/server/CLI 镜像，记录 release ID 和 digest；部署清单写入实际版本。
3. 备份数据库和媒体 manifest，验证备份可读取；保留此前镜像。
4. 在维护窗口运行一次 migrations；采用 expand/contract，避免立即破坏旧应用兼容性。
5. 启动新应用并等待 readiness；在本机和公网执行登录、课程阅读、提交、续学检查。
6. 用当前解释器兼容的 CLI 导入内容 staging release，预览后事务切换 active release。
7. 确认日志、TLS、保存和媒体访问正常，记录部署结果。

应用回滚切回旧镜像，但不能自动“反向执行”任意数据库迁移；只在迁移兼容时直接回滚，否则需要专门恢复/修复方案。内容回滚独立切 active release。上线操作属于后续任务，本次设计不执行。

## 备份、恢复和本机稳定性

建议初期每天逻辑备份（pg_dump custom format）、保留近 7 天和每周 4 份，媒体及 manifest 增量备份；秘密配置单独加密备份。目标 RPO 24 小时、RTO 2 小时均是提案，待数据规模和恢复演练验证。

备份至少另存一个不在生产 live volume/同一故障盘的位置；可先使用用户现有外置盘或加密远端存储，具体目的地待定。仅复制正在写入的 PostgreSQL 数据目录不等于一致备份。

恢复演练：隔离数据库 → pg_restore → 校验 migration 版本/release 指针/课数 → 恢复媒体并校验哈希 → 以测试 origin 启动 → 验证邀请/登录/旧会话/复习。至少在首个生产版本和重大 schema 变更后做完整演练；真实用户数据不能写入仓库。

已有可执行命令和隔离恢复证据，见 [工程备份与恢复说明](08-development.md#docker-备份与恢复)。脚本生成 custom-format 数据库快照与按哈希命名的登记媒体文件，只有全部写入并校验后才生成完整 manifest；恢复仅创建新数据库和新媒体卷，不自动切换应用。样本演练已覆盖旧会话、新登录、学习/复习数据及媒体；生产规模的 RPO/RTO、异盘加密副本、保留策略和用户实际入口仍待验证。

本机生产需要关闭自动休眠、确认断电/重启后 Docker 和服务启动、验证公网 IP 更新及证书续期，并管理磁盘剩余空间。没有外部探测时机器断电无法自己告警；可后续配置一个外部 uptime 检查，但本次不创建自动化或外部服务。

## 部署前待补信息

- 实际域名，Cloudflare DNS only 或代理模式，可用外网端口。
- 生产机器是否为当前 Windows 开发机，Docker 运行方式、LAN 地址和数据存放盘。
- 公网 IPv4/IPv6、是否 CGNAT、是否需要 DDNS。
- 邀请制是否足够，是否有 SMTP 用于账号恢复。
- 备份目的地、可接受停机窗口和容量。

## 官方资料

资料核对日期：2026-10-05。

- [Cloudflare 支持的代理端口](https://developers.cloudflare.com/fundamentals/reference/network-ports/)
- [Cloudflare Full (strict)](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/)
- [Traefik file provider](https://doc.traefik.io/traefik/providers/file/)
- [Traefik entrypoints](https://doc.traefik.io/traefik/routing/entrypoints/)
- [Vite SSR](https://vite.dev/guide/ssr)、[React Router 部署](https://reactrouter.com/start/framework/deploying)

## 手机同步开发预览

用户已确认 Web/API/预览服务尽可能不绑定特定 IP。开发 Web 使用 Vite `server.host: true`、`strictPort: true`，默认 5173；Axum 配置监听 `0.0.0.0:3001`（支持 IPv6 时另明确双栈行为）。预览 Python 不指定 `--bind`。手机使用电脑当前 LAN 地址，不把该地址写入共享代码。数据库仍仅本机或 Docker 内部网络访问。

手机通过 Web 入口和相对 `/api` 请求 API；代理/SSR 的内部连接地址仍可使用 127.0.0.1，它是连接目标，不是对外监听限制。使用手机时 PUBLIC_APP_URL 配置成实际 Web origin；若同时保留 localhost 与 LAN 登录，新增明确的开发允许 origin 列表供 CSRF/重定向检查使用，不能全开放 CORS 或禁用 CSRF。Vite 保留默认 IP 访问支持，不配置 `allowedHosts: true`。

当前设计预览端口 4173 已全接口监听，本机 LAN HTTP 请求通过；手机仍需与电脑同网实测。当前正式 Vite/Axum 工程也按同一全接口监听约定运行。

参考：[Vite server.host](https://vite.dev/config/server-options.html#server-host)。
