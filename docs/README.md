# 设计文档索引

设计日期：2026-10-05（Asia/Shanghai）。状态：v0.1 设计提案，供讨论和后续实现。

| 文档 | 解决的问题 |
| --- | --- |
| [产品与课程规划](01-product.md) | 谁来学、学什么、如何形成日常学习闭环、首版做到哪里 |
| [系统架构与技术栈](02-architecture.md) | 前后端职责、模块边界、数据库、API 和技术选型 |
| [课程解释器与数据契约](03-course-model.md) | 如何用结构化数据描述课程、关联知识、校验和发布 |
| [界面与插图设计](04-experience-design.md) | 页面布局、交互、视觉语言、插图和无障碍 |
| [部署与运维设计](05-deployment.md) | 本机开发、Docker、Cloudflare、TLS、备份和恢复 |
| [开发顺序与验收](06-roadmap.md) | 从设计进入开发的阶段目标、验证方法与待定事项 |
| [设计验证记录](07-design-verification.md) | 哪些草案与交互实际检查过，哪些留待实现 |
| [工程开发说明](08-development.md) | 当前已实现范围、实际启动与检查命令、Docker Compose |
| [实现与验收清单](09-implementation-tracker.md) | 完整功能目标的状态、缺口与验收证据 |
| [决策记录](decisions/0001-foundation.md) | 关键选择的依据、代价及重新评估条件 |
| [示例课程](examples/a1-bakery.lesson.json) | 用一堂面包店课程检验数据设计 |
| [A1 六个单元草稿包](content/a1/README.md) | 24 课作者源文件、固定顺序、结构检查与待审校/素材记录 |
| [目录 release 示例](examples/catalog.release.json) | 显式名称、课程 revision 与教学顺序；引用未审校示例，不能直接发布 |
| [素材与角色清单示例](examples/asset-bundle.json) | 图片哈希、尺寸、来源/授权与角色快照；保持 planned/未确认授权，不能直接导入 |
| [示例课程 Schema](examples/lesson.schema.json) | v0.1 数据契约草案；不等于已实现解释器 |
| [生成的作者课程 Schema](generated/author-lesson.schema.json) | Rust 公共 DTO、私有判分、审校信息和素材引用组成的作者结构契约；不包含语义或发布审校证明 |
| [界面概念稿](preview/index.html) | 可切换首页、阅读、练习、复习与个人设置的静态交互提案 |

## 已确认约束

- 中文母语自学者，优先 A1–A2，支持多个用户。
- 等级 → 单元 → 课程，三层课程组织；正文以真实文章、对话为主体。
- 解释、额外说明、词汇和语法都属于课程结构化数据。
- Web only + server；生产本机 Docker，使用 Cloudflare DNS 和路由器静态端口映射；开发直接本机启动。
- 使用当前稳定、适合长期维护的现代技术；界面美观并有插图。
- 后端优先 Rust；前端选择 Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router，不使用 Next.js。
- 设计文档保留为提案与视觉基准，未实施公网部署。
- 已获准开始开发，工程现状以工程开发说明为准；公网部署仍未实施。生产使用 Docker Compose。

## 本提案采用的默认选择

名称暂用 Brioche；后端采用 Rust/Axum 模块化单体，持久化采用 SeaORM + PostgreSQL；前端采用用户指定栈，建议 React Router Framework Mode 管理 Vite SSR；初次上线采用邀请制账号。CEFR 是组织学习目标的参考，并不把课程完成率当作官方等级认证。完整内容规划覆盖 A1–A2，工程 MVP 先制作 A1 的三个单元。

这些是设计建议，不代表用户已逐项确认。调整时更新关联文档和决策记录，保留已确认约束。外部资料核对日期见架构和部署文档；具体依赖版本以进入开发时复核并提交的 lockfile 为准。


媒体发布诊断进展（2026-10-06）：release-stage 现保留 release 原文件行列和课程条目路径，并附上已导入课程 `/media/{index}`、`/cast/{index}`、`/audio/{index}` 的发布失败位置。视觉文件缺失/不可读与哈希不匹配分别说明，角色未登记版本与快照不匹配分别说明；录音描述、来源校验、文件、解码和解码元数据也有分项消息。导入课程路径属于数据库中固定投影，不冒充原作者文件的行列。activate 与公共 API 仍只收到原有不透明 AppError；数据库/文件系统原始错误不输出。独立 PostgreSQL 的真实 CLI 验证图片损坏/缺失及两类角色错误的 release 行列定位与无 release/entries/audit 新增，录音集成测试验证损坏/缺失诊断、事务回滚及正常恢复后 staging。32 项 server 单元测试、9 项作者 CLI 测试通过；完整基础设施定位、人工审校、A2 与生产/设备验收继续待完成。


表达库分页进展（2026-10-06）：隔离 PostgreSQL + 实际 API/React Router 页面创建 23 条收藏和 23 张复习卡，核对 20→3 分页、无重复/遗漏、浏览器返回标题焦点；第二页暂停/恢复保持记录。修复取消分页全部收藏后误用全库空态，现显示“这一页没有收藏了”并可返回第一页；复习分页空态也区分并提供返回列表。修复暂停/恢复按钮保存期间禁用导致焦点落到 body，改为 aria-disabled/aria-busy 并在点击入口拒绝阻塞状态写入；实际 8 秒延迟期间重复 Enter 只发一次请求，保存后焦点保持按钮、卡片版本只增加 1。真实删除后的下一条/末条空态与 Tab→返回列表 20 条均验证，320/390px 无横向溢出。屏幕阅读器、真实 iPhone、其他错误组合仍待完成。
