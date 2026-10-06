# Brioche

面向中文母语自学者的法语学习 Web App。通过文章、对话和生活任务，把法语学习放进真实的日常场景。

当前处于**工程实现与验收阶段**：已接入邀请认证、个人设置、固定版本课程学习与续学、三类练习判分、收藏与复习排程、学习概览，以及课程批次发布/回滚/撤回、登记媒体和管理员预览。名称暂用 Brioche（布里欧修）。完整状态与证据见 [实现与验收清单](docs/09-implementation-tracker.md) 和 [验证记录](docs/07-design-verification.md)；人工教学审校、正式录音、真实 iPhone 和生产验收仍未完成。

从 [项目文档索引](docs/README.md) 开始阅读。初版范围为 A1–A2、多个用户、Web + 独立服务端；开发时本机直接运行，生产通过本机 Docker、Cloudflare DNS 与路由器端口映射提供访问。

技术方向：Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router；Rust/Axum + SeaORM + PostgreSQL。SSR 由 React Router 管理，Rust 保持唯一业务后端。

- [产品与课程规划](docs/01-product.md)
- [系统架构与技术栈](docs/02-architecture.md)
- [课程解释器与数据契约](docs/03-course-model.md)
- [界面与插图设计](docs/04-experience-design.md)
- [部署与运维设计](docs/05-deployment.md)
- [开发顺序与验收](docs/06-roadmap.md)

开发启动与验收见 [工程说明](docs/08-development.md)。安装 `pnpm install --frozen-lockfile` 后，在两个终端分别运行 `pnpm dev:api`、`pnpm dev:web`，打开 `http://localhost:5173/`。开发模式提供未审校示例课程，生产仅读取数据库已发布版本。

生产使用 Docker Compose，Traefik 作为内部入口，数据库不映射宿主端口。基础配置可单独提供 HTTP30075；当前已接独立 HTTPS 网关，生产叠加 [compose.https.yaml](compose.https.yaml) 取消宿主HTTP映射，维护方式见 [HTTPS接入说明](infra/https-gateway.md)。实际域名、数据库密码与浏览器地址 PUBLIC_APP_URL 仅放部署私有配置。

保留已确认的静态 [Preview](docs/preview/README.md) 作为视觉基准；通过 `python -m http.server 4173 --directory docs/preview` 查看。法语内容与正式媒体仍需审校。

协作约定见 [AGENTS.md](AGENTS.md)；本机信息见不进入版本控制的 `AGENTS.local.md`。早期设计提案保留规划范围；实际启动、作者工具和验收命令以 [工程说明](docs/08-development.md) 为准，不能把提案或草稿当作已验收结果。
