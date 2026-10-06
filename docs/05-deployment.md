# 部署与运维设计

## 前提与默认拓扑

用户已确定生产运行在本机 Docker Compose，域名由 Cloudflare DNS 管理，路由器做静态端口映射。开发时 Web、API 和数据库在本机直接运行，不要求 Docker。工程已提供 `compose.yaml` 与 `infra/` 镜像配置；实际命令和当前限制见 [工程说明](08-development.md)。

本机已启动 Compose 的 production/database 应用栈并保留运行，当前正式课程目录为空；运行验证见 [验证记录](07-design-verification.md)，本机配置位置见被忽略的 AGENTS.local.md。没有修改 DNS、路由器、防火墙或外部 HTTPS；实际公网入口与生产环境验收仍待对接。

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

当前启动方式：本机 Rust（rustfmt/clippy）、Node.js、pnpm，分别跑 Vite Web/Axum API，统一通过 `http://localhost:5173` 使用。版本以锁文件和工具链配置为准。账号持久化模式另需专用 PostgreSQL；开发 fixture 可无数据库运行，但不提供真实账号进度。数据库只能本机访问。Vite 固定端口，代理 `/api` 至 3001；SSR loader 直接连接内部 API。

实际配置分别见根目录 `.env.example`（开发）与 `infra/production.env.example`（Compose）：

| 变量 | 开发示例/用途 |
| --- | --- |
| DATABASE_URL | 指向本机 PostgreSQL 的专用开发数据库；禁止复制生产密码 |
| APP_ENV / CONTENT_MODE | 开发 `development` / `fixture`；Compose 固定 `production` / `database`，生产禁止 fixture |
| API_BIND | 开发和容器内均默认 `0.0.0.0:3001` |
| PUBLIC_APP_URL | `http://localhost:5173`；生产需含实际端口 |
| ADDITIONAL_APP_ORIGINS | 逗号分隔的明确额外 origin；手机登录时使用实际 LAN 地址和端口 |
| INTERNAL_API_URL | 开发 `http://127.0.0.1:3001`；生产 `http://server:3001` |
| MEDIA_ROOT | 本机已发布媒体目录，生产为持久卷路径 |
| POSTGRES_PASSWORD | Compose 专用随机数据库秘密；Compose 组装内部 DATABASE_URL，禁止进入 Git 或公开日志 |

`PUBLIC_APP_URL` 是浏览器实际访问的 origin，Rust 据此配置 CSRF allowlist、认证链接和 Secure Cookie；SSR 使用内部 API 地址连接服务端。会话采用数据库记录和随机 cookie ID，不需要旧提案中的 SESSION_COOKIE_KEY。账号邀请/恢复由 CLI 签发，课程与素材路径由 CLI 参数传入，不读取旧提案中的 REGISTRATION_MODE 或 CONTENT_SOURCE_ROOT。

当前开发命令为 `pnpm dev:api`、`pnpm dev:web`；迁移使用 `cargo run -p brioche-server -- migrate`，课程导入使用 `import <file>` 子命令。离线校验、固定版本/整批 staging 预览、判分预览和登记媒体发布已经接入；完整验收与内容人工审校的状态见 [工程说明](08-development.md) 和 [实现清单](09-implementation-tracker.md)。

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

### 工程收尾后的实际启动

将 `infra/production.env.example` 复制为被 Git 忽略的根目录 `.env`，设置新随机数据库密码与实际浏览器 origin。不要覆盖已有部署秘密；启动前核对当前 Compose 项目和 30075 端口占用。

```sh
docker compose config --quiet
docker compose build
docker compose up -d --wait --wait-timeout 180
docker compose ps --all
pnpm health:check --project brioche --origin http://127.0.0.1:30075
```

验收要求：migrate 退出码为 0，PostgreSQL/API/Web/Traefik 四个长期服务 healthy，入口首页、Web health、API health/ready 成功，只有 HTTP 30075 对外映射。启动失败时读取该项目状态和日志定位，不能只重复执行 up 或以构建成功代替健康验收；不要输出完整含秘密的 Compose 渲染配置。

未激活已审校 release 时，生产目录为空是明确支持的状态。可先启动以进行账号邀请与管理员私有审校预览，随后按发布流程激活正式内容；不能把启动成功或空目录当作完整教学内容已经交付。用户要求最终保留应用在 Docker 中运行，最终交付须报告实际入口和启动验收结果，不能沿用此前已清理的隔离演练作为证明。

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

## 运行巡检

部署后可运行一次性巡检；项目名必须是实际 Compose project name，入口默认为本机 HTTP 30075：

```sh
pnpm health:check --project brioche
node scripts/health-check.mjs --project brioche --origin http://127.0.0.1:30075 --disk-path /your/data/filesystem --minimum-free-gib 5
```

`--disk-path` 应指向要检查的宿主机存储盘；Windows 可传实际磁盘路径。此项测量该路径所在文件系统的可用空间，不能自动证明 Docker Desktop 虚拟磁盘内部、远端备份或每个 volume 的剩余空间。未指定路径时，输出 `disk: null`，不假装磁盘检查通过。

脚本只读指定项目的容器状态，不执行 compose config，不读取容器环境变量、健康日志或 HTTP 正文到报告。期望 postgres/server/web/traefik 各有一个 running/healthy 容器，migrate 有一个退出码为 0 的完成容器；不要在巡检前删除迁移容器。OOM、缺失/重复服务、非就绪状态均失败；one-off compose run 容器排除。restart count 仅报告累计值，没有把它冒充某段时间内的重启频率。

入口检查 `/api/health`、`/api/ready`、`/health` 和首页 HTML；不跟随重定向。默认每次 Docker 命令和每个 HTTP 请求最多 5 秒（`--timeout-ms` 可设 100–30000），返回正文限 512 KiB；超时、格式错误或状态错误均失败。起动过程中 starting 也会失败，定时巡检应避开明确的维护窗口。HTTP 200 与 ready 只证明基础链路和当前数据库结构可用，不验证全部业务或正式课程质量。

每次输出一行版本化 JSON：`status: healthy` 退出 0，检测故障退出 1，参数或脚本级失败退出 2。报告包含检查时间、服务/路径、有限原因、耗时和可选磁盘字节数，可由用户现有的任务计划程序、cron 或监控系统定时执行、保留最近记录，并根据连续失败触发告警。该脚本不发送通知、不自动重启或删除容器；此轮未安装定时任务或配置外部告警收件人。机器断电、Docker/Node 无法运行时还需要外部探测，不能以本机脚本替代。

当前已用独立 Compose 项目验证健康→停止 Web→恢复的实际退出码 0→1→0；检查使用测试镜像与临时数据，不表示正式域名或生产容量验收。实现依据：[Docker inspect](https://docs.docker.com/reference/cli/docker/inspect/)、[Docker labels](https://docs.docker.com/engine/manage-resources/labels/)；具体证据见 [验证记录](07-design-verification.md)。

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
