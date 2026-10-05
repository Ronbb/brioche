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
- 账号学习：`/learning/:sessionId` 按结构数据遍历所有步骤和正文块；服务端固定课程 revision，保存步骤、提示、首次及重试答案，确认完成后生成去重复习卡片。首页显示真实续学入口和课程完成记录。账号 `/reviews` 提供每批最多 10 项到期队列、自评与保存回顾；收藏与复习管理见下文。
- 学习概览：`GET /api/v1/me/dashboard` 在一致性快照中读取个人目标、周学习事实、到期数量/下一次复习与实际续学。按个人 IANA 时区的周一至周日统计步骤确认、练习提交、复习自评和首次完成，打开页面/创建会话与幂等重试不产生额外活动。每天分钟数是设定的目标，没有假装测量已学时长；撤回课程保留历史完成总数，但屏蔽正文、续学和复习入口。续学读取全部最新课程状态，不受 20 项概览分页截断。首页按真实等级/单元组织目录并使用固定版本续学内容；推荐按未学过优先、active release 的明确教学顺序排列。

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

导入始终创建不可见 revision；旧的 `--publish` 参数被明确拒绝，改用目录 release 原子发布。本示例未审校，禁止为测试上线而直接改状态。相同 `(lesson_id, revision)` 重复导入失败，数据库触发器也拒绝改写或删除已有正文/私有答案；审校后重新导入需要新 revision。

### 目录 release 操作

`docs/examples/catalog.release.json` 展示显式等级/单元名称与课程 revision 的顺序。它引用未审校示例，正常 stage 会拒绝，不能作为正式发布包。生产模式没有 active release 时目录为空；迁移不自动把历史 published 记录当作审校并启用。

对已完成审校的正式内容，本地管理员 CLI 支持以下流程（`actor` 是操作者记录，不是自动验证过的账号身份）：

```sh
cargo run -p brioche-server -- release-status
cargo run -p brioche-server -- release-stage <manifest.json> <actor> <reason>
cargo run -p brioche-server -- release-activate <release-id> <expected-generation> <actor> <reason>
cargo run -p brioche-server -- content-withdraw <lesson-id> <revision> <expected-generation> <actor> <reason>
```

stage 校验整个清单、唯一 ID/引用、正文与私有规则一致、结构/语义和审校状态。清单顺序与每个私有完整源 JSON 的 SHA-256 共同组成 release 内容哈希；JSON 空白/对象键顺序不影响哈希，数组顺序保留。整批入库与审计同事务，stage 不公开目录。发布要求当前 generation 一致，在事务中启用引用版本、切换唯一指针和写审计；原包、目录条目与审计不可改写。回滚用相同 activate 命令选择以前的 release，新会话跟随指针，已有会话与复习继续固定旧 revision。硬撤回记录不可逆，阻止正文/新提交/复习与旧成功响应重放；包含撤回 revision 的 release 不能再激活。撤回后目录过滤空单元/等级，历史事实保留。空清单可显式停止新课程入口。

**仍待完成**：媒体实际文件哈希/授权和角色库快照发布校验、可预览 staging 页面、完整作者错误定位与课程搜索。本轮 CLI 只证明发布事务边界；内容审核与媒体门槛未全部实现，不能据此宣称正式课程已可上线。

公开 `/api/lessons/:id` 默认读取 active release；`?revision=N` 精确读取曾启用且未撤回的不可变版本，draft 不可见。学习概览同一数据库快照包含目录与推荐，SSR 首页按返回的 revision 取正文，避免发布恰好切换时混合两个版本。普通回滚仍可读取旧公开快照，硬撤回的精确版本返回 410。

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

步骤、判分记录、幂等结果和完成时复习卡片在事务中保存。相同键/载荷返回原结果，不同载荷拒绝；旧版本返回 409；固定快照撤回后返回 410，包括原幂等结果。浏览器遇到未确认的提交保留原请求和答案，明确重试原键，不自动生成第二次尝试。当前重试保留限于当前页面，刷新或离开后的未提交草稿恢复仍待补齐。目录发布与硬撤回由上述 CLI 管理。

独立 `/practice/:lessonId` 和 `/review/:lessonId` 仍为演示流程，不保存账号复习自评；账号学习完成生成复习卡片，账号 `/reviews` 自评已持久化；其他验收缺口仍按 [实现清单](09-implementation-tracker.md) 继续实施。

示例练习通过 `POST /api/demo/lessons/:id/grade` 调用 Rust 判分，仅在服务端启用 development fixture 时可用，不写数据库；数据库模式返回 404，不替代未来受认证/CSRF 保护的学习提交。请求必须携带匹配 Host 的 Origin，限定版本、题目 ID、答案类型、选项/词块范围与 body 大小。填空规范化 NFC、空白、大小写（按题配置）和法语弯引号，保留重音差异。规则源只在 Rust 服务端加载，生成 TS/前端 bundle 不含答案键。Unicode 处理依据 [unicode-normalization 文档](https://docs.rs/unicode-normalization/0.1.25/unicode_normalization/)。

速度弹窗显式使用 fixed/inset/auto margin 居中，避免 Tailwind reset 覆盖原生 dialog 的默认 margin；最大高度考虑动态视口与安全区，内部滚动不占额外宽度。

依赖兼容依据：[React Router Framework](https://reactrouter.com/start/framework/installation)、[Vite 8](https://vite.dev/blog/announcing-vite8)、[SeaORM 发布记录](https://github.com/SeaQL/sea-orm/releases)。具体依赖以提交的 Cargo.lock / pnpm-lock.yaml 为准。

## 账号复习

`GET /api/v1/me/reviews` 按当前学习时区与服务器时间返回最多 10 张已到期、未暂停且来源仍发布的卡片，以及总到期数/下一到期时间。可提供非未来 `date=YYYY-MM-DD` 过滤当前卡片的到期时间，不重建历史排程。队列与计数使用同一数据库快照。

`GET /api/v1/me/reviews/:id` 读取所属卡片；`POST .../:id/attempts` 接受 `cardVersion`、`idempotencyKey`、`rating`（again/remembered/familiar）。自评、旧/新档位、时区、算法版本、下次时间与原幂等结果在同一事务保存；不接受客户端档位、时间或分数。尚未到期的卡片拒绝重复排期。撤回来源阻断卡片读取与幂等重放；改学习时区不会移动已保存的 UTC 到期时间，之后提交使用新时区。

固定算法按本地日期增加 1/3/7/14/30 天，并将目标日期 09:00 转为 UTC。时区跳变采用 Jiff compatible 规则，依据 [Jiff 时区与歧义处理](https://docs.rs/jiff/0.2.37/jiff/)。网页延用整卡展开/朗读、纵向自评和展开动画；不确定保存可明确重试相同请求。当前页离开后的未确认请求恢复仍需补齐。

## 收藏与复习管理

阅读的词汇解释提供收藏与手动加入复习，个人页进入 `/library`（收藏/复习管理）和 `/review-history`（分页历史）。收藏与复习独立；取消收藏不删除复习或历史。收藏保留第一次来源、快照与创建时间；再次收藏不替换它们。

`GET /api/v1/me/saved-items` 返回每页 20 项和游标，`GET .../:knowledgeId` 返回所属状态。`PUT .../:knowledgeId` 接受 sourceLessonId/sourceRevision/saved/version/idempotencyKey；尚不存在使用 version=0，之后版本防覆盖。取消使用 saved=false，保留记录以避免删除后重建造成旧版本覆盖。首次来源必须是已发布课程中的真实词汇；既有快照被撤回时返回元信息并隐藏词汇，仍允许用户取消收藏。

`POST /api/v1/me/review-enrollments` 接受 knowledgeId、来源与幂等键，创建或读取已有复习卡，不重置档位/排期/暂停状态。`GET /api/v1/me/review-cards` 分页读取含暂停卡的所属列表；`PUT /api/v1/me/reviews/:id/preferences` 接受 cardVersion/suspended/idempotencyKey，暂停/恢复只改变标记和版本，保留 UTC 到期时间和档位。

`GET /api/v1/me/review-history` 每页 20 条，显示原自评及提交时区的时间；数据库保留旧/新档位、排程和算法版本。撤回来源时隐藏词汇正文，仍保留历史事实。三个列表游标均以时间与随机 ID 排序，校验格式，绑定当前登录账号。读写受既有认证/Origin/CSRF/private-no-store 保护，写入与原幂等结果同事务保存。浏览器明确重试不确定请求，离开页面后的未确认操作恢复仍待补齐。

## 视觉素材与角色库

迁移 9 注册不可变素材 revision、角色快照及导入审计。`MEDIA_ROOT` 默认 `.local/media`；服务端和内容 CLI 必须使用同一个目录。Compose 的 server 挂载 `media_data` 到 `/var/lib/brioche/media`，镜像创建 UID 10001 可写的目录；备份和恢复必须同时保留 PostgreSQL 与这个卷，实际恢复演练仍待完成。

`cargo run -p brioche-server -- assets-import <bundle.json> <source-directory> <actor>` 读取严格字段的清单，登记素材与角色，文件按 SHA-256 命名。参考 `examples/asset-bundle.json`：它故意保持 planned 与 rightsConfirmed=false，作者/授权未确认，不能直接导入。正式素材必须明确来源、作者、license、中文替代文本/署名、ready 状态与人工确认授权；工具只记录操作者的声明，不能代替授权审核。

图片支持静态 SVG、PNG、JPEG、WebP，逐文件验证实际 MIME、SHA-256、尺寸和完整解码，大小 1–32 MiB，宽高最多 8192。SVG 仅允许静态图形白名单，拒绝脚本、外部引用、事件属性、DOCTYPE 和任意 HTML；栅格解码分配上限 64 MiB。来源路径必须在指定素材目录内，拒绝绝对路径、父级和 symlink 逃逸。角色引用精确头像 revision，头像必须正方形；当前角色语音 locale 为 fr-FR。

课程私有源增加 `assetRefs`，例如 `[{"assetId":"art-bakery-morning","revision":1}]`，同时引用正文所需的全部头像。课程 import 根据注册表填充公共 `media`，移除私有 assetRefs；cast 必须与已注册角色 revision 的完整快照一致。release-stage 和 release-activate 都核对每个场景插图、角色头像、注册描述与存储文件哈希，缺文件或篡改阻止整个发布。

`GET /api/media/<sha256>.<extension>` 只提供仍被已发布课程引用的素材；仅登记和 staging 不会公开。最后一项引用撤回后返回 404；损坏/缺文件返回 503，响应 no-store，禁止 MIME 嗅探，SVG 不执行脚本。读取使用有限 blocking 并发。网页使用结构化尺寸、中文 alt、署名与缺图回退，SSR 接管时也检查已失败的图片。

文件写入先于数据库事务，采用临时文件和不可覆盖的硬链接。数据库导入失败可能留下未引用的哈希对象，它们不会公开；自动垃圾回收尚未实现，勿直接删除仍被旧发布快照引用的文件。录音、时间对齐和流式播放待后续实现。

## 课程浏览与搜索

`/courses` 提供 SSR 的全部课程与 GET 搜索表单，首页“浏览与搜索”可进入，不增加顶部导航。查询保留在 URL `?q=...`，支持刷新、分享与浏览器历史，空结果可回到全部课程。`GET /api/catalog?q=...` 在同一次 active release 目录读取后筛选，保持等级/单元/课程顺序，清除空组。匹配等级标签、单元中文名、课程中法标题与中文摘要，多词要求全部出现；使用 Unicode NFKD、移除组合重音并转小写，兼容省略法语重音与全角输入。这只是搜索宽容处理，不用于判分。

查询最多 120 个 Unicode 字符，拒绝非空白控制字符，百分号等作为普通字符，不构造 SQL LIKE 或作者代码。没有查询返回完整当前目录。未发布、仅 staging、非当前 release 和已撤回课程不会出现在搜索结果。当前使用已读取目录的线性筛选，后续目录规模增大再依据测量增加索引或分页。

## 学习标签页草稿与原请求恢复

账号 `/learning/:sessionId` 在 sessionStorage 中保存练习答案、当前步骤及未确认请求，键包含账号 ID、会话 ID、课程 revision。它只用于此标签页的刷新/离页恢复，不是跨设备进度，也不保存 cookie、CSRF 或密码；关闭标签页后的保留由浏览器会话恢复策略决定。退出成功显式清理当前账号的草稿，切换身份清理旧账号，其他账号不会读取它。

请求在发出前持久化原 path/method/body/version/idempotencyKey。恢复后必须明确重试同一请求，不能自动创建新的尝试；409 拉取服务器最新进度并保留答案，后续确认使用新版本。成功/明确拒绝清除对应 pending；旧页面的迟到响应只能清理相同幂等键，不能删除新请求。页面尚未恢复存储时禁止写入。完成页也可确认未决的完成请求。

恢复数据视为不可信：限制 JSON 大小、会话路径、允许的变更类型、版本/幂等键及固定题目选项/词块，拒绝其他 endpoint、伪造分数、重复/未知词块。练习草稿带最近一次 attempt ID，服务器出现新尝试时保留本地有效草稿并提示冲突，再次确认后才创建新尝试；本次提交明确确认后清理草稿并显示服务器结果。存储不可用会提示；无法持久化 pending 时不发起变更，避免承诺无法提供的刷新恢复。

`pnpm test:web` 使用 Node 24 原生 TypeScript 与 node:test 验证存储隔离、篡改/损坏数据、存储拒绝和迟到响应清理，CI 已接入。完整真实浏览器的刷新、离页、两标签页冲突与响应丢失流程仍待验收；账号复习和收藏的相同能力随后接入，见下节；浏览器端到端验收仍待完成。

## 收藏与复习保存恢复

收藏/取消收藏、手动加入复习、暂停/恢复和复习自评均在请求发出前保存原 endpoint、method、完整 body 与幂等键；按账号和操作目标隔离，回到原控件后明确重试。目标校验限定来源课程 revision、知识 ID、卡片 ID、版本与允许字段，存储数据不能改变 endpoint 或提交分数。迟到响应仅清理对应幂等键，身份/目标切换后旧响应不更新新组件。

复习自评恢复时先确认原请求，成功后重读队列，避免跳过新队列第一张卡；若已确认保存后的队列读取失败，保留已保存结果并单独提示读取失败，不再将已确认操作标为未决。个人页的 `/pending-saves` 提供当前标签页的未确认收藏/复习清单，即使取消收藏或自评生效后原行已从列表移除，仍能确认原请求。该页面不处理账号学习会话草稿，学习页继续独立恢复。

401/403/429、网络错误和 5xx 保留原提交；登录/CSRF/限流恢复后继续使用原 key。400/404/409/410/422 属于明确拒绝，清理对应请求，冲突返回最新状态；主动退出会清理此账号的标签页草稿。此机制不等于跨标签页草稿同步，服务器乐观锁仍是多端冲突的最终依据。

五项 Web 协议测试、TS 7 检查与 Web/SSR build 通过。覆盖 owner/目标/revision 隔离、请求字段和操作种类拒绝、掉出队列的 pending 发现、登录/CSRF/限流保留；浏览器端到端故障/刷新/两标签页测试尚未完成，继续按验收清单推进。

## 隔离浏览器验收数据

`cargo run -p brioche-server --example browser_fixture` 仅接受 TEST_DATABASE_URL 指向 loopback 的 `/brioche_browser_qa` 数据库，执行迁移并创建合成协议课/测试账号。它绕过正式课程审校与素材发布流程，仅用于一次性浏览器测试，不能替代正式内容发布。该 example 使用与 PostgreSQL 集成测试相同的测试 release helper；拒绝其他数据库或 query 参数。账号为 browser-qa@example.test，固定口令仅用于此隔离测试库，源码内可见，不用于生产。

验收可在独立 PostgreSQL 临时容器（55432）、数据库 API（3003、PUBLIC_APP_URL=http://127.0.0.1:5175）和独立 Web（INTERNAL_API_URL=http://127.0.0.1:3003，react-router dev --port 5175）进行，完成后清理这组资源；不替换用户开发进程或生产服务。测试单选/填空草稿刷新、服务器提交后丢失响应、刷新/SPA 离页后重试和两标签页不同答案冲突，同时核对数据库真实尝试数。

## 请求观测与日志轮换

API 在公共课程、身份/学习、媒体路由合并后统一添加观测层。每次请求由服务器生成 128-bit 随机 ID，通过 X-Request-Id 响应头返回；忽略调用者传入的同名 header。INFO 级请求完成日志只包含 request_id、规范 HTTP method、注册路由模板、status 和 duration_ms。路由参数不记录，未匹配请求记为 <unmatched>；不记录原始 URL/query、请求体、cookie、Authorization、CSRF 或用户资料。随机源不可用时明确返回服务不可用并写固定错误文本。

耗时度量从进入路由中间件到产生响应头，包含处理与数据库等待，不代表网络下载结束。日志暂用于排查单次请求，指标采集、告警及性能基线仍待建立。日志由 RUST_LOG 控制；默认 brioche_server=info。此前只包围公共路由的通用 TraceLayer 已移除，避免高日志级别意外记录原始 URI。

Compose 所有五个服务使用 Docker local 日志驱动，配置 max-size=10m、max-file=3，限制单个容器的保留日志。宿主 Docker local 驱动已确认可用，Compose 解析验证每个服务均应用该配置；本轮没有执行生产容器重建或声称实际磁盘轮换演练完成。仍可使用 docker compose logs 查看日志。
