# Brioche

面向中文母语自学者的法语学习 Web App。通过文章、对话和生活任务，把法语学习放进真实的日常场景。

当前进入**工程开发阶段**：已有 React Router SSR Web、Rust 课程 API、三类示例练习判分、SeaORM 迁移，以及邀请注册、登录、退出和密码恢复。学习进度与个人偏好保存尚未实现。名称暂用 Brioche（布里欧修）。

从 [设计文档索引](docs/README.md) 开始阅读。初版范围为 A1–A2、多个用户、Web + 独立服务端；开发时本机直接运行，生产通过本机 Docker、Cloudflare DNS 与路由器端口映射提供访问。

技术方向：Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router；Rust/Axum + SeaORM + PostgreSQL。SSR 由 React Router 管理，Rust 保持唯一业务后端。

- [产品与课程规划](docs/01-product.md)
- [系统架构与技术栈](docs/02-architecture.md)
- [课程解释器与数据契约](docs/03-course-model.md)
- [界面与插图设计](docs/04-experience-design.md)
- [部署与运维设计](docs/05-deployment.md)
- [开发顺序与验收](docs/06-roadmap.md)

开发启动与验收见 [工程说明](docs/08-development.md)。安装 `pnpm install --frozen-lockfile` 后，在两个终端分别运行 `pnpm dev:api`、`pnpm dev:web`，打开 `http://localhost:5173/`。开发模式提供未审校示例课程，生产仅读取数据库已发布版本。

生产使用 Docker Compose，Traefik 作为入口网关，**仅暴露 HTTP 30075**；HTTPS 由用户在外部处理。页面与 `/api` 共用这个入口，数据库不映射宿主端口。复制 `infra/production.env.example` 为 `.env` 并设置随机数据库密码和浏览器访问地址 `PUBLIC_APP_URL` 后，按工程说明构建和启动。

保留已确认的静态 [Preview](docs/preview/README.md) 作为视觉基准；通过 `python -m http.server 4173 --directory docs/preview` 查看。法语内容与正式媒体仍需审校。

协作约定见 [AGENTS.md](AGENTS.md)；本机信息见不进入版本控制的 `AGENTS.local.md`。文档中的命令和目录结构，除特别说明外，均为后续实现目标，当前不可当作已实现功能。
