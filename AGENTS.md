# Brioche 协作约定

## 项目阶段与阅读顺序

当前进入工程开发，已有 React Router SSR Web、Rust 课程读取/判分 API、邀请认证、跨设备个人设置、固定版本学习提交/续学与 SeaORM 迁移。先读 `docs/README.md`、`docs/08-development.md` 和 `docs/09-implementation-tracker.md`，再按任务读对应文档。已有账号复习排程、收藏、实际学习概览、目录 release 的原子切换/回滚/硬撤回及视觉素材/角色库发布校验；录音媒体与 staging 预览仍待实现，不要把计划或设计稿当成已实现功能。

若存在 `AGENTS.local.md`，也读取它；它是本机补充资料，应被显式读取，不假定工具自动加载。用户当次指令和已确认偏好优先于这里的约定。

## 已确认方向

- 中文母语自学者，A1–A2，多用户。
- 等级 → 单元 → 课程，正文以文章/对话为主体，解释/词汇/语法为结构数据。
- Web only + server；开发本机直接运行，生产本机 Docker Compose、Cloudflare DNS 与路由器端口映射。
- Compose 网关采用 Traefik，仅对外映射 HTTP 30075；HTTPS 由用户处理，不加入自动证书或 HTTPS 端口。
- 界面美观且有情境插图；前台不暴露数据 Schema、部署方式等实现术语。
- 服务端优先 Rust；前端 Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router，不使用 Next.js。
- 持久化采用 SeaORM + PostgreSQL；Entity 与公共 DTO 分开，复杂查询可显式 SQL，事务和用户权限不能交给 ORM 自动推断。

## 实现原则

- 领域规则集中在服务端；课程解释器不执行作者代码，不接受任意 HTML/MDX/JS。
- Rust/TS 共享生成的公共契约；Rust Serde 类型是实现期真源，Schema/TS/OpenAPI 生成物不得手改。Web 不导入答案、数据库代码和 secrets。
- 内容 revision 发布后不可变，学习会话固定版本；跨账号访问、缓存和并发写入需要明确验证。
- 使用当前稳定技术，核对官方资料和 peer dependencies；提交 lockfile，不用预发布版或浮动生产镜像。
- 未知交互类型明确报错，不静默丢弃必需内容。
- 密码、session、邀请/恢复 token、生产地址/秘密不写 Git 或日志；不要将本机配置硬编码进应用。
- SeaORM 生产使用显式迁移，审查实际 SQL，不在启动时 schema sync；事务/关系验证使用 PostgreSQL。密码哈希等重 CPU 工作不得阻塞 Tokio async executor。

## 内容与设计

- 课程中的法语、中文译文、语法解释及文化断言需要审校，素材来源/授权可追溯。
- 完成进度与掌握程度分开；习惯任务可选，无强制解锁/现实行为证明。
- 使用响应式排版、键盘操作、法语/中文 lang 标注、清晰保存状态和缺媒体回退。
- `docs/preview` 是概念稿，不把静态交互替代真正的业务实现。
- 品牌采用用户参考色：暖橙 #FFA62F、浅金 #FFC96F、奶油 #FFE8C8、嫩绿 #ACD793，阅读底色 #FFFAEF，文字深棕/深绿；Apple 系统字体优先，面向 iPhone 安全区与触控设计。操作图标使用 SVG，不使用 Unicode 字符图标。
- 单词、头像整句、全文朗读共享一个播放器；头像同时展开译文，角色介绍只在对话前展示，正文只留头像，短文采用纯段落。全局译文放设置页，全文播放器为整条可点按的细进度线，长按自定义选项面板调速，避免系统 Select。角色来自角色库并在发布时固定快照。课程不添加操作教学提示或泛泛鼓励文案。
- 启动 Web/API/预览服务默认监听所有网络接口，便于手机同步查看；数据库仍保持本机/内部网络访问。不要把开发机 IP 固定进共享配置。

## 修改与验证

- 改关键设计同步更新 docs 与决策记录，标明事实、建议、待验证项。
- 课程先结构/语义校验，再预览与人工审校；结构通过不表示教学内容正确。
- 权限、判分、事务、幂等、版本与恢复做风险对应的验证；文档/低风险样式不添加无意义测试。
- 实际命令：`pnpm dev:api`、`pnpm dev:web`、`pnpm contracts`、`pnpm typecheck`、`pnpm build`；Rust 使用 cargo fmt/check/clippy/test。PostgreSQL 集成测试显式设置专用 `TEST_DATABASE_URL` 后运行，禁止指向生产。TS 7 类型检查不能用 Vite build 代替。
- 不自行修改用户 DNS/路由器或执行生产上线；按相应任务的授权范围工作。
