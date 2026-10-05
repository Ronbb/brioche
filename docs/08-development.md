# 第一轮工程实现

更新日期：2026-10-06。这是工程基础与课程阅读链路，尚未完成多用户学习产品。

## 已实现

- pnpm / Cargo workspace、锁文件、Rust 1.99.0 工具链、TypeScript 7 独立类型检查。
- React Router Framework Mode + Vite SSR；正式客户端路由 `/`、`/lessons/:lessonId`、`/review/:lessonId`、`/profile`。
- 迁移确认过的视觉与主要阅读交互：角色介绍、头像整句、点词朗读和词汇解释、短文、全文播放/暂停/长按调速、复习卡片、统一设置、toast、动效和覆盖式滚动条。
- Axum 公共目录和课程 API、health/readiness、SIGINT/SIGTERM 优雅退出。
- Rust Serde 公共 DTO，生成 TS 联合类型和公共课程 JSON Schema；Web 只导入公共契约。私有答案与编辑状态不进入课程响应。
- SeaORM PostgreSQL Entity、版本化显式迁移、仅插入的课程导入工具、草稿过滤及最新发布 revision 读取。
- Docker Compose：PostgreSQL → 一次性迁移 → API → SSR Web → Caddy。数据库不映射宿主端口，生产关闭示例课程模式。

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
```

测试在独立、随机命名的 schema 中执行迁移、发布读取与唯一约束验证。普通测试运行会跳过它；CI 使用隔离的 PostgreSQL 服务执行。测试失败可能留下该测试 schema，禁止在生产数据库运行。

## Docker Compose

用户已确认部署可使用 Docker Compose。复制 `infra/production.env.example` 为根目录 `.env`，填写域名和随机数据库秘密；密码使用字母数字或正确 URL 编码。然后先检查并构建：

```sh
docker compose config --quiet
docker compose build
```

准备好生产参数和已审校课程后，启动方式为 `docker compose up -d`；不要在缺少域名/端口/备份方案时把本机验证当作公网发布。

Web 镜像用 `pnpm deploy --prod` 保留生产依赖，使用 React Router Node 服务，API 为 Linux release 二进制。Caddy 2.11.7 使用官方 release 二进制及其 SHA-512 清单构建，因为初始化时对应 Docker Hub 标签尚不可用。版本依据：[Caddy 发布记录](https://github.com/caddyserver/caddy/releases/tag/v2.11.7)、[PostgreSQL 18.6](https://www.postgresql.org/docs/release/18.6/)。

当前 Caddy 配置支持常规 ACME 验证；只开放非标准外网端口时的 Cloudflare DNS-01 插件、映射测试、媒体持久卷和备份/恢复尚待部署阶段补齐。TLS/DNS/路由器设置尚未修改。

## 下一阶段

账号邀请/登录、cookie 会话与 CSRF、服务端练习判分、学习会话固定 revision、幂等提交、进度续学、账号复习排程、个人资料保存尚未实现。现在为访客阅读与临时自评，不宣称保存到账号。结构化课程已能提供正文和知识锚点，完整步骤解释器与全部教学块渲染继续按路线图实施。

依赖兼容依据：[React Router Framework](https://reactrouter.com/start/framework/installation)、[Vite 8](https://vite.dev/blog/announcing-vite8)、[SeaORM 发布记录](https://github.com/SeaQL/sea-orm/releases)。具体依赖以提交的 Cargo.lock / pnpm-lock.yaml 为准。
