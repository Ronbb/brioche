# Brioche

面向中文母语自学者的法语学习 Web App。通过文章、对话和生活任务，把法语学习放进真实的日常场景。

当前处于**产品与架构设计阶段**，尚未建立应用、安装应用依赖或配置实际部署。名称暂沿用目录名 Brioche（布里欧修）。

从 [设计文档索引](docs/README.md) 开始阅读。初版范围为 A1–A2、多个用户、Web + 独立服务端；开发时本机直接运行，生产通过本机 Docker、Cloudflare DNS 与路由器端口映射提供访问。

技术方向：Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router；Rust/Axum + SeaORM + PostgreSQL。SSR 由 React Router 管理，Rust 保持唯一业务后端。

- [产品与课程规划](docs/01-product.md)
- [系统架构与技术栈](docs/02-architecture.md)
- [课程解释器与数据契约](docs/03-course-model.md)
- [界面与插图设计](docs/04-experience-design.md)
- [部署与运维设计](docs/05-deployment.md)
- [开发顺序与验收](docs/06-roadmap.md)

界面 Preview 已完成当前一轮设计确认，见 [预览说明](docs/preview/README.md)。在项目根目录运行 `python -m http.server 4173 --directory docs/preview`，然后打开 `http://localhost:4173/`。这仍是静态交互原型，尚无账号、服务端或学习记录持久化；法语内容与正式媒体仍需审校。

协作约定见 [AGENTS.md](AGENTS.md)；本机信息见不进入版本控制的 `AGENTS.local.md`。文档中的命令和目录结构，除特别说明外，均为后续实现目标，当前不可当作已实现功能。
