# Brioche

面向中文母语自学者的法语学习 Web App。通过文章、对话和生活任务，把法语学习放进真实的日常场景。

当前处于**工程实现与验收阶段**：已接入邀请认证、个人设置、固定版本课程学习与续学、三类练习判分、收藏与复习排程、学习概览，以及课程批次发布/回滚/撤回、登记媒体和管理员预览。名称暂用 Brioche（布里欧修）。完整状态与证据见 [实现与验收清单](docs/09-implementation-tracker.md) 和 [验证记录](docs/07-design-verification.md)；48课及正式预生成录音已发布；人工教学审校、真实 iPhone 和完整生产验收仍未完成。

从 [项目文档索引](docs/README.md) 开始阅读。初版范围为 A1–A2、多个用户、Web + 独立服务端；开发时本机直接运行，生产通过本机 Docker、Cloudflare DNS 与路由器端口映射提供访问。

技术方向：Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router；Rust/Axum + SeaORM + PostgreSQL。SSR 由 React Router 管理，Rust 保持唯一业务后端。

- [产品与课程规划](docs/01-product.md)
- [系统架构与技术栈](docs/02-architecture.md)
- [课程解释器与数据契约](docs/03-course-model.md)
- [界面与插图设计](docs/04-experience-design.md)
- [部署与运维设计](docs/05-deployment.md)
- [开发顺序与验收](docs/06-roadmap.md)

开发启动与验收见 [工程说明](docs/08-development.md)。安装 `pnpm install --frozen-lockfile` 后，在两个终端分别运行 `pnpm dev:api`、`pnpm dev:web`，打开 `http://localhost:5173/`。开发模式提供未审校示例课程，生产仅读取数据库已发布版本。

生产使用 [compose.product.yaml](compose.product.yaml) 引用 Chef 的独立身份、学习 API、Web 和内部 Traefik，共用 PostgreSQL 与媒体卷，外部网关处理 HTTPS；不映射宿主 HTTP 或数据库端口。使用 `docker compose --env-file .local/chef-runtime.env -f compose.product.yaml up -d --wait` 启动应用，私有配置含固定镜像、数据库角色、域名与服务密钥。旧 combined `compose.yaml` 仅保留数据库基础设施和历史恢复参考，不再用于启动已分离 schema 的应用；不要对旧项目执行 `--remove-orphans` 或 `down`。迁移与上线证据见 [Hargow 部署记录](https://github.com/Ronbb/hargow/blob/main/docs/production-20261008.md)。

保留已确认的静态 [Preview](docs/preview/README.md) 作为视觉基准；通过 `python -m http.server 4173 --directory docs/preview` 查看。法语内容与正式媒体仍需审校。

协作约定见 [AGENTS.md](AGENTS.md)；本机信息见不进入版本控制的 `AGENTS.local.md`。早期设计提案保留规划范围；实际启动、作者工具和验收命令以 [工程说明](docs/08-development.md) 为准，不能把提案或草稿当作已验收结果。

共享学习/账号/后台/配音后端与数据库迁移已迁入固定 Chef 子模块；本产品 Rust 入口只负责启动装配。首次检出执行 `git submodule update --init --recursive`。2026-10-08 已完成生产身份 schema 分离与双产品部署，原两账号、48课及发布状态保持，Hargow 学习数据按产品隔离。原拆分设计见 [多产品架构](docs/14-multi-product-architecture.md)。

通用 React Router 页面、播放器和管理员界面已迁入 `framework/packages/web`，本产品 Web 保留品牌配置、构建和静态品牌资源，测试通过 Chef 执行。正式运行使用独立身份与按产品授权的学习服务。

法语课源真源为固定的 `curriculum` 子模块（独立 brioche-courses 仓库）。课程、作者示例、历史目录不再在产品保存副本；作者命令使用 `curriculum/docs/...`，生产仍读取既有数据库中的不可变版本。
