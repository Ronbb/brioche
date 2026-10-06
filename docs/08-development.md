# 第一轮工程实现

2026-10-07 当前语音实现：课程播放器仅播放固定录音，已移除浏览器发音及失败回退。缺音频和不完整正文不播放半段，显示toast；媒体失败停止，用户明确点击才重试。暂无正式课程录音，因此首六课朗读暂不可用。预生成TTS覆盖、语音审听/对齐、固定版本登记与新目录发布继续待完成，以下历史记录中的浏览器回退不代表当前策略。

填空提示可选：hintZh 为空或只有空白时，账号练习与演示练习都不显示提示入口。提示 API 对空提示返回 404，不记录 hintUsed 或改变学习进度；有内容的提示保持原幂等记录。键盘主动展开提示后焦点移至提示内容，初始已显示的提示不抢焦点。对应独立浏览器与 PostgreSQL 回归已补齐。

作者文件中的块/练习先按显式类型分派到与公开 DTO 共用字段宏生成的严格结构，保留嵌套类型与未知字段 JSON Pointer。`check`/`import` 可定位对话语块、角色、选项/token 的实际字段行列；缺失字段定位最近存在父节点。公开课程/Schema/TS 形状保持不变，运行 API 的错误仍不暴露作者诊断。

练习步骤约束现由共享 Rust 语义校验执行：exercise 仅被 practice 引用，completion.requiredExerciseIds 必须属于至少一个必做 practice。`check` 与 `import` 数据库前预检报告原 steps/blockIds 或 requiredExerciseIds 项的行列；可选练习与其他 practice 中的再次回顾仍允许，完整登记媒体验证保留在 hydration 后。

选择题作者检查拒绝归一化后相同的选项文字：NFC、空白和法语撇号等价，大小写与重音保留。`check` 与 `import` 均在数据库访问前给出第二个重复选项的 `/blocks/.../options/.../text` 和原文件行列；正式导入仍完整验证登记素材与课程语义。排序题允许重复显示语块，共用同一个 Rust 文本归一化函数，避免作者校验与判分规则漂移。

更新日期：2026-10-06。这是工程基础与课程阅读链路，尚未完成多用户学习产品。

## 已实现

- 排序题仍要求所有原语块 ID 各使用一次，判分按其对应的逐位置文字序列进行；相同或 NFC/空白/撇号等价的重复语块可互换，避免内部 ID 造成相同文字的误判。大小写、重音与不同词的位置仍有意义，重复/缺失/未知 ID 不会被等价文字绕过。公开契约和历史尝试结果不自动改写。

- 填空输入共享 Rust 生成的 `@brioche/contracts/answer-limits`：1024 UTF-16 code units，与 HTML maxlength/JS length 一致，同时保留 4096 UTF-8 字节上限。作者 accepted 保留原字节上限，要求 NFC、空白/撇号归一化后有可输入表示；check/import 在数据库前定位不可输入答案。实际判分拒绝超过输入限额的文本，前端保持原始重音/组合字符提交，归一化仍集中在服务端。

- 账号学习进入“课程已撤回/学习记录暂不可用”后，继续保留独立收藏与复习原请求的导航确认。正常和不可用页面共享同一 blocker 生命周期，当前会话被确定拒绝不代表其他原请求已确认；独立请求全部确认后关闭旧离页提示并留在当前页面，不自动导航或重发。

- `/pending-saves` 按账号隔离保存生命周期，列表同步原请求存储的变化，收敛先前控件的迟到确认；保存按钮保持键盘焦点并以同步锁拒绝重复确认。503 后明确重试沿用原 path/body/key，当前存储已确认或已替换的旧行不会被再次提交。未确认原请求保留离页与刷新提醒，确认全部请求后解除；旧账号响应不会清空或锁住新账号列表。

- 个人表达写入的页面保护覆盖公开阅读、账号学习、账号复习和表达库。按当前账号的已校验原请求存储聚合收藏/加入复习/复习偏好/自评状态；卡片或解释面板收起后仍有离页确认与刷新提醒。单项确认仅清除自己的原请求，全部确认后取消已打开的旧跳转提示，保留当前页供检查；每个页面只注册一个导航拦截器。

- 主屏幕入口：`public/manifest.webmanifest`、Apple 主屏幕名称及 180px PNG、192/512px manifest 图标与 SVG/32px favicon；源文件与维护约定见 `apps/web/public/icons/README.md`。standalone 启动仍保持在线学习，不注册 service worker 或缓存私人数据。实际 iPhone 添加和启动效果待设备验收。

- pnpm / Cargo workspace、锁文件、Rust 1.99.0 工具链、TypeScript 7 独立类型检查。
- React Router Framework Mode + Vite SSR；正式客户端包含首页、课程阅读、固定版本账号学习、`/reviews` 账号复习与 `/profile`；单课 `/practice/:lessonId`、`/review/:lessonId` 仅在 developmentFixture 模式提供演示。
- 迁移确认过的视觉与主要阅读交互：角色介绍、头像整句、点词朗读和词汇解释、短文、全文播放/暂停/长按调速、复习卡片、统一设置、toast、动效和覆盖式滚动条。
- 录音失败转浏览器语音时保持用户暂停意图：暂停期间迟到的媒体错误不会开始语音等待或朗读，明确恢复才从当前句以最新倍速尝试；无语音仍保留实际失败提示。等待中的切词/离页取消旧监听与回调。
- 当前系统语音语句返回终止错误（含 canceled/interrupted）时停止播放器并用 toast 提示明确重试，不继续显示播放或推进下一句；应用停止/切词/调速导致的旧语句错误仍按播放代次和语句身份忽略。
- Axum 公共目录和课程 API、health/readiness、SIGINT/SIGTERM 优雅退出。
- Rust Serde 公共 DTO，生成 TS 联合类型和公共课程 JSON Schema；Web 只导入公共契约。私有答案与编辑状态不进入课程响应。
- 课程块拒绝未知字段：10 类块及三种练习均严格解析，现有平铺练习格式不变。`check`/`import` 在连接前拒绝额外字段，诊断定位原文件对应字段的 JSON Pointer 与值的行列，并显示未知字段名；生成的公共/作者 Schema 同步约束额外属性。
- SeaORM PostgreSQL Entity、版本化显式迁移、仅插入的课程导入工具、草稿过滤及最新发布 revision 读取。
- `/practice/:lessonId` 示例练习：选择、填空、排序由 Rust 判分，错误反馈/重试/回顾由页面呈现。私有答案规则在导入时校验，公共契约只包含提交和反馈 DTO。
- 正文增加结构化解释、文化范围、词汇与语法；点语法锚点可打开解释。校验拒绝坏解释锚点、重复题目选项、未知步骤类型和不可达教学块。
- Docker Compose：PostgreSQL → 一次性迁移 → API → SSR Web → Traefik。入口仅 HTTP 30075，HTTPS 由用户外部处理；数据库不映射宿主端口，生产关闭示例课程模式。
- 账号模块：邀请注册、登录、退出、密码恢复，Argon2id 密码哈希、PostgreSQL 会话与一次性 token、精确 Origin + CSRF、持久化登录限流及过期清理。Web 增加 `/login`、`/invite`、`/reset-password`，SSR 从 Cookie 读取实际身份，私有响应禁止缓存。
- 跨标签身份同步：成功登录、退出、接受邀请或重置密码后，只发送无身份数据的 storage 通知；其他标签立即停止播放、清理原账号学习草稿并重新加载服务端授权页面。页面重新可见/聚焦及可见时每 30 秒检查 `/api/v1/me` 的账号 ID 与权限角色，覆盖会话过期、外部撤销和降权；传输失败不冒充退出，后续继续检查，所有私有操作仍由服务端授权。pagehide 停止播放，BFCache 恢复重新加载；fixture 模式不请求身份接口。
- 个人资料与设置：昵称、IANA 时区、每周 3/5/7 天与每天 5/10/15 分钟目标、中文译文和 0.75/1/1.25/1.5 倍速。登录用户跨设备保存，访客保留浏览内设置；个人页使用可搜索的自定义时区面板。版本锁拒绝旧设备覆盖，失败保留编辑草稿，读取最新状态后明确重试。
- 设置失败后的资料重读与提交完成状态受当前保存代次约束；账号或 loader 资料版本更新后，旧重读不覆盖当前资料、不显示旧错误，也不阻塞新 scope 的明确保存。普通冲突仍读取当前资料并保留错误供用户确认，不自动重发写入。
- 保存失败后重读个人资料若明确返回 401，清除当前本地身份、停止播放并移除旧账号编辑器，提示重新登录；其旧路由拦截不会阻挡登录入口。其他重读失败保持已有资料与草稿，不把网络失败当作退出。
- 资料编辑器按账号隔离：身份改变即丢弃旧昵称/目标草稿、关闭旧弹窗并重置编辑锁；旧账号迟到的保存结果不能关闭或锁住当前账号的编辑器。同账号版本更新继续保留当前编辑草稿供明确确认。
- 资料弹窗有未保存修改时，关闭按钮或 Escape 先显示放弃确认；继续编辑保留草稿并返回昵称输入，明确放弃才关闭。应用内跳转和历史返回共用此确认，继续编辑取消跳转，放弃后进入原目标。已发出的保存暂挂时先保留页面并提示等待，成功后再继续跳转。仅编辑弹窗打开且有草稿或保存仍在进行时注册 beforeunload，刷新提示由浏览器处理；关闭/成功保存后移除。原生刷新提示、真实 iPhone 历史手势和失败保存后的更多导航组合仍待验收。
- 账号学习：`/learning/:sessionId` 按结构数据遍历所有步骤和正文块；服务端固定课程 revision，保存步骤、提示、首次及重试答案，确认完成后生成去重复习卡片。首页显示真实续学入口和课程完成记录。账号 `/reviews` 提供每批最多 10 项到期队列、自评与保存回顾；收藏与复习管理见下文。
- 学习与账号复习的未确认写入增加应用内离页确认，继续离开保留此标签页中的原提交，返回后需明确确认结果；留在当前页或 Escape 取消跳转并返回页面标题焦点。若写入在确认弹窗打开时已得到确定结果，关闭提示并取消旧跳转，保留当前页面供检查，不自动离开或重发。路由确认与 beforeunload 只检查实际未确认写入，队列/进度读取不会冒充未保存操作。真实设备历史手势、原生刷新提示与更多故障组合仍待验收。
- 复习离页提示关闭后按当前状态恢复焦点：队列待恢复、原提交待确认、错误反馈、普通复习标题依次优先。撤回响应与离页提示重叠时，仍先隐藏旧表达/来源/自评并停止播放，聚焦队列恢复提示；重读不能让已标记不可用的卡片重新出现。
- 学习概览：`GET /api/v1/me/dashboard` 在一致性快照中读取个人目标、周学习事实、到期数量/下一次复习与实际续学。按个人 IANA 时区的周一至周日统计步骤确认、练习提交、复习自评和首次完成，打开页面/创建会话与幂等重试不产生额外活动。每天分钟数是设定的目标，没有假装测量已学时长；撤回课程保留历史完成总数，但屏蔽正文、续学和复习入口。续学读取全部最新课程状态，不受 20 项概览分页截断。首页按真实等级/单元组织目录并使用固定版本续学内容；推荐按未学过优先、active release 的明确教学顺序排列。

## 本机开发

首页仅在课程的 reviewItemIds 对应有效词汇时显示可展开表达卡片；没有表达时仍保留账号复习与续学入口。卡片切换取消上一段高度动画，并从当前视觉高度开始；离页清理动画和待执行帧，减少动态效果设置下直接展开。课程搜索保留原输入节点和键盘焦点，等待中重复提交被拦截；结果返回时保留等待期间新输入的草稿，查看全部课程则同步清空输入。

作者课程 `check <lesson.json>` 与目录 `check-release <manifest.json>` 保留原 JSON 的位置索引，类型错误与带 JSON pointer 的语义错误输出 `文件:行:列: /字段/路径: 原因`。行/列从 1 开始，列按原文件 Unicode 字符计数，CRLF、转义键与数组索引均保留；定位在投影之前建立，不使用重序列化课程的行号。没有对应字段时定位最近存在的父值；私有规则支持缺失/多余规则、kind、反馈、选项引用、accepted 项与排序 token 项的路径，release 支持各层 ID/名称/版本/重复引用与数量上限路径。正文流程校验定位角色快照字段、句子/段落/语块 ID、空句子的 segments、解释目标中出错的 blockId/entryId/segmentId、题目字段和步骤字段；重复数组引用定位第二次出现的项，不可达教学块定位其 id。题目字段位于 block 根部，路径不加入不存在的 exercise 层。严格 JSON/重复字段/2 MiB 限制保持；离线课程位置索引最多 100000 个值。课程导入和目录 staging 已接入原文件索引；素材/录音导入包及媒体发布内部诊断仍需细化，不表示作者工具全部完成。

课程 `import` 在连接数据库前还检查 `serverOnly/grading` 的字段类型，使用同一作者解析器按已知 kind 检查三类答案结构，类型错误保留具体字段/数组项路径。连接前另检查公开 DTO 的字段类型；存在固定 assetRefs/audioRefs 时，忽略随后将由登记描述覆盖的 media/audio 占位值，其他字段继续检查。它仍需连接数据库登记课程；练习选项/token 引用、登记媒体及最终 hydrated 课程的语义在原导入阶段验证。运行时判分只返回不透明内容错误，不向客户端提供作者诊断或私有答案。

图片文件可先运行数据库无关的检查：

```sh
cargo run -p brioche-server -- asset-check docs/content/a1/assets/first-conversations.svg image/svg+xml
```

`asset-check <file> <MIME>` 支持 image/svg+xml、image/png、image/jpeg、image/webp，输出 JSON（sha256、byteLength、mimeType、width、height）。复用正式素材导入的格式解码与安全 SVG 白名单、32 MiB 文件和尺寸/内存限制，拒绝伪装 MIME；文件检查在 blocking worker 完成。它不登记文件或证明授权。首版新场景源文件与 planned 清单见 [课程素材记录](content/a1/README.md)。

`assets-import` 和 `audio-import` 保留原 JSON 文件位置索引，在连接数据库前检查包类型与导入元数据：ID/revision、状态/授权/署名、MIME、哈希格式、相对路径、尺寸或时长；视觉包另检查角色快照。错误显示原文件行/列和字段 JSON pointer。正式导入复用同一元数据检查；实际文件不存在、越界、格式不符、哈希或尺寸/时长不匹配也定位该清单项，角色缺头像 revision 或非正方形头像定位角色引用。已登记的图片/录音版本定位 `/assets/<index>/revision`，已登记角色定位 `/characters/<index>/snapshot/revision`；整批版本检查在同一登记事务和现有内容锁内完成，官方导入并发时仍保留准确错误位置，后项重复不会登记前项或新增审计。文件按哈希写入发生在事务之前，失败可能留下未登记对象，媒体接口不会公开它们。存储/数据库基础设施错误可能定位根容器，不能把根错误理解为事务一定未提交，应核对登记状态后重试。

录音文件可先运行数据库无关的检查：

```sh
cargo run -p brioche-server -- audio-check recording.mp3 audio/mpeg
cargo run -p brioche-server -- audio-check recording.wav audio/wav
```

输出为 JSON（SHA-256、字节数、实际解码时长、采样率、声道数）；日志写 stderr。使用 [Symphonia 0.6.1](https://docs.rs/symphonia/0.6.1/symphonia/) 完整解码，拒绝解码错误，按解码帧计算时长并向上取整到毫秒。限制 32 MiB、30 分钟、8–96 kHz、单/双声道；MP3 要求标准 Layer III 帧，拒绝自由码率、截断帧及未知尾随数据，ID3 标签不超过 64 KiB。WAV 支持 RIFF PCM 8/16/24/32 位及 IEEE float 32/64 位，检查完整 chunk、样本对齐和头部一致性；当前不支持 RF64、WAVE extensible 或压缩 WAV。解码在 blocking worker 执行，不阻塞异步执行器。`audio-check` 本身不登记文件，也不证明授权。

录音登记与课程引用：

整包登记前可离线核对清单与源文件：

```sh
cargo run -p brioche-server -- assets-check asset-bundle.json ./visuals
cargo run -p brioche-server -- audio-bundle-check audio-bundle.json ./recordings
```

两条命令不连接数据库、不写 `MEDIA_ROOT` 或修改源文件。严格读取至多 2 MiB 清单，沿用正式导入的元数据要求（包括 ready 与实际授权确认），并逐项复用正式导入的有界文件读取、目录边界、哈希、图片解码/尺寸或完整音频解码/时长检查；每次只保留一份至多 32 MiB 的原文件，检查在 blocking worker 执行。错误定位清单原字段行列；源目录错误定位根容器。同包包含角色引用的确切头像 ID/revision 时，元数据预检即要求方形，并定位 `/characters/<index>/snapshot/avatarId`；引用包外已登记版本仍合法，由导入查询数据库核对存在性与实际形状，不能用同 ID 的其他 revision 代替。成功只表示本地清单和文件一致，包外头像、已登记版本冲突及课程时间轴和发布审校仍由正式导入/release 验证。planned 草稿保持原状态，可先用单文件 `asset-check`/`audio-check` 制作描述，不为通过整包检查伪造授权。文件检查后仍可能变化，正式导入会重新检查，离线通过不能代替登记或发布。

```sh
cargo run -p brioche-server -- migrate
cargo run -p brioche-server -- audio-import audio-bundle.json ./recordings operator-name
cargo run -p brioche-server -- import lesson.json
```

`audio-bundle.json` 使用 `schemaVersion: "1.0"` 和非空 `assets` 数组。每项填写 assetId、revision、sha256、mimeType、durationMs、creditZh、相对 file、status、source、license、creator 和 rightsConfirmed；哈希/时长来自 `audio-check`，status 必须 ready，只有确认实际授权后才能设置 rightsConfirmed=true。最多 500 项，路径必须在源目录内。登记记录和审计不可修改，重复 ID/revision 拒绝并回滚整批数据库写入；升级录音使用新 revision。文件按哈希写入 `MEDIA_ROOT`，事务失败可能留下未被登记引用的对象，媒体路由不会公开它们。

课程源文件的 `audioRefs: [{"assetId":"audio-bakery","revision":1}]` 固定录音版本，导入工具覆盖 `audio` 为登记描述，`audioTracks` 仍由作者提供并校验。来源/授权/原文件路径只保存在私有登记信息中。release-stage 和 release-activate 重新比较登记描述、磁盘哈希、大小和解码参数，课程时间轴不可超出真实时长。素材登记与 staging 不公开文件；`GET/HEAD /api/audio/<sha>.mp3|wav` 只提供已发布未撤回课程引用的录音。支持单段 bytes Range、206/416、ETag/If-Range、Accept-Ranges 和 no-store；多个区间拒绝，文件完整哈希验证通过后才返回字节，读文件使用两个 blocking worker 许可。管理员固定版本预览将 URL 改为课程范围内的私有音频路由，逐次检查 operator、撤回与课程引用，返回 private/no-store。

课程浏览页、账号学习和管理员正文共用 recording-playback/播放器：优先录音，点词匹配标注，全文连续读取，媒体时钟驱动进度和可选单词高亮；暂停/调速不重放录音整句，切换/卸载/身份变化清理旧播放。加载中允许暂停；旧 metadata/error/play Promise/帧回调不覆盖新请求。有法语设备声音时才尝试失败回退，无声音则明确错误；Chrome 无声音环境已验证错误与恢复，真实 iPhone 和有声音设备继续验收。测试用浏览器例程仅接受命名的 loopback 数据库 `/brioche_browser_qa`；可通过 `BROWSER_QA_RECORDING` 指定本机合成 WAV（至少覆盖正文时间轴），`MEDIA_ROOT` 指定隔离存储，生成未审校的录音协议课程；禁止作为正式课程发布依据。

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

作者可先运行无需数据库的检查命令：

```sh
cargo run -p brioche-server -- check docs/examples/a1-bakery.lesson.json
cargo run -p brioche-server -- check-release docs/examples/catalog.release.json
cargo run -p brioche-server -- check-release docs/content/a2/catalog.full.release.json --sources docs/content/a1 docs/content/a2 docs/examples/a1-bakery.lesson.json
```

`check` 使用实际公共 DTO、课程引用/步骤校验及服务端私有判分校验，另检查素材引用 ID、revision 和重复引用；`check-release` 默认仅校验目录清单结构及 ID、revision、重复引用。可选 `--sources` 接受 1–20 个课源文件或目录，逐课复用 check 并核对清单中的 ID/revision/等级/单元。目录只查清单引用的 `<lessonId>.lesson.json`，不递归扫描；别名文件需明确传入，例如例课 `a1-bakery.lesson.json`。同一实际文件去重，不同文件提供同一课源时报歧义；显式提供但不在清单中的课源拒绝。目录候选须为解析后仍位于该目录内的文件。

两者不连接数据库、不写入内容，也不验证数据库登记或授权、实际素材文件或人工审校；带 sources 只证明本地课源关系，不证明相同 revision 已导入数据库。全部检查成功后才报告课数，成功明确提示后续仍需媒体登记、审校和 release-stage。公共投影类型、课程语义、私有判分与目录语义错误显示原文件行列和字段路径；私有规则的语义诊断描述原因，不回显接受答案或正确选项值。HTTP 判分/发布调用仍转换为原有通用错误，不返回作者诊断。

`editorial` 是必需的严格作者信息：status 仅接受 draft/reviewed，note 必需且非空、最多 8000 UTF-8 字节，拒绝未知字段及控制字符（允许换行/制表符）。课程检查、导入、release-stage 共用该校验，发布仅接受 reviewed；该状态是作者声明，不构成人工审校真实性的自动证明。

`pnpm contracts` 同时生成公共契约和 `docs/generated/author-lesson.schema.json`。作者 Schema 引用实际 Rust 公开 DTO、私有规则、审校类型和素材引用类型，单独保存在 docs，禁止导入 Web 契约包。它描述结构类型，语义限制（例如正 revision、引用关联、判分一致性、note 文本边界、发布授权）仍由 Rust 校验；check 使用 Serde 与语义校验，不运行另一套 JSON Schema 引擎。CI 校验生成文件无漂移；原 `docs/examples/lesson.schema.json` 保留为设计快照。

导入时，在本机 PostgreSQL 中建立专用数据库并设置 `DATABASE_URL`。使用 CLI 执行迁移，服务启动不会自动同步表结构。

```sh
cargo run -p brioche-server -- migrate
cargo run -p brioche-server -- import docs/examples/a1-bakery.lesson.json
```

导入始终创建不可见 revision；旧的 `--publish` 参数被明确拒绝，改用目录 release 原子发布。本示例未审校，禁止为测试上线而直接改状态。相同 `(lesson_id, revision)` 重复导入失败，数据库触发器也拒绝改写或删除已有正文/私有答案；审校后重新导入需要新 revision。

`check`、`check-release`、`import`、`assets-import` 与 `release-stage` 共用严格 JSON 文件读取：实际读取最多 2 MiB + 1 字节，超过 2 MiB 拒绝；所有层级的重复字段、尾随第二个文档、无效 UTF-8 和过深嵌套均拒绝。错误链包含输入文件名及 JSON 行列；重复字段显示 JSON Pointer（例如 `/steps/0/id`），素材/发布清单类型错误显示字段路径。类型定位使用 [serde_path_to_error](https://docs.rs/serde_path_to_error/0.1.20/serde_path_to_error/)。`check`/`check-release` 的公开投影、私有判分与目录语义校验已接入原文件位置索引。

`import` 在连接数据库前读取原课程并检查 editorial 和素材/录音引用类型，随后定位登记 revision 缺失、投影/私有判分错误、超出数据库范围的 revision 和重复 revision。拒绝多余参数，旧 `--publish` 在连接前明确拒绝。`release-stage` 在连接前校验原清单结构/语义，事务内通过同一个 stage 实现检查固定版本、撤回、reviewed 状态、目录对应关系、判分与媒体；诊断指向清单中的 revision、课程项或 release id。媒体发布错误目前定位课程项并提示检查登记/授权/文件，没有伪造数据库中课程源文件的行号；数据库异常定位根并提示核对实际状态后重试，不把响应失败等同于提交必然失败。运行时 stage 仍返回原有不透明 AppError。素材/录音导入包及每个媒体发布失败的内部细分位置仍待完善。

### 管理员固定版本预览

使用 `invite <email> <private-output-file> --operator` 建立内容管理员，邀请链接仍只写私有文件。登录后个人页显示“课程预览”，进入 `/author-preview` 输入已导入的课程 ID 和 revision。页面按课程步骤展示全部正文/教学内容与交互练习；点词、头像和正文朗读复用现有组件，不创建学习会话。只有管理员可访问，开发 fixture 模式没有管理员身份入口。

`GET /api/v1/operator/lessons/{id}/revisions/{revision}` 返回公开 DTO，允许已导入但未发布的 draft；不返回 editorial 或私有答案。图片 URL 改为该版本下的私有 media 路径，每次读取都验证当前 operator 身份与未撤回状态，只能读取该课程引用的素材。私有与公开媒体共用限量文件读取、路径 containment、SHA-256 校验及安全响应头；预览媒体独立限并发为 2。所有预览响应 private/no-store，退出、降权和撤回均使后续请求失去权限。

同页可以输入已 stage 的发布批次 ID，按清单的等级、单元、课程顺序预览整个目录；点击课程使用清单指定的 revision，保留批次上下文。`GET /api/v1/operator/releases/{id}` 在 repeatable-read 快照中读取清单、不可变 entries 与撤回信息，仅返回公共目录摘要和撤回 ID。未激活的批次也可预览，不切换 active release 或 generation；已撤回课程保留摘要并标注，正文/媒体仍拒绝访问。页面所选课程必须属于所选批次的固定版本。

管理员题目预览复用单选、填空、排序编辑器；提示与答案只保留在页面内存，不保存标签页草稿。`POST /api/v1/operator/lessons/{id}/revisions/{revision}/grade` 接受现有 GradeRequest，要求当前 operator、有效 Origin/CSRF、请求 revision 与路径一致以及课程未撤回。它读取固定版本私有规则，使用正式 Grader，仅返回 correct/feedbackZh/exerciseId，不返回答案键，不写 learning_sessions、exercise_attempts、完成状态或复习数据；无需学习提交幂等键。失败保留页面答案并用 toast 提示，可以重试；离页后丢弃预览结果。

固定 revision、整批 staging 目录和三类题目判分预览已接入；浏览器完整体验验收和自动审校仍未完成。线上发布仍通过 release-stage/release-activate。

### 目录 release 命令

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

准备好部署参数、核对项目与端口占用后，启动方式为 `docker compose up -d --wait --wait-timeout 180`，再核对 `docker compose ps --all` 和 `pnpm health:check --project brioche`。用户要求工程收尾后实际启动并保留应用在 Docker 中运行；具体验收见 [部署说明](05-deployment.md#工程收尾后的实际启动)。未发布正式内容时支持空目录与管理员私有预览，不能为启动伪造人工审校。访问 `http://<宿主机地址>:30075`；对外 HTTPS、域名和路由器由用户处理。Compose 内不申请证书，也不开放 HTTPS 端口。

Web 镜像用 `pnpm deploy --prod` 保留生产依赖，使用 React Router Node 服务，API 为 Linux release 二进制。入口使用官方 `traefik:v3.7.13` 镜像，固定发布 `30075:8080`。配置位于 `infra/traefik`，file provider 保留 API 路径前缀、页面走 SSR；没有 Docker socket、公开 dashboard 或证书卷。版本依据：[Traefik 3.7.13](https://github.com/traefik/traefik/releases/tag/v3.7.13)、[PostgreSQL 18.6](https://www.postgresql.org/docs/release/18.6/)。

数据库和媒体持久卷、备份/新目标恢复已接入并实际演练。Rust/Node/Debian/PostgreSQL 与既有 Traefik 的基础镜像固定 index digest，版本与更新方式见 [镜像说明](../infra/images.md)。完整 Compose 已在独立 brioche-compose-qa 项目验证：生产空目录、一次性迁移退出 0、四个长期服务 healthy、仅 HTTP 30075 发布、非 root Web/API 和可写媒体卷；同入口的邀请/注册/登录/退出、精确 Origin 拒绝、SSR/静态资源，以及恢复后的学习/收藏/复习/音频/幂等重放均通过。恢复的测试源只用于协议验收，不作为已审校正式课程。生产参数、容量、用户外部入口与公网验收继续待完成；TLS/DNS/路由器设置尚未修改。

## 账号与课程入口

退出操作使用独立同步锁和当前资料身份的请求生命周期：等待时按钮保留焦点，失败以 toast 提示并保留草稿；确认成功后仅清理对应账号的标签页草稿、停止播放并返回首页。离开个人页或身份替换会取消客户端等待，旧成功/失败不再刷新新页面或向新资料显示提示。已经发出的 POST 仍可能在服务端完成，取消不表示回滚。

账号表单等待时保留焦点，输入只读、提交以同步锁去重；失败保留输入，仅在焦点仍位于表单内时聚焦错误。密码恢复确认后清空密码/token 并聚焦成功标题。离页或读取新的邀请/恢复 fragment 会取消当前客户端等待；新链接先清空旧密码、昵称和反馈，旧响应不覆盖新链接或导航。CSRF 返回后重新检查取消状态，避免离页后继续发起账号 POST。取消已经发送的请求不表示服务端回滚，身份仍以服务器会话为准。

认证采用 `axum-login 0.18.0` 配套的 `tower-sessions 0.14.0`，避免与 0.15 创建两套 Session 类型（[官方依赖清单](https://docs.rs/crate/axum-login/0.18.0/source/Cargo.toml)）。只存会话 ID 的 SHA-256，记录用 `timestamptz` 到期；create 不覆盖冲突、save 不插入，撤销后的旧响应不能恢复记录。CSRF 用系统随机数、常量时间比较和配置的 origin allowlist，不从代理 header 推断可信 origin。密码哈希通过有限并发的 blocking worker 执行；登录轮换会话，密码恢复撤销所有旧会话，定时清理过期记录。

账号通过管理员 CLI 发出一次性邀请；恢复也由管理员确认邮箱后生成链接，没有尚未配置的邮件发送入口。设置 `DATABASE_URL`、`PUBLIC_APP_URL` 并执行迁移后运行：

```sh
cargo run -p brioche-server -- invite learner@example.com .local/invite-link.txt
cargo run -p brioche-server -- reset-password learner@example.com .local/reset-link.txt
```

输出文件必须尚不存在；链接只写入该私有文件，不输出到日志。邀请有效 48 小时，恢复有效 30 分钟；重新签发撤销同类旧链接。链接 token 放在 URL fragment 中，页面读取后移除，数据库只保留 SHA-256。生产容器可用 `docker compose exec server brioche-server invite learner@example.com /tmp/invite-link.txt`，管理员私下读取和交付，再删除该文件。Unix 创建权限为 0600；Windows 输出位置应使用管理员私有目录。`--operator` 仅用于邀请授予管理员角色；账号管理仍通过 CLI；管理员固定版本和批次课程预览已提供，见下文。不要提交、截图或公开链接文件。

登录用户通过 `POST /api/v1/learning-sessions` 开始或恢复当前课程，通过 `GET /api/v1/learning-sessions/:id` 读取固定版本与进度。`PUT .../steps/:stepId` 确认步骤、`POST .../attempts` 提交答案、`POST .../hints/:exerciseId` 记录提示、`POST .../complete` 完成本课；写入携带版本和幂等键，答案只由 Rust 判分。`GET /api/v1/me/learning` 提供每课最近记录与游标分页。所有读写验证会话所有者，写入同时验证 Origin/CSRF。完成要求必需步骤确认与必需题目尝试，不要求全部答对；首次完成时间保持不变。

步骤、判分记录、幂等结果和完成时复习卡片在事务中保存。相同键/载荷返回原结果，不同载荷拒绝；旧版本返回 409；固定快照撤回后返回 410，包括原幂等结果。浏览器遇到未确认的提交保留原请求和答案，明确重试原键，不自动生成第二次尝试。未提交答案与未确认原请求按账号、会话和固定 revision 保留在当前标签页；刷新或 SPA 离页后可以恢复原草稿与原幂等请求。已保存进度可跨设备读取，标签页本地草稿不跨设备同步。目录发布与硬撤回由上述 CLI 管理。

独立 `/practice/:lessonId` 和 `/review/:lessonId` 仅在 developmentFixture 模式提供演示，不保存账号进度。正式模式旧练习入口重定向到对应课程，由用户明确开始或继续账号学习；旧单课复习入口重定向到 `/reviews`，未登录时继续进入 `/login?next=/reviews`。首页正式复习入口也按身份进入账号队列或登录，不跳演示页。账号学习完成生成复习卡片，账号 `/reviews` 自评已持久化；其他验收缺口仍按 [实现清单](09-implementation-tracker.md) 继续实施。

SSR 入口回归运行 `pnpm build` 后再运行 `pnpm test:ssr`（CI 已接入相同顺序）。测试加载实际构建的 React Router 服务端，用隔离本地 HTTP 适配器控制目录模式和身份；覆盖正式入口重定向、匿名首页、开发演示与已登录私有队列读取。它不替代真实数据库、浏览器交互或设备验收。

公开课程的 explore 步骤可引用练习块，其入口与页尾共用身份/模式判断：正式账号通过明确按钮开始或继续学习，匿名用户登录后返回该课，开发示例进入演示练习。通用 TeachingBlock 不再处理 exercise；账号学习与管理员预览分别使用真实 ExerciseEditor 和预览判分，避免通用渲染器生成指向旧演示路由的循环入口。

公开阅读按正文块 ID 切换，支持同课多个 dialogue/article；同类型有多段时用 block.titleZh 区分，单段仍显示对话/短文。每个正文的译文展开状态独立，切换停止旧播放并关闭知识解释。标题列表在窄屏内部横向滚动，不增加系统滚动条占位；箭头/Home/End 切换并将选中标题滚入可见区域。账号学习继续按步骤遍历全部正文和练习。

关键组件的 Chromium 自动化回归运行 `pnpm test:browser`；首次需要 `pnpm exec agent-browser install`，Linux CI 先执行 `pnpm exec agent-browser install --with-deps`。agent-browser 固定为 0.27.0 并随 pnpm lockfile 保存，安装脚本只对该依赖显式允许。测试文件在 `apps/web/browser-tests`，合成课程按公共 DTO 类型检查，真实 StartLearning/Lesson/LearningProvider 和 React Router 用独立 Vite 临时端口装配。每次生成独立浏览器会话，结束关闭会话与临时服务；没有数据库、账号、私有答案或生产 API。覆盖键盘焦点/重复提交/登录回跳、课程替换后的迟到响应与同课重试幂等、多正文选择/译文/播放停止和 320/390/900px 标题容器范围。声音列表与 SpeechSynthesis 是受控适配器，不能将其计为真实法语声音或 iPhone/屏幕阅读器验收。

pnpm 11 使用 `pnpm-workspace.yaml` 的 `allowBuilds`，旧 `onlyBuiltDependencies` 已移除；当前显式允许锁定的 `agent-browser@0.27.0`、Tailwind oxide 与 esbuild，不关闭严格安装脚本检查。参考 [pnpm 11 迁移说明](https://pnpm.io/blog/releases/11.0)。浏览器用例结束会核对并清空未捕获页面异常，避免只检查 UI 状态而漏掉脚本错误；语音回归另覆盖旧 utterance 的迟到 start/error/end 不影响新播放、当前 interrupted 后停止并可重试。在精简 Linux 验收容器使用安装器时须具备 sudo；浏览器系统依赖仅用于测试环境，不加入生产运行镜像。

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

`GET /api/v1/me/review-history` 每页 20 条，显示原自评及提交时区的时间；数据库保留旧/新档位、排程和算法版本。撤回来源时隐藏词汇正文，仍保留历史事实。旧记录页提供返回最新记录的入口；空旧页与尚无复习记录分别提示，分页后焦点回到标题。长表达按可用宽度换行，保留下次复习日期。三个列表游标均以时间与随机 ID 排序，校验格式，绑定当前登录账号。读写受既有认证/Origin/CSRF/private-no-store 保护，写入与原幂等结果同事务保存。未确认写入保留原请求，可从个人页的待确认记录恢复并明确重试；更多故障组合与真实设备仍须验收。

## 视觉素材与角色库

迁移 9 注册不可变素材 revision、角色快照及导入审计。`MEDIA_ROOT` 默认 `.local/media`；服务端和内容 CLI 必须使用同一个目录。Compose 的 server 挂载 `media_data` 到 `/var/lib/brioche/media`，镜像创建 UID 10001 可写的目录；备份和恢复同时保留 PostgreSQL 与这个卷。隔离样本的实际恢复已验证，生产规模和异盘副本仍待验收，操作见下文。

## Docker 备份与恢复

需要 Node 24 与 Docker。`scripts/backup.mjs` 使用现有 PostgreSQL 容器内的工具，不要求宿主安装 psql/pg_dump，不通过 PowerShell 文本管道传输二进制，也不把数据库密码放到命令行。用 `docker compose ps -q postgres` 获取实际数据库容器 ID，用 `docker volume ls --filter label=com.docker.compose.project=brioche --filter label=com.docker.compose.volume=media_data --format '{{.Name}}'` 核对媒体卷；自定义 Compose 项目名时修改 project filter。下面的容器/卷名称是默认项目示例，应替换为实际核对值。

```sh
node scripts/backup.mjs backup --database-container brioche-postgres-1 --media-volume brioche_media_data --output backups/2026-10-06
node scripts/backup.mjs verify --input backups/2026-10-06
node scripts/backup.mjs restore --database-container brioche-postgres-1 --database brioche_restore_20261006 --media-volume brioche_restore_media_20261006 --input backups/2026-10-06
```

backup 默认数据库和角色为 brioche，可用 `--database`/`--user` 指定。输出目录必须不存在，父目录需要先创建；不会覆盖旧备份。数据库使用 pg_dump 的一致性快照；随后读取不可变视觉/录音登记，复制全部登记对象并核对 SHA-256，不只复制当前已公开课程。登记和对象只增不改，因此随后加入的额外对象不影响之前快照的可恢复性；备份期间禁止迁移、手工改写/删除登记文件或 prune。临时读取容器复用当前数据库的实际 image ID、无网络、媒体只读；退出后删除自己的临时容器。备份不会包含未登记临时文件、环境秘密、Docker 镜像或全局 PostgreSQL 角色，秘密和应用镜像版本应另行保存。

目录包含 `database.dump`、`media/<sha>.<extension>` 与最后写入的 `manifest.json`。没有 manifest 或校验失败的目录不能视为完整备份。文件/目录采用 600/700 权限，Windows 仍需使用受限 ACL 的存放目录；脚本没有加密功能，真实备份包含账号、会话、私有判分和学习数据，必须保持私有并复制到异盘或加密存储。限制 dump 10 GiB、单媒体 32 MiB、最多 100000 个媒体对象；达到上限需调整运维方案，不应静默漏备份。

restore 首先完整校验 manifest、文件大小/哈希和 pg_restore 的 archive 目录，并要求相同 PostgreSQL major。只允许显式的新数据库和新媒体卷，拒绝存在的目标；volume 的操作标签还防止并发创建后误写其他卷。数据库 pg_restore 使用 single-transaction/exit-on-error/no-owner/no-acl，目标归当前数据库角色所有；媒体写给应用 UID 10001，并重新核对容器中哈希。实际传输也重新计算源文件哈希，拒绝检查后被改写的备份。不会替换 DATABASE_URL、媒体卷绑定或 active release，不会启动应用；失败时保留目录/新目标供检查，不自动清除或覆盖。网络/提交失败仍应检查实际状态后再选择新的目标重试。

恢复后先在隔离 origin 启动应用，校对迁移版本、release 指针/内容版本、媒体、旧会话和新登录、进度/判分/收藏/复习，再安排维护窗口切换数据库与媒体卷。不要在恢复副本上运行任意反向迁移。备份中的历史 cookie/token 状态也会被恢复，生产故障恢复时应决定是否强制退出旧会话和重发恢复链接。

`pnpm test:ops` 运行两项无 Docker 验证测试；显式设置 `BRIOCHE_BACKUP_DOCKER_TEST=1` 后运行同命令，会创建自己的无网络 PostgreSQL 容器和媒体卷，验证大于 pipe buffer 的真实 custom archive、100000 行数据恢复、媒体校验及目标/坏备份拒绝，并清理自建资源。当前已实际执行该集成测试。另已用真实应用迁移和账号数据做恢复演练，证据见 `07-design-verification.md`；它证明样本可恢复，不能替代生产容量、RPO/RTO、保留策略和异盘存储验收。

`cargo run -p brioche-server -- assets-import <bundle.json> <source-directory> <actor>` 读取严格字段的清单，登记素材与角色，文件按 SHA-256 命名。参考 `examples/asset-bundle.json`：它故意保持 planned 与 rightsConfirmed=false，作者/授权未确认，不能直接导入。正式素材必须明确来源、作者、license、中文替代文本/署名、ready 状态与人工确认授权；工具只记录操作者的声明，不能代替授权审核。

仓库 SVG 通过 `.gitattributes` 固定 LF。编辑素材后先保存为 LF，再运行 `asset-check` 更新草稿清单的真实文件哈希；Windows 遗留 CRLF 工作副本会改变字节，不能用该副本哈希描述 Git/Linux 中的 LF 文件。`curriculum` 测试同时检查示例四张图片与 A1/A2 场景素材的哈希和尺寸。已登记版本不可原位替换，正式素材变更仍需新 revision。

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

## 部署运行巡检

`pnpm health:check --project <Compose 项目名>` 检查指定项目的五个服务与 HTTP 入口，默认访问 `http://127.0.0.1:30075`。`--origin` 可指定实际入口，`--disk-path` 和 `--minimum-free-gib` 可检查指定宿主文件系统空间。Node CLI 的退出码为 0（健康）、1（检出故障）、2（参数/脚本失败），stdout 为一行 JSON；不读取环境秘密到报告，不发送通知或自动修复。完整参数、范围与定时执行边界见 [部署巡检说明](05-deployment.md#运行巡检)。`pnpm test:ops` 会执行巡检单元和真实 HTTP 协议测试，已有 CI 命令自动包含它们；Docker 生产演练与外部告警另行验收。


### 媒体发布失败定位

`release-stage` 的媒体错误定位到 release 原文件课程条目，并附上 `imported lesson /media/0/sha256` 等固定课程投影路径；后者不是原作者 JSON 文件的行列。图片文件缺失/不可读、哈希不匹配，角色版本未登记/快照不匹配，录音登记描述/来源/文件/解码不匹配分别提供受控消息。HTTP 及 activate 保留原有 AppError，不暴露这些本地作者诊断或数据库、文件系统错误详情。不能修改已登记 revision 来修复不匹配；应登记新版本、导入新课程 revision 并重新 staging。


表达库分页空态现区分当前页和全库；继续页无记录时可返回对应收藏/复习列表。复习卡暂停/恢复按钮用 `aria-disabled` 保留键盘焦点，点击处理显式拒绝 saving、uncertain、未 ready、最新读取失败或读取期间操作；共用 owned-write 的 busy/pending 防重复门禁保持。不是只改变视觉禁用状态。


A2 第一单元四课作者草稿与 `docs/content/a2/catalog.pilot.release.json` 已加入，详见 [A2 草稿说明](content/a2/README.md)。联合目录保留 A1 固定 revision，新增 A2 等级/单元；curriculum 检查覆盖两级内容与知识一致。草稿不自动替换开发 fixture 或正式目录，没有将“结构通过”标为 reviewed。


学习完成页焦点：在未完成→已确认完成、或已完成但未确认原提交→确认成功的客户端状态转换后，焦点移到“本课已完成”标题并滚到标题。直接加载已完成页面保留普通初次加载行为；动画帧在卸载或状态改变时取消。双标签 CAS/最新读取失败/恢复及完成幂等已补实际数据库浏览器证据，范围见验证记录。


A2 一起生活进展（2026-10-06）：新增分配家务、共同空间规则、生活习惯、比较住处四课原创作者草稿，两段八轮对话与两篇六段短文，正文分别 181/156/158/208 个空白分隔词。复用 art-home-morning revision 1 与 Camille/Luc 固定快照；保留原 A1 与 A2 pilot 目录，新增联合目录包含 32 课、96 道题。七项 curriculum 测试通过，核对两级目录、共享知识/角色一致、私有字段剥离、96 道题正确及合法错误答案和八课 A2 长度/形式；四个新单课及新 release 的 CLI 结构检查、cargo fmt 与 diff 检查通过。课程仍 draft，素材仍 planned/未确认授权，未访问数据库、导入或发布课程；其余四个 A2 单元、逐课预览、人工内容审校、真实设备与生产验收仍待完成。


A2 日常事务进展（2026-10-06）：新增预约服务、填写并核对信息、解释借阅卡问题、询问后续处理方式四课原创作者草稿，三段八轮对话与一篇六段说明，正文分别 186/162/201/206 个空白分隔词。复用城市场景与 Camille/Luc revision 1；保留之前目录，新增三单元 A2 联合目录，共 36 课、108 道题。八项 curriculum 测试通过，包括固定顺序、跨级共享知识/角色、公开投影、正式 Grader 的正确及合法错误答案；新四课/new release CLI 校验和格式检查通过。表格为阅读说明，不收集真实个人信息；流程、日期和时限为虚构设定，收件确认与最终答复区分。课程保持 draft，未导入或发布，素材/人工审校/正式录音继续待完成；A2 另三个单元及设备/生产验收仍待推进。


学习撤回恢复进展（2026-10-06）：真实隔离 PostgreSQL/API/学习页复现硬撤回后 POST 410 只显示错误却保留练习和操作。现把已收到的 410（以及记录不可用的 404）设为不可继续状态；同步阻止写入/重读/原请求重试，停止播放器、收起已渲染课程内容和操作，聚焦状态标题，只清理该用户/该会话/该 revision 的步骤、答案与 pending，其他会话/版本/用户草稿保留。没有声称服务器撤回后无请求就实时推送。

隔离实测：正式 CLI 撤回后实际 POST 410；另一设备实际提示把 version 4→5，旧 POST 409 后延迟 GET，正式撤回后放行实际 GET 410；另一个旧 POST 409 的第一次 GET 注入传输失败，保留/锁定选择与填空，随后正式撤回、手动重读实际 GET 410。三条路径均收起可见正文、零学习操作按钮、H1 焦点、本会话草稿为零；直接撤回路径还核对其他 scope 草稿保留及 speechSynthesis.cancel 调用一次（不声称该设备真实法语发声）。数据库确认撤回路径没有新增尝试/完成/额外提示；两个冲突场景只保留另一设备显式保存的 1 条提示/version 5，直接撤回仍 version 4/零提示。正常未撤回样本补验旧提交 409→GET 200，保留填空草稿并按 version 5 再提交到 6，只有 1 次正确尝试。17 项 Web 测试（增加限定 scope 清理断言）、TS 7、SSR/client build、diff 检查通过；320/390px 无溢出，390px 截图已查看。新 404 分支沿用同一不可用处理，但本轮没有人为制造真实 404；完整屏幕阅读器/iPhone 和其他故障组合仍待验证。


A2 身体与状态进展（2026-10-06）：新增描述不适、预约就诊、表达情绪、读接待指引四份原创草稿，两段八轮对话/两篇六段短文，正文 206/209/175/173 个空白分隔词。联合目录 A1 24 + A2 16 课、120 道题，原目录保留。身体位置/持续时间、礼貌请求、状态形容词/原因、肯否定指令有独立练习；不推断疾病或严重度，不给出诊断、治疗、分诊或等待时限判断，任务使用虚构人物。新增原创接待室 SVG（640×470、2497 字节，SHA-256 bc9f8b9fc00c17faade6b0682ea5c371940ebe7165471325c1d0f789202e60fe），A2 独立素材清单保持 planned/rightsConfirmed=false；预约/接待课使用新图，朋友交谈/情绪短文复用室内图。十项 curriculum 测试含跨级知识/角色/投影/120 题正确与合法错误判分、16 课 A2 正文形式长度、两份素材清单真实哈希尺寸；四单课、新 release、asset-check 和格式/diff 检查通过。离线浏览器插图 320/390/900px 按比例缩放无溢出，390px 截图已查看，专用浏览器已关闭。未访问数据库或导入/激活，全部仍 draft；operator 逐课预览、人工语言/译文/教学/画面审校和正式录音待完成。A2 工作学习与经历、表达与协商两个单元及完整设备/生产验收继续待推进。


A2 工作学习与经历进展（2026-10-06）：新增简述实践经历、安排协作、说明进度、讲述昨天四课原创作者草稿，一段八轮对话与三篇短文，正文分别 166/210/171/191 个空白分隔词。复用固定 Camille/Luc 与室内图，保留之前目录，新增五单元 A2 联合目录，共 44 课、132 道题。十一项 curriculum 测试通过，核对两级目录、知识/角色一致、公开投影、所有正确及合法错误判分、20 课 A2 正文长度/形式和素材清单；四课、新 release CLI 校验、fmt/diff 检查通过。重点区分已完成/尚未完成/计划，si 条件与间接疑问，以及本课非代动词 être 过去分词主语配合；未完成过去时支持表达仍待难度审校。没有导入、发布或声称内容审校完成，全部 draft/无正式录音；A2 最后单元、逐课预览、人工审校和设备/生产验收继续待完成。


A2 表达与协商进展（2026-10-06）：新增评价体验、说明偏好、提出替代方案、澄清误会四课原创草稿，一篇六段短文与三段六轮对话，正文 179/168/166/171 个空白分隔词。保留之前全部目录，新增 catalog.full.release.json（a1-a2-full-draft-v1），A1/A2 各六单元 24 课，共 48 课、144 道题。复用固定 Camille/Luc 与室内图；个人看法不冒充客观品质，偏好不评判他人，建议经过双方确认才成为约定，澄清分别核对日期/时刻/入口。十二项 curriculum 测试通过，覆盖跨级目录、知识/角色一致、公开投影、144 题正确与合法错误判分、24 课 A2 正文形式长度和素材清单；四课与新 release CLI check、fmt/diff 检查通过。没有数据库导入或发布，全部仍 draft；语言、译文、教学难度、operator 逐课预览、正式素材/录音、辅助技术/iPhone 和生产验收继续待完成。

录音语义定位进展（2026-10-06）：课程录音描述的 assetId/revision/durationMs/sha256/url/creditZh 分别给出字段指针；过多 tracks、重复 block、空/超量 cues、未使用录音也指向对应集合或条目。时间轴区分 endMs 不晚于 startMs、越过录音时长、整句阅读顺序重叠、子区间早于/晚于父区间、缺少父 segment；保留原 cues 数组索引，不能用正文顺序冒充源位置。CLI check 通过现有 Document 索引映射原文件行列，新增九个实际子进程故障样本，包含中文和 CRLF、使用不可达 DATABASE_URL，证明确切字段值定位且不连接数据库；既有整段 cue 定位测试改为断言 endMs 值位置。新增契约测试验证乱序 cues 与子/父时间轴位置，原录音接受/拒绝规则不变。11 项契约、32 项 server 单元、10 项作者 CLI、12 项课程测试通过，Clippy 无警告；开发 API 在重新链接后恢复，/api/health 返回 200。此轮只改作者诊断，没有新录音登记、正式内容发布或生产上线；完整作者工具、人工审校和设备/生产验收仍待完成。

作者预览交互进展（2026-10-06）：隔离 PostgreSQL/API/Web 的 operator 草稿预览，合法地在两个步骤复用同一正文块，实际复现两个 knowledge-title ID 重复，第二个弹窗的 aria-labelledby 解析到第一个空标题，辅助树名称退化为整段内容。ReadingBlock 改为 React useId 实例标题，SSR/客户端关联稳定；第二个弹窗现在仅名为 baguette，全页无重复 ID，modal 初始焦点在关闭按钮、Escape 返回原词按钮。作者预览明确关闭 personalActions，不渲染收藏/加入复习组件；仍可查看与朗读，并保留三类判分预览。实际 operator 草稿题目判分后，数据库 learning_sessions/exercise_attempts/review_cards/saved_items 均为零；未发布草稿和测试素材只在隔离库。另建 synthetic reviewed 对照课程并在隔离库激活，正常账号学习页保留操作，实际收藏/加入复习各产生一条记录，未产生题目尝试。320/390/900px 弹窗在视口内、无横向溢出，390px 截图已查看，reduce 的动画 none/过渡 0s；TS 7、17 项 Web 测试和 SSR/client build 通过。专用浏览器、两个 QA 服务和临时 PostgreSQL 已清理，常用 API 健康 200。该检查不代表真实屏幕阅读器或 iPhone 验收，全部课程人工审校/录音与生产验收继续待完成。

练习反馈与重试进展（2026-10-06）：隔离 PostgreSQL/API/真实账号学习页复现确认后焦点落 BODY。ExerciseEditor 在本次提交确认后聚焦 feedback（tabIndex=-1），重试后聚焦当前题目的答案控件；初始已保存结果与外部最新记录不主动抢焦点。确认保存的恢复通知也可触发反馈焦点，完整恢复组合仍待验收。继续实测发现结果的 type=button 重试按钮在状态更新时被 React 复用为默认提交按钮，点击重试实际产生额外尝试。现为 retry/submit 使用不同 key，提交按钮显式 type=submit；新 DOM 节点避免浏览器默认动作把重试当成提交。实际键盘选择、填空、排序各自确认后焦点在该题反馈；重试分别回到 radio、保留 une 的 input、已排序语块按钮。修复前重试试验累计 4 次尝试；修复后重试保持 4，明确提交错误选择增至 5、明确提交填空增至 6、明确提交排序增至 7，各类后续重试不新增。作者草稿另合法复用填空块，两个 input 改 React useId 独立关联标签，DOM 无重复 ID；第二题实际预览判分焦点正确，重试回到第二 input，fetch 计数从 1 保持 1，数据库尝试仍为 7。320/390px 无溢出，390px 聚焦反馈截图已查看；TS 7、17 项 Web 测试与最终 SSR/client build 通过。专用浏览器、3002/5177 两个服务、临时 PostgreSQL 已清理，常用 API 健康 200。未发布正式内容；会话失效/请求失败更多组合、辅助技术/真实 iPhone、人工审校与生产验收继续待推进。

固定版本预览入口同步（2026-10-06）：AuthorPreview 原表单使用 defaultValue，目录切换后可保留旧输入；现按接受的 releaseId/lessonId/revision 同步三个可见字段，焦点转到当前课程或目录标题，初始进入不抢焦点。新增原生键盘/MemoryRouter 回归覆盖跨课、跨批次和返回；生产 SSR handler 另验证 operator 门槛、确切批次成员版本和私有缓存/session cookie 边界。三项受影响 Chromium、27 Web/10 SSR、TS7、client/SSR build 与格式/diff 通过；本轮未重跑全 35 浏览器。受控装配不代替数据库发布、正式审校/录音、真实设备/辅助技术及生产验收，详见验证记录。

预览判分身份隔离（2026-10-06）：PreviewExercise 按 operator/固定课程版本/题目隔离草稿、判分结果与等待锁，离页或身份变化取消专用请求。privateRequest 可选外部 signal 与原超时组合，并在 CSRF 读取后复查取消，阻止离开旧预览后才开始判分。真实组件受控回归确认新身份能独立作答、旧反馈不覆盖、新反馈焦点，以及切换课程期间旧 CSRF 不继续 POST。28 Web/10 SSR、TS7、client/SSR build 和定向浏览器/格式检查通过；取消客户端等待不等于服务端回滚，详见验证记录。

练习等待焦点进展（2026-10-06）：共用 ExerciseEditor 的确认按钮在有答案但 blocked 时用 aria-disabled/aria-busy 保留焦点，表单状态守卫与调用方同步锁继续拦截重复提交，答案控件仍禁用。原生键盘选择/填空/排序回归核对确切请求内容、等待焦点、连续确认仅一请求、503 后原答案明确重试及成功反馈焦点。28 Web/10 SSR、TS7、构建与受影响定向浏览器/格式检查通过；受控判分不是实际账号数据库/辅助技术验收，详见验证记录。

覆盖滚动条首屏接入（2026-10-06）：实际 Layout 的 html 原缺 overlay-scroll，现从 SSR 初始文档直接启用已有覆盖滚动条 CSS，避免客户端挂载后再切样式。生产 SSR 检查匿名/私有页面都有该类；独立现有CSS/Scrollbar装配确认四档宽度、长短内容切换无宽度变化与原生Home/End滚动。11 SSR、TS7、构建和定向Chromium/格式通过；完整生产页面壳、iPhone/辅助技术仍另行验收，详见验证记录。

访客练习焦点与生命周期（2026-10-06）：独立PracticeSession现保持等待确认焦点和同步提交锁，结果聚焦反馈，答错重试回答案控件并保留选择/文本/排序；layout清理取消请求，迟到成功/失败不再更新卸载页。三类题目的503/答错/重试/下一题/回顾及离页后新会话隔离已通过真实组件受控浏览器回归，11SSR、TS7、构建和格式通过。demo仍不写真实账号进度，正式服务/真实设备验收边界见验证记录。

生产页面壳检查（2026-10-06）：先 `pnpm build` 再 `pnpm test:browser:ssr`。测试使用当前实际SSR/client构建、Layout/路由/静态资源，在两个独立HTTP随机端口上提供受控公开fixture API；验证首页四档宽度、390px跳到正文/个人页/首页/目录的SPA焦点、语速弹窗位置/Escape返回及页面异常。该项已通过（44.80s），CI已接入；不是实际Rust/数据库/生产/iPhone验收，也未覆盖所有页面的全部宽度。原41项组件回归仍用 `pnpm test:browser`，两组测试资源均独立清理。

身份失效页面收敛（2026-10-06）：实际生产SSR/client资料页确认Bob身份已取回时，旧Alice编辑器仍在beforeunload里阻止离页；测试beacon实际证明旧资料/打开modal/阻挡监听同时存在。现Layout同步卸载旧路由和LearningProvider后再reload，清旧owner草稿并停止播放，中性页面可重新加载；不以客户端身份直接授权。两项页面壳Chromium51.44s、28Web/11SSR、TS7、构建/格式通过；受控Cookie/HTTP/focus不是实际登录/数据库/设备验收，详细范围见验证记录。155b3b0与4b95aeb的完整远端CI本轮均确认成功。

错误页恢复（2026-10-06）：根 ErrorBoundary 区分404/410/503，提供目录入口或原地址重载，恢复操作纵向排列；401/403使用身份/权限说明，400保留输入校验文字。根错误恢复导致Layout重新挂载时，RouteFocus现补PUSH/REPLACE目的页标题焦点。新增生产页面壳原生键盘回归及SSR错误投影检查，三项浏览器60.87s、28Web/12SSR、TS7、构建/格式通过；受控服务故障不是实际生产/数据库/iPhone验收，POP回退组合和完整目标门槛仍待验证，详见设计验证记录。

根错误页历史恢复（2026-10-06）：补浏览器back回归后发现上一轮PUSH/REPLACE判断遗漏POP；现ErrorBoundary退出后对已提交目标页调用共用焦点恢复，不依赖导航类型，首次挂载仍不主动抢焦点。404/410链接返回与实际back各自通过，三项生产页面壳66.43s、28Web/12SSR、TS7、构建/格式通过。真实后端/设备与完整生产门槛仍待验，详情见验证记录。

激活作者诊断（2026-10-06）：release-activate现通过activate_author调用与运行时相同事务，错误给出release参数、generation expected/current、撤回课程或固定课程ID/revision的媒体字段；不泄露SQL/连接信息。四项隔离PostgreSQL回归验证失败原子性/正常CLI激活/回滚撤回/录音，38server单元、25作者CLI、13课程与Clippy/格式通过。独立target/author-qa避开开发服务exe锁；QA容器因自动审批拒绝删除已停止并保留，baseline临时媒体同样保留。完整人工/设备/生产门槛继续见验证记录。

撤回作者诊断（2026-10-06）：content-withdraw通过withdraw_author共享运行时同一事务，给出参数、generation冲突或课程不存在/已撤回的定位，不输出SQL/连接信息。真实CLI故障不改变发布状态、generation及审计，成功撤回/重复拒绝实测；四项隔离PG、38单元/25作者CLI/13课程、Clippy/格式通过。复用QA容器已停止，baseline残留按此前删除被拒绝后的保留策略记录于验证文档；内容、设备及生产门槛仍待验。

账号步骤/完成等待焦点（2026-10-06）：学习页把异步等待与未就绪条件分开，步骤确认/完成/重试/读取控件等待时aria-disabled/aria-busy保留焦点，初始化/必需题目未满足仍原生disabled，hook同步提交锁和原请求保留不变。两项定向Chromium62.16s覆盖步骤与完成503重试的完整方法/路径/body，以及离页返回；28Web/12SSR、TS7、build/格式通过。现42项组件测试未全量重跑，真实数据库/设备、其他组合及完整目标仍见验证记录。

跨步骤恢复推进（2026-10-06）：步骤PUT成功回执现产生step ID/幂等key确认，页面只消费一次推进；原存储请求不能恢复onSaved回调的断点已修复。实际多步骤离页/返回/原提交重试、回看不重复推进和主动正常确认已通过，连同单步骤完成及既有离页测试三项72.92s、补充用例42.46s；28Web/12SSR、TS7、构建/格式通过。现43组件用例未全量复跑，受控HTTP不等于实际数据库/真实设备验收，详情见验证记录。

进度冲突读取补验（2026-10-06）：新增受控GET学习记录装配，409→GET503→手动GETversion7不自动推进，用户新确认才使用新版本/key；下一步骤409→GET410移除正文/操作、清会话草稿并聚焦撤回标题。最终Chromium43.57s、TS7/格式通过，没有修改生产逻辑或重新构建；现44项组件回归，本轮仅运行新增用例。完整真实后端/设备/内容及生产门槛见验证记录。

组件浏览器草稿隔离（2026-10-06）：共用Chromium的独立测试在beforeEach清理sessionStorage，同一测试内仍保留离页/返回/重试所需草稿。修复完整CI中上一多步骤测试残留回顾步骤导致冲突用例及待确认保存用例失败的问题，新增保存成功/冲突读失败/撤回时弹窗关闭、原路由和标题焦点检查。四项定向85.43s、完整45组件642.11s通过，无生产应用改动；未重复构建/TS7/Web/SSR或生产页面壳。原42bc05d完整CI实际failure，修复提交须另行确认；设备、内容及生产门槛见验证记录。

本机Docker已实际启动（2026-10-06）：本次使用被忽略的.local/docker.env，维护命令为docker compose --env-file .local/docker.env --project-name brioche ...；未覆盖开发.env。迁移exit0、四常驻服务healthy，HTTP30075的loopback/LAN巡检成功，栈保持运行。正式目录为空，管理员邀请待用户邮箱，未发布未审校内容；实际手机/公网与完整业务验收继续保留。知识点已统一分类笔记卡片，操作入口移除长箭头，详见验证记录。

导入前语义定位补齐（2026-10-06）：新增回归实际复现必需教学文本等非法内容在check可定位、import却先进入环境检查；现PublicLesson.validate_intrinsic统一课程自身的ID/引用/教学文本/步骤/练习等检查，import连接数据库前复用。audio/登记描述仍延后补齐，完整validate与发布媒体校验保留。38server单元、25作者CLI、13课程、19公共契约及Clippy/fmt通过，生成契约无差异；未改HTTP契约或审校/发布数据。Docker API/migrate已按当前源构建并up --wait更新，迁移exit0、四服务healthy，loopback/LAN HTTP30075巡检均通过、零重启；设备、正式内容/录音与生产门槛继续保留。

导入前私有规则引用校验（2026-10-06）：回归复现check可定位未知正确选项/排序语块，import却先报环境错误；现import前置语义检查对原课源调用同一Grader::from_author_source，核对规则与真实练习块及非空反馈，不输出私有答案。补三项import字段场景，38server单元、25作者CLI、13课程与Clippy/fmt通过；完整48课离线目录通过。本轮未改公开契约/Web/课程数据，未重跑此前19契约/浏览器或PG事务；实际Docker API/migrate已构建并up --wait更新，迁移exit0、四服务healthy，loopback/LAN HTTP30075健康、零重启。eb7fd8f固定提交CI37473650814本轮实际in_progress；完整人工内容/录音、真实设备及生产门槛继续保留。

管理员操作（2026-10-07）：operator 在个人页进入 /admin，课程批准/退回在后台完成，预览页只读；后台可导入 JSON 课程源及发布目录，填写理由后调用与 CLI 共用的不可变版本/素材/审批校验。批准不等于激活。首六课已按用户指示实际导入生产并激活，目录 generation1；后续切换读取当前 generation。部署前备份与服务检查已完成，CLI/网页审计不冒充彼此 actor；详见10-admin-development.md、content/releases/README.md。

Qwen后台音色创建：可选-f compose.tts.yaml仅给server加载.local/tts.env，不向Web/迁移传密钥。生产维护仍须带-f compose.https.yaml；若启用TTS，追加-f compose.tts.yaml，避免下次up丢失提供方配置。不要打印Compose完整config或容器环境。/admin/voice-jobs保存不可变创建/查询状态，未知结果找回只查已有音色；详情和剩余试听/正式发布门槛见characters/README.md。
