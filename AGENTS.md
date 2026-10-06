# Brioche 协作约定

## 项目阶段与阅读顺序

当前进入工程开发，已有 React Router SSR Web、Rust 课程读取/判分 API、邀请认证、跨设备个人设置、固定版本学习提交/续学与 SeaORM 迁移。先读 `docs/README.md`、`docs/08-development.md` 和 `docs/09-implementation-tracker.md`，再按任务读对应文档。已有账号复习排程、收藏、实际学习概览、目录 release 的原子切换/回滚/硬撤回、视觉素材/角色库发布校验、管理员固定 revision 与整批 staging 目录预览；三类管理员题目判分预览也已接入；录音登记/解码/发布/分段媒体、私有预览与前端共用播放器已接入，部分浏览器验收已完成，完整可访问性、真实 iPhone 与有法语声音设备的 TTS 回退仍待验证，不要把计划或设计稿当成已实现功能。

若存在 `AGENTS.local.md`，也读取它；它是本机补充资料，应被显式读取，不假定工具自动加载。用户当次指令和已确认偏好优先于这里的约定。

## 已确认方向

- 中文母语自学者，A1–A2，多用户。
- 等级 → 单元 → 课程，正文以文章/对话为主体，解释/词汇/语法为结构数据。
- Web only + server；开发本机直接运行，生产本机 Docker Compose、Cloudflare DNS 与路由器端口映射。
- Compose 网关采用 Traefik，仅对外映射 HTTP 30075；HTTPS 由用户处理，不加入自动证书或 HTTPS 端口。
- 用户要求工程收尾后在 Docker Compose 中实际启动应用，核对迁移、服务健康与 HTTP 30075 入口；不能仅交付 Compose 文件或构建镜像代替启动。不以此授权修改外部 DNS、路由器或 HTTPS。
- 2026-10-07 已要求新增全功能管理员后台，从个人页进入；范围及前六课用户批准记录见 docs/10-admin-development.md。明确用户批准可以记录 reviewed，不得冒充独立专家审校或据此伪造素材授权。后台仍需实现，不将课程预览视作已完成后台管理。
- 界面美观且有情境插图；前台不暴露数据 Schema、部署方式等实现术语。
- 操作入口不使用长箭头装饰；课程知识点采用分类色签的笔记卡片、圆形 SVG 展开标记与柔和展开动效，保留键盘操作、清晰焦点与 reduced-motion。
- 服务端优先 Rust；前端 Vite 8 + TypeScript 7 + Tailwind CSS 4 + React Router，不使用 Next.js。
- 持久化采用 SeaORM + PostgreSQL；Entity 与公共 DTO 分开，复杂查询可显式 SQL，事务和用户权限不能交给 ORM 自动推断。

## 实现原则

- 领域规则集中在服务端；课程解释器不执行作者代码，不接受任意 HTML/MDX/JS。
- Rust/TS 共享生成的公共契约；Rust Serde 类型是实现期真源，Schema/TS/OpenAPI 生成物不得手改。Web 不导入答案、数据库代码和 secrets。
- 填空输入限额由 Rust 公共契约生成 `answer-limits.ts`，HTML maxlength/JS 字符串长度采用 UTF-16 code units；不得各层重复硬编码。作者答案需有 NFC、空白和撇号归一化后符合限额的表示，大小写转换不参与表示长度检查；服务端仍验证实际输入，不能只依赖浏览器。
- 内容 revision 发布后不可变，学习会话固定版本；跨账号访问、缓存和并发写入需要明确验证。
- 步骤确认后推进依赖服务器成功回执，并按幂等键只消费一次；待确认请求恢复不能依赖无法持久化的页面回调，也不能把冲突后的读取当作成功确认自动推进。
- 身份失效后先同步卸载旧私有路由、LearningProvider 和离页保护，再刷新服务器授权的页面；不能只请求 reload，让旧草稿的 beforeunload 留住旧账号界面。网络失败不证明身份失效，仍保留原身份等待后续确认；失效不能伪装服务器写入回滚。
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
- `release-activate` / `content-withdraw` 的本地作者诊断与运行时共用事务及发布/撤回检查；可指出 generation 冲突、撤回课程及媒体字段，但不得输出 SQL/连接秘密，也不得为诊断跳过媒体校验或原子性。
- 课程先结构/语义校验，再预览与人工审校；结构通过不表示教学内容正确。
- 作者文件先运行 `cargo run -p brioche-server -- check <lesson.json>` 或 `check-release <manifest.json>`，不要求数据库。它们检查结构与本地引用/判分一致性，不替代素材登记、授权、人工审校或 release-stage 的数据库发布校验。
- 整个本地课包可用 `check-release <manifest.json> --sources <file-or-directory> ...`（1–20 个来源）。目录仅按清单引用查 `<lessonId>.lesson.json`，别名课源显式传文件；会检查课源完整结构/语义/私有规则与清单 ID/revision/等级/单元对应，缺失或歧义失败。默认不带 sources 仍只检查清单，不将离线成功当作导入/审校/发布证明。
- 图片可先运行 `asset-check <file> <image/svg+xml|image/png|image/jpeg|image/webp>`，不访问数据库，输出实际哈希/字节数/尺寸；复用正式导入的大小、格式与安全 SVG 校验，不表示素材已经登记或获得授权。
- 登记用素材包可先运行 `assets-check <bundle.json> <source-directory>`，录音包用 `audio-bundle-check` 同参数；只读、不连接 DB/写媒体，复用导入的元数据、目录边界、哈希、解码与尺寸/时长检查。ready/授权元数据不得伪造；角色登记引用、重复版本、课程时间轴与发布状态仍须正式导入/release 验证，检查后正式导入重新读取文件。
- `/author-preview` 仅供 operator 查看已导入固定 revision；私有媒体也必须经过当前管理员身份与撤回检查。不得把预览素材放到公开路由、返回私有答案或以预览创建真实学习进度。
- `/admin` 是实际后台入口，首批含审批、目录切换和版本撤回。网页审批追加不可变 editorial_reviews，当前用户为 actor、版本控制防止覆盖；未有网页决定时沿用作者源 editorial。stage 与 activate 都检查最新决定，并与撤回共用 content_state→revision 锁顺序。不要修改不可变 server_document 来实现审批，或把后台首批能力当作全部管理功能完成。
- 后台 JSON 课源导入与 CLI 复用 author_import，网页相同内容重试幂等，CLI 保留重复版本冲突；导入审计不可变。测试草稿必须显式设置 editorial，不依赖示例课源一直为 draft。2026-10-07 用户授权的首六课已正式发布，登记素材范围仅为该批项目原创 SVG；全量 planned 清单和其他 42 课仍待审，不把首批上线推广为所有内容与全部后台完成。
- 后台账号发放复用 identity.issue_token_impl，在 account-admin→邮箱锁内复核当前 operator 并与审计原子提交。后续角色/会话管理沿用账号管理锁，并保护最后管理员。明文一次性链接只在当前对话框内存显示，关闭清空，不进入SSR/日志/持久化；网页发放不自动发邮件。管理员写入必须带离页取消信号，迟到CSRF不能继续提交；取消不能冒充服务端事务回滚。
- 录音先 `audio-check`，再 `audio-import <bundle.json> <source-directory> <actor>`；`audioRefs` 固定登记版本，登记不公开文件。发布必须重新验证登记描述、实际文件哈希/解码、来源授权与正文时间轴；音频公开路由和私有预览均保留撤回检查、no-store 与有界读取。
- 权限、判分、事务、幂等、版本与恢复做风险对应的验证；文档/低风险样式不添加无意义测试。
- 异步确认按钮在等待期间保留键盘焦点，用 aria-disabled/aria-busy 配合同步提交锁；未满足答题/步骤条件或恢复初始化未完成时仍原生 disabled，不能仅依赖 ARIA 阻止重复写入。
- 实际命令：`pnpm dev:api`、`pnpm dev:web`、`pnpm contracts`、`pnpm typecheck`、`pnpm build`；Rust 使用 cargo fmt/check/clippy/test。PostgreSQL 集成测试显式设置专用 `TEST_DATABASE_URL` 后运行，禁止指向生产。TS 7 类型检查不能用 Vite build 代替。
- `pnpm test:browser` 使用锁定的 agent-browser 和独立 Chromium 会话，对真实组件进行键盘/迟到响应/多正文回归；首次安装浏览器用 `pnpm exec agent-browser install`，Linux CI 使用 `--with-deps`。装配只连接自己的临时端口、合成课程与受控请求，自动清理；不连接用户浏览器/账号，也不替代数据库、真实法语声音、iPhone 或辅助技术验收。
- 生产页面壳另用 `pnpm test:browser:ssr`；先运行 `pnpm build`，测试直接加载当前 server/client 构建、真实 Layout/客户端路由与 CSS。后端为受控公共课程 API，两台 HTTP 服务及 Chromium 均使用独立随机 loopback 端口/会话并自动关闭；不能将它当作实际 Rust/数据库/生产或 iPhone 验收。
- 共用浏览器会话的独立测试须隔离 sessionStorage 草稿；同一用例内的离页、返回和重试保留真实草稿。不能把上一用例恢复的步骤当作当前应用推进或焦点缺陷，定向通过后还需核对完整套件的顺序影响。
- 不自行修改用户 DNS/路由器或执行生产上线；按相应任务的授权范围工作。
- 运维备份使用 `scripts/backup.mjs`，同时保留数据库与全部登记媒体；真实备份/会话/私有判分不得入 Git。restore 只创建新数据库和新媒体卷，不自动切换应用或清理失败目标。`pnpm test:ops` 运行运维检查，显式 `BRIOCHE_BACKUP_DOCKER_TEST=1` 才创建隔离 Docker 演练资源；恢复样本通过不等于生产 RPO/RTO 或公网验收通过。
- 运行巡检使用 `pnpm health:check --project <明确的 Compose 项目名>`；只读服务状态与入口 HTTP，可选检查指定宿主盘，不读取/输出环境秘密。退出码 0/1/2 分别表示健康/检测故障/参数或脚本失败。脚本不安装定时任务、发送通知或自动修复；实际生产告警与外部探测仍须按用户环境配置和验收。
