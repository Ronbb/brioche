# ADR 0001：初版产品与架构基础

日期：2026-10-05。状态：用户指定前后端方向，具体工程方案建议采用。用户已确认的约束见 docs/README.md。

## 决定

用用户指定的 Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router 构建响应式 Web，Rust/Axum 提供独立 API，SeaORM + PostgreSQL 存关系状态和 JSONB 课程快照。建议 React Router Framework Mode 管理 Vite SSR。前端 pnpm、后端 Cargo workspace，通过 Rust 类型导出的 Schema/TS/OpenAPI 共用契约。生产本机 Docker Compose + Caddy，Cloudflare 管理 DNS，开发进程本机运行。

课程用 JSON Schema 声明的 block AST 和 step flow，服务端保留判题规则，浏览器只接收公共 DTO。文件是可审查的课程编辑源，发布生成不可变 revision 与整批目录 release。

## 理由与代价

明确的 Rust server 边界适合后续新增客户端、导入工具和内容任务，符合用户服务端偏好。React SSR 保留独立 Node 渲染进程，不把用户鉴权、判题和持久化业务复制到 Web。代价是两种语言的工具链、Web/API 两个进程、内部 SSR 请求和生成契约。服务端保持单体，限制为简单部署和单数据库事务。

不使用 Next.js，这是用户明确偏好。Vite 原生支持 SSR，但其底层接口不是一套完整的生产路由/渲染服务器；React Router Framework Mode 负责这些职责，避免手写 SSR 生命周期。也可使用 React Router Data Mode + 自定义 Vite SSR，但当前没有需要承担该维护量的定制要求。

Rust 前端（Leptos/Yew）被允许作为备选，但先采用用户具体指定的 React 技术栈，适配文本学习交互及组件资源更直接。需要 WASM 计算时可局部引入，不以服务端 Rust 推导前端也必须 Rust。若后续完全无需 SSR/公开页面，可关闭 SSR 并将 Web 静态文件交给 Caddy，减少运行进程。

用户认可 SeaORM 等 Rust ORM，选择 SeaORM 处理关系数据和事务，保留 SeaQuery/raw SQL 处理复杂查询。ORM Entity 不作为公开 API 模型；生产使用显式迁移，不自动 schema sync。代价是需要理解生成 SQL、避免 N+1 并验证 session store 与 ORM 版本配套。

JSON AST 限制作者自由排版，但能校验引用、复用解释、隐藏私有判分和保持一致的无障碍界面。与 MDX 相比，新增交互需要显式扩展类型；这是接受的维护成本。实现期 Rust Serde 类型为真源，schemars 导出 Schema，再生成 TS，减少跨语言漂移；设计期 Schema 留作可审查快照。

发布快照同时保留公共/私有文档，增加少量存储，但保证旧会话和复习不会随内容编辑失效。首版没有在线 CMS，内容作者需文件编辑和 CLI 发布；当作者数量增长且工作流成为瓶颈，再增加编辑后台。

## 重新评估条件

- API 与页面职责长期没有独立价值，维护两个进程成本明显：评估合并。
- 多实例 API 或异步任务需要共享限流/队列：引入专用服务，先测量而非预设。
- 媒体量、带宽或备份速度成为瓶颈：迁移 S3 兼容存储，保持 asset ID/哈希契约。
- 复习用户量和长期效果数据足够：比较固定档位与更成熟的调度算法。
- Cloudflare/公网入口不能满足本机端口映射：调整接入方式，保留同源 API 和 TLS 边界。

技术系列来自当日官方资料，进入开发时再复核稳定版本和依赖兼容，不以本 ADR 固定未来补丁号。
