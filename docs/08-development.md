# 第一轮工程实现

更新日期：2026-10-06。这是工程基础与课程阅读链路，尚未完成多用户学习产品。

## 已实现

- pnpm / Cargo workspace、锁文件、Rust 1.99.0 工具链、TypeScript 7 独立类型检查。
- React Router Framework Mode + Vite SSR；正式客户端路由 `/`、`/lessons/:lessonId`、`/review/:lessonId`、`/profile`。
- 迁移确认过的视觉与主要阅读交互：角色介绍、头像整句、点词朗读和词汇解释、短文、全文播放/暂停/长按调速、复习卡片、统一设置、toast、动效和覆盖式滚动条。
- Axum 公共目录和课程 API、health/readiness、SIGINT/SIGTERM 优雅退出。
- Rust Serde 公共 DTO，生成 TS 联合类型和公共课程 JSON Schema；Web 只导入公共契约。私有答案与编辑状态不进入课程响应。
- SeaORM PostgreSQL Entity、版本化显式迁移、仅插入的课程导入工具、草稿过滤及最新发布 revision 读取。
- `/practice/:lessonId` 示例练习：选择、填空、排序由 Rust 判分，错误反馈/重试/回顾由页面呈现。私有答案规则在导入时校验，公共契约只包含提交和反馈 DTO。
- 正文增加结构化解释、文化范围、词汇与语法；点语法锚点可打开解释。校验拒绝坏解释锚点、重复题目选项、未知步骤类型和不可达教学块。
- Docker Compose：PostgreSQL → 一次性迁移 → API → SSR Web → Traefik。入口仅 HTTP 30075，HTTPS 由用户外部处理；数据库不映射宿主端口，生产关闭示例课程模式。
- 账号模块：邀请注册、登录、退出、密码恢复，Argon2id 密码哈希、PostgreSQL 会话与一次性 token、精确 Origin + CSRF、持久化登录限流及过期清理。Web 增加 `/login`、`/invite`、`/reset-password`，SSR 从 Cookie 读取实际身份，私有响应禁止缓存。
- 个人资料与设置：昵称、IANA 时区、每周 3/5/7 天与每天 5/10/15 分钟目标、中文译文和 0.75/1/1.25/1.5 倍速。登录用户跨设备保存，访客保留浏览内设置；个人页使用可搜索的自定义时区面板。版本锁拒绝旧设备覆盖，失败保留编辑草稿，读取最新状态后明确重试。
- 账号学习：`/learning/:sessionId` 按结构数据遍历所有步骤和正文块；服务端固定课程 revision，保存步骤、提示、首次及重试答案，确认完成后生成去重复习卡片。首页显示真实续学入口和课程完成记录。账号 `/reviews` 提供每批最多 10 项到期队列、自评与保存回顾；收藏尚待实现。

## 本机开发

要求 Node 24、pnpm 11.11.0、Rust 1.99.0。`rust-toolchain.toml` 会固定工具链并安装 rustfmt/clippy。

```sh
pnpm install --frozen-lockfile
pnpm contracts
pnpm dev:api
```

另开终端：

```sh
pnpm dev:web
```

打开 `http://localhost:5173/`；手机使用 `http://<电脑局域网地址>:5173/`。两个服务默认监听所有 IPv4 接口。开发 Web 将 `/api` 代理到本机 3001，SSR 通过 `INTERNAL_API_URL` 查询 Rust。

`dev:api` 显式设置 `APP_ENV=development`、`CONTENT_MODE=fixture`。示例课含未审校内容，仅供开发；这条路径不需要数据库。`cargo run -p brioche-server` 直接运行默认采用生产数据库模式，需要 `DATABASE_URL`，不会默认开放草稿。

可复制根目录 `.env.example` 配置 API；内部 SSR 地址通过环境变量传入 Web。生产域名、秘密和实际连接串不可提交。

## 数据库与课程导入

在本机 PostgreSQL 中建立专用数据库并设置 `DATABASE_URL`。使用 CLI 执行迁移，服务启动不会自动同步表结构。

```sh
cargo run -p brioche-server -- migrate
cargo run -p brioche-server -- import docs/examples/a1-bakery.lesson.json
```

不带 `--publish` 导入不可见草稿；带 `--publish` 要求 `editorial.status=reviewed`。本示例未审校，禁止为测试上线而直接改状态。相同 `(lesson_id, revision)` 重复导入失败，不覆盖已有快照。正式发布流程还需内容哈希、媒体授权检查、目录 release、审计及撤回机制；当前导入工具不能替代完整发布流程。

## 实际检查命令

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm contracts
pnpm typecheck
pnpm build
```

PostgreSQL 集成测试显式要求 `TEST_DATABASE_URL` 指向专用测试数据库：

```sh
cargo test -p brioche-server --test postgres -- --ignored
cargo test -p brioche-server --test identity -- --ignored
cargo test -p brioche-server --test learning -- --ignored
```

测试在独立、随机命名的 schema 中执行迁移、发布读取与唯一约束验证。普通测试运行会跳过它；CI 使用隔离的 PostgreSQL 服务执行。测试失败可能留下该测试 schema，禁止在生产数据库运行。

## Docker Compose

用户已确认部署可使用 Docker Compose。复制 `infra/production.env.example` 为根目录 `.env`，填写随机数据库秘密和 `PUBLIC_APP_URL`；密码使用字母数字或正确 URL 编码。`PUBLIC_APP_URL` 必须是浏览器最终访问的 origin，如外部 HTTPS 域名；后端由此选择 Secure Cookie 和 CSRF allowlist，内部网关仍为 HTTP。手机开发访问可用 `ADDITIONAL_APP_ORIGINS` 显式补充实际 LAN origin，不接受通配符。然后先检查并构建：

```sh
docker compose config --quiet
docker compose build
```

准备好生产参数和已审校课程后，启动方式为 `docker compose up -d`。访问 `http://<宿主机地址>:30075`；对外 HTTPS、域名和路由器由用户处理。Compose 内不申请证书，也不开放 HTTPS 端口。

Web 镜像用 `pnpm deploy --prod` 保留生产依赖，使用 React Router Node 服务，API 为 Linux release 二进制。入口使用官方 `traefik:v3.7.13` 镜像，固定发布 `30075:8080`。配置位于 `infra/traefik`，file provider 保留 API 路径前缀、页面走 SSR；没有 Docker socket、公开 dashboard 或证书卷。版本依据：[Traefik 3.7.13](https://github.com/traefik/traefik/releases/tag/v3.7.13)、[PostgreSQL 18.6](https://www.postgresql.org/docs/release/18.6/)。

媒体持久卷和备份/恢复尚待部署阶段补齐。旧 Caddy 镜像、配置和证书卷声明已移除；用户既有卷不会因修改 Compose 被删除。TLS/DNS/路由器设置尚未修改。

## 下一阶段

认证采用 `axum-login 0.18.0` 配套的 `tower-sessions 0.14.0`，避免与 0.15 创建两套 Session 类型（[官方依赖清单](https://docs.rs/crate/axum-login/0.18.0/source/Cargo.toml)）。只存会话 ID 的 SHA-256，记录用 `timestamptz` 到期；create 不覆盖冲突、save 不插入，撤销后的旧响应不能恢复记录。CSRF 用系统随机数、常量时间比较和配置的 origin allowlist，不从代理 header 推断可信 origin。密码哈希通过有限并发的 blocking worker 执行；登录轮换会话，密码恢复撤销所有旧会话，定时清理过期记录。

账号通过管理员 CLI 发出一次性邀请；恢复也由管理员确认邮箱后生成链接，没有尚未配置的邮件发送入口。设置 `DATABASE_URL`、`PUBLIC_APP_URL` 并执行迁移后运行：

```sh
cargo run -p brioche-server -- invite learner@example.com .local/invite-link.txt
cargo run -p brioche-server -- reset-password learner@example.com .local/reset-link.txt
```

输出文件必须尚不存在；链接只写入该私有文件，不输出到日志。邀请有效 48 小时，恢复有效 30 分钟；重新签发撤销同类旧链接。链接 token 放在 URL fragment 中，页面读取后移除，数据库只保留 SHA-256。生产容器可用 `docker compose exec server brioche-server invite learner@example.com /tmp/invite-link.txt`，管理员私下读取和交付，再删除该文件。Unix 创建权限为 0600；Windows 输出位置应使用管理员私有目录。`--operator` 仅用于邀请授予管理员角色；当前尚未提供后台页面。不要提交、截图或公开链接文件。

登录用户通过 `POST /api/v1/learning-sessions` 开始或恢复当前课程，通过 `GET /api/v1/learning-sessions/:id` 读取固定版本与进度。`PUT .../steps/:stepId` 确认步骤、`POST .../attempts` 提交答案、`POST .../hints/:exerciseId` 记录提示、`POST .../complete` 完成本课；写入携带版本和幂等键，答案只由 Rust 判分。`GET /api/v1/me/learning` 提供每课最近记录与游标分页。所有读写验证会话所有者，写入同时验证 Origin/CSRF。完成要求必需步骤确认与必需题目尝试，不要求全部答对；首次完成时间保持不变。

步骤、判分记录、幂等结果和完成时复习卡片在事务中保存。相同键/载荷返回原结果，不同载荷拒绝；旧版本返回 409；固定快照撤回后返回 410，包括原幂等结果。浏览器遇到未确认的提交保留原请求和答案，明确重试原键，不自动生成第二次尝试。当前重试保留限于当前页面，刷新或离开后的未提交草稿恢复仍待补齐。完整 release/硬撤回管理工具尚待实现。

独立 `/practice/:lessonId` 和 `/review/:lessonId` 仍为演示流程，不保存账号复习自评；账号学习完成生成复习卡片，账号 `/reviews` 自评已持久化；收藏与其他验收缺口仍按 [实现清单](09-implementation-tracker.md) 继续实施。

示例练习通过 `POST /api/demo/lessons/:id/grade` 调用 Rust 判分，仅在服务端启用 development fixture 时可用，不写数据库；数据库模式返回 404，不替代未来受认证/CSRF 保护的学习提交。请求必须携带匹配 Host 的 Origin，限定版本、题目 ID、答案类型、选项/词块范围与 body 大小。填空规范化 NFC、空白、大小写（按题配置）和法语弯引号，保留重音差异。规则源只在 Rust 服务端加载，生成 TS/前端 bundle 不含答案键。Unicode 处理依据 [unicode-normalization 文档](https://docs.rs/unicode-normalization/0.1.25/unicode_normalization/)。

速度弹窗显式使用 fixed/inset/auto margin 居中，避免 Tailwind reset 覆盖原生 dialog 的默认 margin；最大高度考虑动态视口与安全区，内部滚动不占额外宽度。

依赖兼容依据：[React Router Framework](https://reactrouter.com/start/framework/installation)、[Vite 8](https://vite.dev/blog/announcing-vite8)、[SeaORM 发布记录](https://github.com/SeaQL/sea-orm/releases)。具体依赖以提交的 Cargo.lock / pnpm-lock.yaml 为准。

## 账号复习

`GET /api/v1/me/reviews` 按当前学习时区与服务器时间返回最多 10 张已到期、未暂停且来源仍发布的卡片，以及总到期数/下一到期时间。可提供非未来 `date=YYYY-MM-DD` 过滤当前卡片的到期时间，不重建历史排程。队列与计数使用同一数据库快照。

`GET /api/v1/me/reviews/:id` 读取所属卡片；`POST .../:id/attempts` 接受 `cardVersion`、`idempotencyKey`、`rating`（again/remembered/familiar）。自评、旧/新档位、时区、算法版本、下次时间与原幂等结果在同一事务保存；不接受客户端档位、时间或分数。尚未到期的卡片拒绝重复排期。撤回来源阻断卡片读取与幂等重放；改学习时区不会移动已保存的 UTC 到期时间，之后提交使用新时区。

固定算法按本地日期增加 1/3/7/14/30 天，并将目标日期 09:00 转为 UTC。时区跳变采用 Jiff compatible 规则，依据 [Jiff 时区与歧义处理](https://docs.rs/jiff/0.2.37/jiff/)。网页延用整卡展开/朗读、纵向自评和展开动画；不确定保存可明确重试相同请求。当前页离开后的未确认请求恢复仍需补齐。
