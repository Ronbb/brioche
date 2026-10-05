# 部署与运维设计

## 前提与默认拓扑

用户已确定生产运行在本机 Docker，域名由 Cloudflare DNS 管理，路由器做静态端口映射。开发时 Web、API 和数据库在本机直接运行，不要求 Docker。

本设计没有修改 DNS、路由器、防火墙，也没有启动生产服务。实际域名、公网地址、生产机器/系统和可映射外网端口仍待部署前补齐。

默认建议：外网 443 → 生产机器 443 → Caddy；Cloudflare 可先 DNS only 验证，再启用代理。若外网 443 不可用，选 Cloudflare 官方支持的 HTTPS 端口（例如 8443），浏览器 URL 必须显式带端口；普通 DNS 记录不会把一个端口改写成另一个。

使用非标准外网端口时，应用 canonical origin、认证跳转及可选 HTTP → HTTPS 重定向都必须保留外网端口，不能把内部 Caddy 的 443 当作浏览器访问端口；实际 Caddy 配置需按外网映射验证。

```text
浏览器 https://<domain>[:port]
  ├─ DNS only：域名解析到公网 IP，浏览器直接连接源站
  └─ Proxied：浏览器连接 Cloudflare，边缘再连接源站
       ↓
路由器 external HTTPS port → 固定 LAN 地址:443
       ↓
Caddy（Docker 对宿主机只映射所需 HTTPS 端口）
  ├─ /api/*   → Rust server:3001（保留路径前缀）
  ├─ /media/* → 只读 published media 持久卷
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

后续计划命令为 `pnpm dev:web`、`cargo run -p brioche-server`，迁移由 sea-orm-migration 的专用 CLI/子命令运行，内容工具计划使用 Rust CLI 的 `content check/publish` 子命令；具体可执行名随工程建立记录。本阶段尚未创建 package.json/Cargo.toml，因此这些命令当前不可运行。

## Docker 服务与网络

| 服务 | 职责 | 持久化/端口 |
| --- | --- | --- |
| caddy | TLS、路由、静态媒体 | 证书数据卷、只读媒体卷；宿主映射 HTTPS，必要时 HTTP |
| web | React Router SSR Node 服务 | Vite client/server 构建包含在镜像；内部 3000 |
| server | Rust Axum 二进制 | 只读媒体/配置；内部 3001，DB 连接 |
| postgres | 数据库 | named volume；内部 5432 |
| migrate / content-cli | 一次性维护任务 | 与当前 release 镜像一致；正常运行后退出 |

Caddy/Web/API 共享应用内部网络，API/PostgreSQL 共享数据库网络，Web 不直接连接数据库。仅 Caddy 入口发布宿主端口；不对公网或 LAN 暴露数据库、Docker API、认证 secrets 和 Caddy 管理 API。

使用 Linux 容器。如果生产宿主机是 Windows，Docker Desktop/WSL2 的服务自启、网络转发和磁盘权限需要单独验证；不能把 `restart: unless-stopped` 当作 Docker 引擎本身会在开机后启动。媒体卷与 DB 卷路径必须确定，备份目录与 live volume 分开。

多阶段构建、非 root 应用进程、生产依赖、frozen lockfile、healthcheck、日志轮转、资源上限。在 Compose 中 readiness 和迁移顺序显式配置，不能只靠 depends_on 的启动顺序猜测数据库已就绪。应用支持 SIGTERM 优雅停止。

Web 使用 React Router 官方 Node 部署方式运行 Vite 生成的 client/server bundle，正确复制 assets 和生产依赖；不使用 `vite preview` 作为生产服务器。Rust 单独多阶段构建 release 二进制，固定 target/libc/TLS 配套环境，避免在 Windows 直接构建的 exe 放进 Linux 容器。Cargo.lock 和 pnpm-lock.yaml 均保留；SeaORM 迁移随版本构建为一次性工具，应用启动不自动 schema sync。

域名、session 配置和内部 API 地址是 runtime 配置；不得把生产秘密烘进镜像。VITE_ 变量会进入浏览器 bundle，只能保存公开值，优先同源相对 `/api`，不把内部 API URL 暴露给浏览器。SSR 关闭时可切换静态文件服务模式，但本提案默认保留 Web SSR 进程。

## DNS、Cloudflare 与证书

### DNS only

`A` 指向可访问的公网 IPv4；只有 IPv6 也能实际访问且防火墙就绪时才设置 `AAAA`。Caddy 使用公网可信证书。若能开放外网 80/443，可使用自动 ACME HTTP/TLS 验证；如果只开放自定义 HTTPS 端口，推荐 DNS-01。

### 启用 Cloudflare 代理

仅使用 Cloudflare 支持的端口；TLS 设为 Full (strict)，源站必须有符合要求的证书。默认选公开 ACME 证书，方便 DNS only、局域网和诊断访问；Cloudflare Origin CA 是另一方案，但不被普通浏览器直接信任。

DNS-01 推荐使用 `caddy-dns/cloudflare` provider；这需要**构建包含插件的 Caddy 镜像**，普通官方 Caddy 镜像并不自带该 DNS provider。API token 限定相关 zone 和 DNS 修改权限，配置见 provider 官方要求，不写全局 API key、不进 Git、不输出到日志。

DNS-01 不需要 CA 访问源站 80/443，但浏览器到源站/边缘的实际入口仍然必须可达。TLS-ALPN 验证可能被 Cloudflare 代理终止而无法抵达 Caddy，因此代理模式优先 DNS-01；证书选择由实际入口条件决定。

建议在明确源站模式后建立相应防火墙规则。代理模式可以只接受官方 Cloudflare IP 段，需维护列表并保留本机运维通道；DNS only 模式不能使用同样的限制阻断正常浏览器。Caddy 信任的 forwarded headers 和 API 的 trustProxy 只限实际代理来源，不设全网信任。

Cloudflare 缓存规则：认证/API/个人页面一律绕过，含 session cookie 的个性化 HTML/loader data 不缓存；Vite 内容哈希 `/assets/*` 与内容哈希媒体可长期缓存。公共目录按 release ID 缓存，active 指针短缓存/及时失效。正式使用 CDN 缓存前用两个账号验证没有进度和会话串用。

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
- [Caddy 自动 HTTPS 与验证方式](https://caddyserver.com/docs/automatic-https)
- [Caddy Cloudflare DNS provider](https://github.com/caddy-dns/cloudflare)
- [Vite SSR](https://vite.dev/guide/ssr)、[React Router 部署](https://reactrouter.com/start/framework/deploying)

## 手机同步开发预览

用户已确认 Web/API/预览服务尽可能不绑定特定 IP。开发 Web 使用 Vite `server.host: true`、`strictPort: true`，默认 5173；Axum 配置监听 `0.0.0.0:3001`（支持 IPv6 时另明确双栈行为）。预览 Python 不指定 `--bind`。手机使用电脑当前 LAN 地址，不把该地址写入共享代码。数据库仍仅本机或 Docker 内部网络访问。

手机通过 Web 入口和相对 `/api` 请求 API；代理/SSR 的内部连接地址仍可使用 127.0.0.1，它是连接目标，不是对外监听限制。使用手机时 PUBLIC_APP_URL 配置成实际 Web origin；若同时保留 localhost 与 LAN 登录，新增明确的开发允许 origin 列表供 CSRF/重定向检查使用，不能全开放 CORS 或禁用 CSRF。Vite 保留默认 IP 访问支持，不配置 `allowedHosts: true`。

当前设计预览端口 4173 已全接口监听，本机 LAN HTTP 请求通过；手机仍需与电脑同网实测。正式应用构建前暂无可运行的 Vite/Axum 配置。

参考：[Vite server.host](https://vite.dev/config/server-options.html#server-host)。

