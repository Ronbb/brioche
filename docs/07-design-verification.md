# 设计验证记录

## 2026-10-06：手机词汇抽屉的模态焦点

- 实际在公开阅读页 390px 点 Bonjour 后按 Tab，原知识 aside 虽已展示，activeElement 仍落在其外的背景正文，证实手机覆盖面板缺少模态焦点管理。
- 新增 ResponsiveKnowledge：桌面保持原侧栏，手机采用 native dialog/showModal；同一时刻只挂载一份词汇/语法视图，避免重复收藏/复习控件。手机抽屉沿用原底部视觉、品牌配色、安全区与动效，用原生 backdrop 取代背景关闭按钮；对话名称关联当前词汇/语法标题。Escape、关闭按钮及外部点击关闭会清理当前解释，原生对话框负责焦点隔离与返回。
- 独立 agent-browser 会话实测打开后可访问树只展示当前解释，连续 Tab 焦点留在 dialog 内；Escape 与关闭按钮后 dialog closed 且焦点返回 Bonjour。390px 截图核对底部抽屉构图；320px 无横向溢出。900px 打开状态变为侧栏且保留 bonjour，缩回 390px 恢复同一词的模态焦点，始终只有一个标题 ID。减少动效时抽屉 animationName=none。
- TS 7 typecheck、client/SSR build、15 项 Web 协议测试和 diff 检查通过。这里验证访客公开阅读页；账号控件的宽度切换、辅助技术实际朗读、真实 iPhone 与完整故障流程仍待验收，不作为整体完成证明。

## 2026-10-06：键盘入口、页签与减少动效

- 补全局“跳到正文”链接，只有键盘聚焦时显示，不占内容空间；正文 main 可接收程序/片段焦点。独立 agent-browser 会话从 profile 页面首次 Tab 实际聚焦链接，Enter 后 activeElement 为 page-content/main。
- 设置页语速按钮改为包含字段与当前值的可访问名称，声明弹窗及其控制目标；通用选择面板触发器名称同时包含当前选择。浏览器可访问树显示“朗读速度：1×”；键盘进入速度面板，End 选中 1.5×，Escape 关闭后焦点回到“朗读速度：1.5×”。
- 公开阅读页对话/短文 tab 补 roving tabindex、ArrowLeft/Right 循环、Home/End 与 tabpanel/aria-controls/aria-labelledby。实测 ArrowRight 从对话切到短文，焦点/selected/唯一 Tab 入口和面板名称一致，正文为短文；Home/End 和末项循环返回对话正常。原切换停止朗读行为保留，未改变课程数据与学习记录。
- 浏览器减少动效设置为 reduce 时，阅读页当前所有元素 animationName=none、transitionDuration=0；320px/390px 无横向溢出。这里只检查当前公开阅读/个人页，不代表整个产品所有状态或真实辅助技术/iPhone 已验收。
- TS 7 typecheck、client/SSR build 和 15 项现有 Web 协议测试通过。未为这些可逆样式/标签修改添加镜像实现的单元测试；使用真实 DOM/键盘证据。完整屏幕阅读器、账号学习/复习/嵌套弹窗和真实 iPhone 仍继续待验收。

## 2026-10-06：素材/录音导入原文件诊断

- `assets-import` / `audio-import` 在数据库连接前保留原作者 Document，严格类型解码和元数据校验映射原文件行/列。元数据规则与正式导入复用，覆盖 ID/revision/重复、状态与授权、来源/署名、SHA-256、MIME、文件路径、视觉尺寸/角色和录音时长。planned 素材仍拒绝；没有改写素材权利声明或发布内容。
- 文件层补具体清单项路径：不存在/逃逸源目录/格式错误定位 file，哈希不匹配定位 sha256，尺寸/解码时长不匹配定位 width/height/durationMs；角色缺登记头像 revision、非正方形头像分别定位角色引用。根存储/数据库错误与数据库重复版本内部细分尚待完善，HTTP 对外不暴露作者诊断。
- 32 项服务端单元、9 项作者 CLI 和 2 项课程包测试通过；新增 CLI 以不可连接数据库和不存在源目录验证 14 类元数据错误先报精确 CRLF 原文件行/列，不连接数据库。隔离 PostgreSQL 实际 CLI 验证图片/音频各 3 类文件故障，同样精确定位并核对登记表/审计表没有新增；原迁移发布与录音不可变/事务测试通过，all-targets Clippy 与 fmt 通过。
- 演练只使用专用 brioche-author-media-qa 容器、临时 schema 和 LicenseRef-TestOnly 合成材料；检查结束删除专用容器及其卷，开发 3001/5173 恢复、health 200，不涉及用户生产数据库或 DNS/入口。生产发布/审校和剩余功能验收继续待完成。

## 前端统一录音播放器（2026-10-06）

Node 播放器协议测试通过：媒体时间驱动进度/词高亮/片段结束，暂停保持位置且暂停期间的 play Promise 不重启；旧监听/metadata/帧与拒绝 Promise 不干扰新片段；同一文件只创建/加载一次，设备外部暂停可见；实际媒体过短/提前 EOF/权限拒绝/加载失败有明确失败回调；Unicode scalar 对齐和不同 entry 下重复 segment ID 不冲突。共十项 Web 测试（五项播放器、五项既有恢复）、TS 7 检查和 SSR/客户端构建通过。

使用隔离 PostgreSQL/API/Web 与本机 12 秒合成 WAV，在真实 Chromium 观察原生 Audio 元素（未替换其播放实现）。公开课程首次 Bonjour 单词区间为 0.1–0.6 秒，采样观察实际 currentTime 推进与词高亮；头像整句约 1.000 秒停止并展开“您好！”。全文约 8.002 秒结束、进度 100%，一次 Audio 实例；短文约 3.008 秒结束且没有头像。暂停位置 0.75432 秒在后续观察中不变，自定义面板改为 1.5 倍后仍保持该位置，恢复后到 1.562123 秒再暂停。全文进行中改点词，复用一个原生元素，在约 0.603 秒结束，没有被旧 pause 事件干扰。实际磁盘文件腐坏触发媒体 503，零法语设备声音环境显示“录音暂时无法播放，请重试。”并恢复闲置；恢复文件后再次点击进度正常推进。390px 无横向溢出，查看暂停时正文/词高亮截图；键盘 Shift+F10 打开速度面板。HMR 页面重载使两个观察等待失去测试对象，按失败记录并重新加载观察，未将它们当作播放验收。未验证真实 iPhone、法语 TTS 可用设备、账号/私有预览前端、完整键盘与后台行为；合成音不证明法语审校。

## 录音登记、发布与媒体（2026-10-06）

独立 PostgreSQL 实测第十个迁移 up/up/down，录音登记并发仅一方成功、版本不可更新/删除、审计不可改、包含重复旧版本的整批写入回滚；拒绝无授权、错哈希、错时长、MIME 伪装与路径越界。课程 audioRefs 精确取得版本并覆盖伪造描述，公共投影不含来源/授权/原文件路径。真实 release-stage/activate 使用协议课程走全套视觉/录音文件与时间轴校验；登记与 staging 的公开请求均 404。发布后 MP3 GET/HEAD 完整内容与单段/开放末尾/后缀/截断末端 Range、206/416、重复 Range、ETag/If-Range 均通过。管理员私有 URL 对匿名 401、普通用户/降权 403、跨课程素材 404；同课程预览可分段读并返回 private/no-store。篡改磁盘文件后公开/私有请求 503，发布验证拒绝；恢复后正常，再硬撤回则公开 404、私有 410。其他身份、学习、发布与 PostgreSQL 集成测试同步通过。使用一秒合成音和隔离协议课程，不代表法语教学/配音人工审校或前端播放验收。

## 录音文件检查（2026-10-06）

新增 Symphonia 0.6.1 MP3/WAV 全流解码检查和数据库无关 `audio-check` CLI。使用提交的一秒 440 Hz 合成 MP3，实际解码得到 1000 ms、24 kHz、单声道；PCM WAV 验证 1001 ms 向上取整和双声道。覆盖截断 MP3/WAV、MIME 伪装、错误 RIFF/chunk/对齐/字节率、超 32 MiB、超 30 分钟、异常声道/采样率、ID3 长度越界。CLI 在不可连接数据库和生产 fixture 环境组合下仍成功，输出 JSON 与 stderr 日志分离。测试素材没有第三方录音，不代表法语发音或教学审校。此阶段不改变录音发布的拒绝规则，登记、媒体访问和播放器尚待实现。

## 初期设计验证（2026-10-05）

日期：2026-10-05。范围：当前文档、Schema 草案、面包店示例 JSON 和原生 HTML/CSS/JS 概念稿。

## 已执行

- 用 JSON Schema 2020-12 实现校验 Schema 自身及示例文件，JSON 解析拒绝重复 key。
- 一次性检查样例内的 block/step、角色、正文锚点、词汇/语法、完成条件、三种题型规则及复习引用；当前 12 个 blocks、6 个 steps、3 个 exercises 通过。
- 检查示例文本 NFC、文档相对链接、插图 SVG XML 以及概念稿 JavaScript 语法。
- 浏览器检查首页 → 阅读 → 练习，词汇解释切换、译文显示、手机知识抽屉及 Esc 关闭。
- 浏览器检查单选空答案不可提交、错误反馈、重试和正确反馈；未向任何 API 提交数据。
- 对首页/阅读/练习分别检查 320、390、768、1440px 宽度，无页面横向溢出；查看桌面首页、手机首页和手机知识抽屉截图。

## 尚未验证

- Rust/Axum/SeaORM、Vite/TS 7/React Router 的实际依赖组合与生产构建：应用尚未创建。
- Schema → Rust 类型 → 公共 TS/OpenAPI 的生成链路、正式语义校验器和内容发布器：仍为设计。
- 数据库关系、事务、权限、cookie/CSRF、跨设备、断网与内容版本行为：没有 API，尚未集成验证。
- 法语教学内容的人工审校、正式图片授权清单与音频制作：样例仍为 draft，正式素材引用未解析。
- 完整屏幕阅读器/WCAG 审计、真实手机体验与公网 TLS/备份恢复：后续实现阶段验证。

一次性样例检查不能替代正式解释器、完整语义校验器或教学审校；概念稿交互检查也不能视为生产 App 测试通过。

## 角色、朗读与品牌迭代

- 增加 3 位角色、原创头像及课程 cast 快照；新增三段短文示例。外置脚本执行 Node 语法检查，公共示例数据不包含 serverOnly 判分规则。
- 浏览器检查 8 轮对话及头像加载、全局/单句译文、词语释义、手机抽屉关闭、原文面板及选项保留、练习反馈。
- 在隔离的检查浏览器中使用临时 Speech API 替身验证：全文顺序、角色轮换、暂停/继续、停止、切换点词/整句、旧回调失效、文章播放完成、缺法语声音状态。替身只在检查会话使用；这些结果不代表真实音频、系统声音或 iPhone Safari 播放通过。
- 首页/阅读/练习在 320、390、430、768、1440px 下没有水平溢出；检查手机阅读与桌面首页截图。品牌配色及 Apple 系统字体回退已应用，实际 Apple 字体渲染仍需 iPhone 检查。
- 预览服务监听所有网络接口，监听地址 `::`；本机向当前 LAN 地址的 HTTP 请求返回 200。同网手机、路由/防火墙路径尚待用户实际设备验证。

## 阅读界面收敛

- 阅读页移除所有语言开关，角色集中介绍、正文只显示头像；短文采用纯段落。新增设置页，默认译文与朗读速度只在本次页面会话保存。
- 用隔离浏览器和临时 Speech API 替身检查 25 个状态断言：头像同时翻译/朗读、局部译文不改全局偏好、设置切换、整条播放线暂停/继续、boundary 进度、长按面板、抬手不误播、变速和暂停保留、旧回调失效、无声音时仍可翻译、纯短文播放及原文保留答案。
- 实际浏览器点击设置入口/译文开关，使用 Shift+F10 打开速度面板；查看手机对话、设置和速度面板截图。首页、对话、短文、练习、设置在 320/390/430/768/1440px 无水平溢出，浏览器未报告脚本错误。
- Node 语法、Schema/引用及本地文档链接校验通过。长按采用 Pointer Events，真实 iPhone 的触摸、系统 Select 与语音仍需实机验证；替身不能证明真实声音播放效果。

## 浏览器标注修正

- 移除头像顶部 5px 偏移，将对话首行行高统一为 44px，与头像中心对齐；原文面板采用同一规则，头像悬停不再位移。
- 统一去掉首页入口、课程列表、阅读“练习”和练习提交的装饰箭头。检查按钮无字符图标，剩余操作图标为 SVG namespace 下的路径图形，无 SVG text 元素。
- 320/390/564/1440px 下 8 个对话头像中心与首行中心误差为 0，正文左边界一致；查看用户标注对应 564px 宽度的完整截图。脚本语法及浏览器错误检查通过。
- 更新预览 CSS/JS 的资源版本标记，避免浏览器复用旧样式。本次为静态概念稿修正。

## 表达卡片与参考配色

- 首页表达卡片改为原生 button，整个区域点击展开/收起，不含嵌套按钮。实际浏览器检查点击、Space 收起、Enter 展开，aria-expanded 与释义可见性一致。
- 采用用户参考色的暖橙/浅金/奶油/嫩绿组合，按钮使用深棕文字、绿色卡片使用深绿文字；同步插图配色与资源版本标记。
- 主按钮、正文、次要文字、表达卡片、链接文字的实色对比度分别为 7.14、13.62、6.67、4.99、5.80:1；仅为这些配色组合的计算，不代表完整无障碍审计。
- 卡片展开状态下检查 320/564/1440px 首页及其他页面无横向溢出，查看 564px 卡片截图，脚本与内容校验通过。

## 动效迭代

- 正常动态偏好下检查表达卡片高度过渡、快速展开/收起、按压波纹清理、阅读淡入与头像对齐，均通过。
- 减少动态偏好下 CSS/JS 动画数为 0，卡片仍立即展开；在运行中切换偏好可取消动画。
- 原有阅读交互的 25 个状态检查仍通过，包括长按调速和旧播放回调失效。浏览器无脚本错误；实际 iPhone 动效观感与语音仍需设备验证。

## Toast 与自定义速度面板

- 移除无导航功能的步骤示意，语音错误不再占用正文布局。Toast 重复触发不堆叠，约 5.5 秒自动关闭，可手动关闭，不抢焦点。
- 移除设置页与速度对话框全部 native select，改用统一 radio group，当前选择聚焦、方向键选择、设置/阅读同步。
- 隔离浏览器检查 19 个状态断言，包括 toast 布局稳定、重复/手动/自动关闭、键盘选速、共享面板、播放使用所选速度、暂停后调速与续播。Speech API 使用临时替身，实际设备声音仍未验证。
- 实际键盘 ArrowDown + Space 将速度从 1 改为 1.25，并关闭面板回到设置入口；查看 564px 自定义面板截图。320/564/1440px 的页面布局检查、Node 语法与内容校验通过。

## 复习页与个人入口

- 新增独立复习页，队列来自示例课程 3 个 reviewItemIds；支持整卡揭示、三个自评等级、轮次进度、回顾、只重练未熟项及全部重来。
- 检查 21 项状态断言：唯一 DOM ID、导航/当前项、揭示前不可评价、快速重复点击防跳题、离开返回保留卡片、全局译文不提前揭示、回顾计数、未熟重练、重置、临时语音替身播放、头像进入个人信息、阅读偏好及头像加载。
- 右上角新增原创示例用户头像，个人页展示 Lin 的示例资料及设置。底色改为 #FFFEFA / #FAF9F5，保留品牌橙与绿。
- 查看手机复习卡片、个人页与完成回顾截图；320/390/564/768/1440px 检查包括复习完成态在内的全部页面无水平溢出，Node 语法/内容校验通过，浏览器无脚本错误。
- 复习记录与个人资料仅为内存 demo，不代表账号 API、排程、持久保存或真实设备语音已实现。

## 复习操作与滚动条调整（2026-10-05）

- 自评由三个并排按钮改为连续的纵向选择列表；完成页采用单一主操作与下方文字入口，全部记住时主操作自动变为重新开始。
- 移除复习卡片下方的轮次/返回行与独立朗读按钮，点击卡片展开并朗读；增加高度、释义、例句和选择区过渡。
- 新增覆盖式页面滚动条，原生滚动保留，细条不占内容宽度；验证键盘滚动、320/390/564/1440 宽度无横向溢出，展开前后内容宽度不变。
- 浏览器检查通过揭示后自评、连续点击防跳项、完成计数、未熟项重练、全记住后重启、减少动态效果模式。朗读使用隔离浏览器模拟法语语音验证调用；真实 iPhone 语音仍需设备验证。
- 查看 390px 下展开卡片、自评列表与完成页截图；两个 JS 文件语法检查通过。

## 顶部主导航调整（2026-10-05）

“今天 / 课程 / 复习”提到品牌栏同一行；手机小屏保留品牌图形和个人头像，预览调试条移至底部。浏览器检查 320/390/564px 下导航、品牌、头像同一行，无横向溢出；查看 390px 截图。

## 简化顶部栏（2026-10-05）

移除顶部“今天 / 课程 / 复习”导航，品牌与头像成为仅有的顶部入口，窄屏恢复完整品牌文字。首页表达卡片下方新增“复习这组表达”文字入口。浏览器验证 320/390/564px 下首页、课程、复习、个人页入口均可用，且无横向溢出；查看 564px 顶部截图。前一节顶部导航方案已被此方案替代。

## 第一轮工程验证（2026-10-06）

- 已建立 pnpm/Cargo workspace；Rust fmt、clippy（warnings as errors）、6 个单元测试、TypeScript 7 类型检查、Vite client/SSR 构建通过。
- PostgreSQL 18.6 独立测试 schema 验证：迁移重复执行、重复 revision 拒绝、草稿不可见、最新已发布 revision 读取、公共响应不包含私有字段。普通 cargo test 跳过此测试，显式配置测试 URL 后另行通过。
- React 页面浏览器检查：头像译文/整句朗读、点词与知识抽屉、短文纯段落、全文暂停、长按调速、暂停变速后续播、复习自评计数、个人设置与品牌返回；语音采用隔离浏览器模拟法语声音，实机语音仍待验证。
- 320/390/768/1440px 下首页、对话、短文、复习展开与个人页无横向溢出，对话首行词按钮与头像中心对齐；减少动态效果模式可操作。SSR HTML 含课程正文，SSR 数据和 client bundle 未发现 grading/数据库字段。
- API、Web、Caddy 三个 Linux 镜像构建通过。隔离 Compose 项目验证 PostgreSQL → migrate → API → SSR Web → Caddy；只映射本机 18080 测试端口，health/readiness 与同源路由可用，SSR 空目录显示准备中。导入草稿仍不显示，生产 fixture 模式明确拒绝。
- Web 与 API 开发服务本机/局域网访问成功。未改 DNS、路由器或启动公网生产服务。
- 当前完成工程基础和阅读链路；账号、判分、进度续学、完整步骤/发布流水线与正式媒体仍按工程说明继续实现。

## 2026-10-06：示例练习与速度面板修正

- Rust 增加三类私有判分规则、导入前检查、公共提交/反馈契约；NFC、法语弯引号与空白规范化保留重音。选择/排序只接受当前题定义的 ID。单元与 HTTP 测试合计 12 项通过；PostgreSQL 测试本轮未重复运行，迁移未变更。
- `cargo fmt --all --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`pnpm contracts`、`pnpm typecheck`、`pnpm build` 通过。
- 在 390×844 浏览器验证：错误选择反馈 → 再试 → 正确选择 → 填空 ` UNE ` → 排序 → 三题回顾。判分走真实 Rust API，无成功响应替身。示例结果不保存到账号，生产数据库模式不开放 demo 判分。
- 正文加入结构化教学补充及语法锚点；完整学习步骤与多正文块流程仍在实现清单中，不能宣称课程解释器全部完成。
- 用户反馈速度面板贴左上角；原因是 CSS reset 覆盖原生 dialog 自动 margin。显式 fixed/inset/auto margin 后，564×884 截图与 320×568、390×844、768×1024、1440×900 几何检查通过，面板中心误差小于 1px。
- 564×260 横屏检查：面板保持 16px 上下边距，内区可滚动且不占额外宽度；无页面横向溢出。自定义 radio 保留 roving tabIndex，Escape 关闭。真实 iPhone 语音与安全区仍需实机验收。

## 2026-10-06：账号基础与 Traefik 入口

- 新增 browser_sessions 显式迁移与 SeaORM/tower-sessions 适配，实际 PostgreSQL 验证创建/冲突重试/更新/过期/清理/幂等删除；删除后旧 save 返回错误，不恢复会话。库级 cycle_id 更换 ID 且删除旧记录，flush 删除新记录。数据库不含原始 cookie ID。
- 测试 Router 验证 HttpOnly/Secure/SameSite=Lax/Path=/ cookie、精确 Origin 与会话绑定 CSRF。缺少/错误 Origin、缺少/伪造 token、跨会话 token 均 403；nonce 轮换后旧 token 403、新 token 成功。这里是基础模块测试，不代表生产登录路由已完成。
- 用户指定入口 Traefik、HTTP 30075、外部自行处理 HTTPS；已移除 Caddy 文件和 Compose 证书卷声明，新增只读 file provider。官方 traefik:v3.7.13 拉取成功并固定 digest。
- 隔离 brioche-traefik-test Compose 使用已有 Web/API 测试镜像验证新网关，PostgreSQL/migrate/API/Web/Traefik 启动健康，HTTP 30075 的首页 SSR、/health、/api/health、/api/ready、/api/catalog 全部 200。此烟测验证入口替换，未宣称重新构建本轮账号模块镜像。
- 网关以非 root 运行，无 Docker socket 和公开管理入口；healthcheck 经内部 127.0.0.1:8082/ping 成功，只有 8080 映射到宿主 30075。测试 override 临时限定 loopback，共享 Compose 默认映射全部接口。测试资源随后清理，不修改外部 HTTPS/DNS/路由器。

## 2026-10-06 认证接入验证

- PostgreSQL 18 临时隔离容器：所有迁移、课程读取/唯一约束及认证集成测试通过。认证覆盖邀请重放/并发消费/重新签发/到期、Origin/CSRF 拒绝、登录轮换、密码恢复撤销两端会话、旧密码拒绝、退出及未知邮箱限流。
- Rust workspace 单元测试通过；clippy 无警告，TS 独立类型检查与 Vite client/SSR 构建通过。
- 浏览器 390×844：一次性邀请注册后 SSR 个人页显示实际昵称/邮箱，退出后恢复访客登录入口。修复 Strict Mode 重复 effect 清空邀请 token 的问题。未宣称真实 iPhone 或生产公网验证。
- Compose 入口继续固定 Traefik HTTP 30075，无 TLS/证书端口；PUBLIC_APP_URL 用于浏览器 Origin 与 Secure Cookie，HTTPS 仍由用户在外部处理。

## 2026-10-06 个人设置验证

- 新增显式 profile_version 迁移、Rust UserSettings/UpdateProfileRequest 生成契约及受认证/Origin/CSRF 保护的 PATCH /api/v1/me/settings。字段 allowlist 禁止修改角色、邮箱或用户 ID；目标和语速必须属于指定档位，空昵称/空更新/无效时区均拒绝。
- PostgreSQL 集成测试：设置从另一次登录读取；两个相同版本的并发写只有一个成功，另一请求 409；其他账号保持默认设置，越权字段被拒绝。全套迁移 up/down 通过。
- 浏览器 390×844：改昵称、选择巴黎时区/每周 3 天、切换译文与 1.25 倍速，刷新后全部恢复；模拟另一设备更新，旧页面提示冲突并刷新已保存设置。320×740 页面宽度等于视口，没有横向溢出；自定义资料弹窗居中，时区搜索“巴黎”可选择结果。
- 快速连续的阅读偏好修改在客户端按序提交版本，保留最新待保存视觉状态。网络异常不自动重放 PATCH；读取服务器现状并明确提示，编辑草稿保留。
- 时区采用 [Jiff 0.2.37 的 IANA 数据库](https://docs.rs/jiff/0.2.37/jiff/tz/struct.TimeZoneDatabase.html)，包含 bundled fallback，避免 Windows 或精简容器缺少系统时区资料；后续复习排程使用同一来源。真实 iPhone、公网与学习进度仍待验收。

## 2026-10-06：固定版本账号学习

- Rust fmt、clippy（全部 target，warnings 为错误）与 workspace 测试通过；TS 7 类型检查、SSR/客户端生产构建通过。
- 独立临时 PostgreSQL 的认证、迁移/发布读取、学习集成测试均通过。学习测试覆盖跨账号隔离、并发创建/提交、固定 revision 判分、首次/重试/提示记录、伪造载荷拒绝、幂等重放/冲突、完成事务失败回滚、首次完成保持、复习卡片去重、撤回与概览分页；没有修改正式课程的人工审校状态。
- agent-browser 独立 QA 账号在 390×844 浏览器完成全部六个步骤和三类练习，刷新后仍显示完成，返回首页显示“已学过”。阅读步骤同时渲染对话与短文；点词解释弹窗内已能访问语音错误 toast 的关闭按钮。
- 真实 iPhone、完整尺寸/键盘检查、浏览器网络丢失后的重试和两标签页草稿恢复仍需补验；账号复习排程、收藏、正式内容发布与审校仍未完成。
- 浏览器故障注入：步骤请求在服务端返回成功后故意丢弃响应，页面显示“重试保存”；明确重试的两份载荷完全一致，随后进入阅读步骤。尚未覆盖离开页面或两标签页冲突时的未提交草稿恢复。

## 2026-10-06：账号复习排程

- PostgreSQL 集成验证到期过滤、10 项上限和稳定顺序、账号隔离、自评载荷白名单、旧版本/未来卡拒绝、原键重放、不同载荷冲突、并发更新仅一次成功、失败回滚、时区变更保留 UTC 排程，以及撤回屏蔽队列和重放；三组既有数据库测试均通过。
- 纯规则测试覆盖所有档位与三种自评、初始档位、上限、巴黎夏令时切换和上海本地日期。Rust workspace 测试、clippy、TS 7 类型检查和 Web SSR/客户端构建通过。
- agent-browser 隔离账号/临时数据库在 390×844 完成三张卡，数据库保存三条实际自评。第一张响应故意在提交后丢弃，重试载荷完全一致，数据库仅新增一条。刷新后到期队列为空并显示下一次复习日期，页面没有横向溢出。卡片由临时 QA 数据生成，正式示例仍未人工审校。
- 收藏、暂停复习操作、复习历史页、离页保存恢复、完整尺寸/键盘与真实 iPhone 验收仍需继续实现/验证。
- 同一复习卡展开后在 320 与 1440 宽度检查，均无横向溢出；这不替代完整可访问性或真实 iPhone 验收。

## 2026-10-06：收藏与复习管理

- 临时 PostgreSQL 覆盖收藏首次来源保持、取消后重建保留时间/版本、跨账号隔离、原键重放与载荷冲突、并发取消仅一次成功、未知知识拒绝、收藏与复习互不删除、手动加入已有卡不重置、暂停拒绝新自评、恢复保持排期/档位、撤回正文隐藏及仍可取消收藏。收藏 25 项分页 20+5，历史 32 条分页 20+12，无重复，并拒绝错误游标。认证/迁移/学习三组集成测试通过。
- Rust workspace 测试与 clippy、TS 7 类型检查及 Web 构建通过。
- agent-browser 独立账号在 390×844 从实际阅读词汇解释收藏/手动加入，再进入复习管理暂停、刷新验证状态、恢复、提交自评、读取历史和取消收藏。数据库显示收藏已取消（version=2），复习卡保留（version=4，stage=1）及 familiar 历史。收藏响应在服务端成功后故意丢弃，重试两份载荷一致，页面确认已收藏；收藏页展开没有横向溢出。
- 完整尺寸/键盘/离页恢复和真实 iPhone 验收仍需补齐；所有发布测试均在临时数据中，正式示例没有伪造人工审校。

## 2026-10-06：实际学习首页

- 新学习概览 PostgreSQL 验证账号隔离、打开会话不计活动、重复完成只计一课、幂等重试不增加周活动、硬撤回保留历史总数且屏蔽当前课程/到期入口，以及未完成课程在 20 项分页之外仍能续学。跨时区测试让同一 UTC 自评分别落在 Kiritimati 周一和 Honolulu 周日；纯规则验证巴黎夏令时 23 小时日与本地周边界。
- agent-browser 独立账号/临时数据库在 390×844 验证首页从 0 天变为实际学习 1 天，从阅读加入表达后到期数为 1，首页进入复习并保存自评后到期数为 0、显示下一次日期，周记录包含实际复习 1 项。读取和操作均走现有账号 API，没有修改正式示例的审校状态。
- 首页在 320、390、768、1440 宽度没有横向溢出，390 和 1440 截图检查了周记录、复习入口与续学布局。该证据不替代完整键盘、reduced-motion 或真实 iPhone 验收。每天分钟数仅显示账号目标，尚无实际学习时长测量；课程推荐的教学顺序仍待目录 release。

## 2026-10-06：目录 release 与撤回

- 新迁移与 PostgreSQL 集成测试验证未激活目录为空、stage 不可见、显式名称/顺序、未审校与缺引用整批拒绝、发布审计失败时指针/availability 一起回滚、两个相同 generation 并发切换仅一个成功、新会话固定 active revision，以及普通回滚后旧/新两个会话都可继续。
- 硬撤回实际返回 410，含撤回 revision 的包不能激活，直接把 withdrawn 版本 published 改回 true 也被数据库拒绝；当前目录过滤撤回课程，学习概览继续推荐其余课程。显式空 release 可清空目录。触发器实际拒绝正文、release 清单和审计改写，以及向 staged release 追加条目；迁移完整 down/up 与既有认证/学习测试通过。
- 这些测试使用隔离 schema 的合成审校元数据，只验证发布协议，不代表任何法语内容通过人工审核。完整媒体/授权/角色库与 staging 预览仍待实现；现有示例文件保持 draft。
- 精确 revision API 验证普通回滚后仍可读取旧公开快照、无 revision 时读取当前目录、错误 revision 拒绝、硬撤回的精确版本返回 410。账号首页的目录和推荐来自同一事务快照，正文按明确 revision 读取。

## 2026-10-06：视觉素材与角色库

- 素材/角色/导入审计迁移，登记后不可变；发布再次验证注册描述、精确角色快照、头像 revision 及存储哈希。隔离协议测试使用合成授权声明，不代表示例课程已完成人工审校或素材授权确认。
- PostgreSQL 发布测试覆盖未确认授权、planned、错哈希/尺寸、路径逃逸、篡改角色、激活前损坏文件、私有 staging 素材拒绝公开及最后引用撤回后 404；图片接口校验正确内容/安全 headers 和损坏 503。
- 独立浏览器 390px 检查数据驱动插图与头像，页面 scrollWidth=390。临时移走测试头像发现 SSR 接管前 onError 漏报，已补接管检查；复测所有缺失头像均回退到 learner.svg 并成功加载，测试文件已恢复。真实 iPhone 与其他宽度仍需整体验收。
- 本轮 cargo fmt、workspace clippy（warnings 拒绝）、18 项契约/服务端单元测试与四项 PostgreSQL 集成测试通过；pnpm contracts、TS 7 类型检查和 Web/SSR build 通过，Compose config 校验通过。PNG/JPEG/WebP 正向解码、错 MIME 与截断数据通过单元验证。

## 2026-10-06：课程浏览与搜索

- `/courses` SSR GET 表单与 `/api/catalog?q=` 搜索当前 release，保持教学顺序；中文场景、法语标题、多词、大小写/重音/全角规范化、无匹配与查询边界通过 API 单元测试。数据库测试确认草稿不进搜索，硬撤回后仅剩可用课程。
- 独立浏览器实测 Enter 提交 BOULANGERIE 返回一课、URL 保留查询；无匹配显示空态和全部课程入口。390px 与 320px 无横向溢出，空态截图已检查并补齐页面安全区边距。真实 iPhone 待验证。
- workspace clippy、单元测试、相关 PostgreSQL 发布集成测试、TS 7 类型检查与 Web/SSR build 通过。

## 2026-10-06：学习草稿与未确认请求恢复实现

- 账号学习页接入按账号/会话/revision 的 sessionStorage 答案、步骤与原请求存储，恢复时保留原幂等键并禁止新提交；退出清理当前账号，存储拒绝提示。迟到响应只清除相同 pending key，完成页保留未决请求入口。
- pnpm test:web 两项协议测试覆盖固定课程答案、endpoint/版本/key/额外字段拒绝、未知/重复词块、owner/revision 隔离、精确 JSON、损坏/拒绝存储和迟到旧请求不清理新请求。TS 7 检查通过，Web/SSR build 通过。尚未宣称真实浏览器刷新和两标签页流程通过；这些与复习/收藏离页恢复继续验收。

## 2026-10-06：收藏与复习未确认保存恢复

- 收藏、加入复习、暂停/恢复与自评均保存原请求并明确重试；个人页增加 pending-saves，发现服务器已移出列表的未确认操作。自评恢复重读队列，不跳过新队列首卡，队列读取失败与已确认写入分开处理。
- pnpm test:web 五项协议测试、类型检查、Web/SSR build 通过；新增恢复目标/字段校验、掉出队列条目发现、owner 隔离及认证/CSRF/限流保留请求覆盖。尚未完成真实浏览器端到端故障流程验收。

## 2026-10-06：学习页浏览器故障与两标签页复测

- 独立 PostgreSQL brioche_browser_qa、API 3003、Web 5175、浏览器 brioche-recovery；真实登录后选择单选、填写 une，刷新后均保留草稿。
- fetch 故障注入在服务器真实 200 返回后丢弃响应；首次写入 exercise_attempts=1，刷新恢复保留相同幂等键，重试后仍为 1。刷新触发 beforeunload，CLI reload 等待超时，但接受确认后浏览器已实际重新加载，以注入函数消失和重新取得服务器尝试记录验证；未重新启动浏览器掩盖状态。
- 两标签页先后提交不同答案暴露旧 bug：409 拉取新进度覆盖本页未提交的 une。修复后有效草稿保留并明确提示已有新提交；再确认尝试数从 7 到 8，刷新显示实际保存的 une。
- 排序题服务器 200 后丢失响应，SPA 返回首页再续学，原请求恢复；重试后总尝试数仍为 9，pending/answer 存储均清空，显示实际结果。确认原请求的通知独立于 attempt ID 变化，避免已在加载时读取到结果的页面仍停在编辑草稿。
- 浏览器回退/冲突复测部分通过 DOM click/requestSubmit 触发实际 React 表单事件，未以脚本直发判分替代页面；CLI 坐标点击在长页上不稳定，不能据此宣称真实 iPhone 触控验收通过。复习/收藏端到端故障恢复与其他窗口/设备仍待逐项验收。
- 五项 Web 协议测试、TS 7 检查、Web/SSR build 与 workspace clippy 通过。

## 2026-10-06：请求观测与日志保留限制

- 请求观测统一包围合并后的 API；响应生成新的 X-Request-Id，忽略客户端 ID。单元测试检查 400/404/413、唯一 ID、route template、duration_ms，并确认路径参数/query/body/cookie/Authorization/外部 ID 不出现在捕获日志中。
- workspace clippy 与 20 项契约/服务端单元测试通过。数据库集成测试本轮未重跑：没有修改数据库、权限或写入规则。实际 API health 返回 200 与 32 位 hex ID，客户端 ID 未沿用，开发 Web 首页 200。
- Docker local logging 插件存在；Compose config 确认五个服务均有 10m × 3 日志限制。未实际部署或进行磁盘轮换/告警演练。

## 2026-10-06：作者 JSON 文件边界与错误定位

- 课程、素材与 release 文件统一限量读取；不再依赖读完整文件后检查大小或仅检查 metadata。所有嵌套对象拒绝重复成员，包括 Unicode 转义后相同的键，并报告转义后的 JSON Pointer 与原文件行列。
- 五项新增测试覆盖数组内重复字段、Pointer 转义、Unicode 别名、独立对象同名合法、无效 UTF-8、尾随文档、递归限制、数值类型、实际超大文件、文件名、清单类型错误的字段路径/原行列和三个现有示例可读取。
- workspace 的 25 项契约/服务端单元测试和 all-targets clippy 通过；数据库集成测试按默认规则忽略，本轮未修改事务或数据库。完整课程 Schema、语义错误的原文件定位与数据库无关的检查命令仍待补齐。

## 2026-10-06：离线作者检查命令

- 新增 check 与 check-release，在数据库连接和运行模式检查前执行纯文件校验。CLI 子进程测试显式传入不可用数据库与 production/fixture 组合，验证示例草稿检查成功且明确提示发布仍需审校/媒体/目录检查，不输出私有答案。
- 子进程验证拒绝无效私有规则、重复素材引用、错误课程 revision 类型、重复 JSON、错误清单版本和多余参数。素材引用结构/ID/revision 校验由离线检查与数据库 hydration 共用，公共投影补充字段路径。
- workspace 25 项单元测试与 3 项 CLI 集成测试、all-targets clippy 通过；四项 PostgreSQL 测试默认忽略，本轮未执行。CI 增加两份作者示例检查；尚未验证本次远程 CI 结果。离线检查不会验证实际媒体文件、数据库 revision、授权或人工审校，完整源 Schema 与语义行列诊断仍待补齐。

## 2026-10-06：审校元数据与生成的作者 Schema

- 课程投影前校验必需 editorial 的严格状态/说明/未知字段，stage 使用相同类型化状态；不再只比较任意 JSON 的 status 字符串。draft/reviewed 仍是作者声明，未将示例改为真实已审校内容。
- pnpm contracts 从实际 Rust 类型生成 docs/generated/author-lesson.schema.json，包含公共结构、私有规则、审校、可选非 null 素材引用；CI 增加生成文件漂移检查。Rust 测试确认必需字段、私有 Schema 与公共契约隔离。独立 Python Draft202012Validator 验证 Schema 有效、当前草稿通过、非法审校状态/未知审校字段/null 素材引用拒绝；Python 使用本机既有验证依赖，并非新增 CI 依赖。
- 27 项契约/服务端单元测试、3 项 CLI 测试、all-targets clippy、契约导出通过。独立临时 PostgreSQL brioche_author_qa（loopback 55437）通过两项学习/release 集成测试，覆盖固定版本与事务、原子切换、回滚、撤回；临时容器已清理。身份/基础迁移专门测试本轮未单独重跑。
- Schema 仍不取代 Rust 语义校验、实际媒体授权和人工审校；完整语义错误定位、staging 预览与录音继续待实现。

## 2026-10-06：管理员固定版本预览

- 新增私有 operator 课程/媒体路由及 /author-preview SSR 页面，个人页仅管理员显示入口；读取已导入固定 revision，按步骤展示正文、教学块和只读题面，不接入学习提交。返回公共 DTO，替换为课程版本范围内的私有素材 URL；没有返回 serverOnly、editorial 或答案键。
- 独立 PostgreSQL brioche_preview_qa（loopback 55437）两项学习/release 集成测试通过，新增验证匿名 401、learner 403、operator draft 200、私有素材权限/200、未引用媒体 404、缺课程 404、无效 revision 400、降权同会话 403、撤回正文与素材 410、预览不增加学习会话。现有公开素材、切换/回滚/硬撤回校验保持通过。
- workspace 27 项单元测试、3 项 CLI 测试、all-targets clippy、TS 7、Web/SSR build 和 5 项 Web 协议测试通过。页面真实浏览器交互、窄屏及退出后的前端残留内容仍待专门验收，未宣称 iPhone 验收完成；整批 staging 目录和题目判分预览继续待补齐。

## 2026-10-06：整批 staging 目录预览

- 新增 operator release 读取与生成的 PreviewRelease 公共 DTO，按不可变清单逐项核对 entries/public_document 的 ID、revision、父级及顺序，repeatable-read 快照读取撤回信息；异常或缺失内容不静默省略。只返回目录摘要，未返回完整私有源、审校元数据或答案。
- 隔离 PostgreSQL brioche_release_preview_qa（loopback 55437）两项集成测试通过。新增未激活批次预览、401/403/200/404、清单顺序与版本/名称、私有键不存在、active release/generation 不变、降权拒绝、撤回后目录保留条目并标注的断言；原固定版本预览及私有媒体验证仍通过。
- workspace 27 项单元测试、3 项 CLI 测试与 all-targets clippy 通过；契约生成、TS 7 和 Web/SSR build 通过。同页新增批次表单/目录/固定版本课程链接与统一表单样式；浏览器视觉、键盘、窄屏与真实 iPhone 仍待验收，题目判分预览继续待实现。

## 2026-10-06：管理员题目交互判分

- 新增 operator-only grade POST，复用 GradeRequest/GradeResult 与正式 Grader，固定路径/request revision；只读取版本内容，私有规则不返回。前端复用 ExerciseEditor，预览不提供 draftKey，关闭存储与草稿恢复，仅保留本页题目答案/提示/结果，失败 toast 并保留答案；普通学习页仍传入原有 draftKey。
- 独立 PostgreSQL brioche_preview_grade_qa（loopback 55437）两项学习/release 集成测试通过。新增三题型正确/错误判分、匿名 401、learner 403、缺 CSRF 403、降权 403、撤回 410、revision 不符 400、未知题 404、伪造选项/重复排序 token 400、额外 correct 字段 422、结果仅三个字段及 learning_sessions/exercise_attempts 数量不变的断言。
- workspace 27 项单元测试、3 项 CLI 测试、all-targets clippy、TS 7、Web/SSR build 与 5 项 Web 协议测试通过；现有普通学习事务测试保持通过。真实浏览器答题/刷新/退出/请求失败和 iPhone 仍待下一步验收，不能用数据库测试代替 UI 证据。

## 2026-10-06：管理员预览浏览器验收

- 使用 agent-browser 隔离浏览器 brioche-preview-qa，独立 PostgreSQL brioche_browser_qa（loopback 55437）、数据库 API 3003、Web 5175；通过已有受限 browser_fixture 建立合成协议数据，仅将临时测试账号角色设为 operator。未改真实示例的审校状态或正式数据库。
- 真实登录后打开批次目录并进入指定版本，单选正确、填空错误后改为正确、排序正确均显示服务器实际反馈。长页面坐标点击偶尔未触发；先 scrollintoview，部分重试/提交/排序通过 DOM click/requestSubmit 触发真实 React 事件，未直接调用判分 API 替代页面。
- network route abort 的尝试未得到故障证据，移除拦截后改用页面 fetch 包装仅拒绝 grade 请求。失败后 une 保留、提交重新可用，toast 显示中文；修复此前直接显示浏览器技术错误的行为。恢复原 fetch 后原答案判分成功。标签页仅有 react-router-scroll-positions，无预览草稿；SQL 核对 learning_sessions=0、exercise_attempts=0。
- 刷新清空答案与反馈，退出登录后直接打开预览被拒绝。320/390/768/1440 的 DOM/body 宽度等于 viewport，未发现横向越界元素；查看 390 宽度目录与题目截图（本机 .local/preview-qa），基础 Tab 焦点有 solid outline，reduced-motion 媒体模拟生效。这不是完整键盘/屏幕阅读器/iPhone 实机验收。
- 已关闭隔离浏览器、3003/5175 测试进程及临时 PostgreSQL 容器；用户 3001/5173 开发服务保留。TS 7 与 Web/SSR 构建通过。真实私有图片、跨标签退出与迟到响应等完整浏览器流程仍需继续验收。

## 2026-10-06：录音目标与时间轴契约

- 新增 AudioAsset/AudioTrack/AudioCue/AudioWordRange 及可选课程字段；为空时不序列化，保持已有 public_document 与源投影一致性，不改写已发布 revision。音频 Schema/TS 与作者 Schema随契约导出生成。
- 三项新增测试验证整句/语块/Unicode scalar 单词区间、emoji 偏移以及旧文档无音频字段。十二种负例覆盖错误时长/外部 URL/素材引用/正文引用、缺整句、时序重叠、重复目标、缺 segment、子区间越界、缺语块父区间、非正文 block 与未使用录音。
- workspace 30 项契约/服务端单元测试、3 项 CLI 测试、all-targets clippy、契约生成与 TS 7 通过。没有运行新增 PostgreSQL 音频测试：尚无音频表或导入工具。含录音的发布暂时明确拒绝，实际文件时长/授权/媒体服务及播放器仍待实现，不宣称录音功能完成。

## 2026-10-06：跨标签账号变更与私有录音清理

- 补上根布局的身份同步。成功认证变更发送无账号/token 数据的随机 storage 通知，其他标签停止媒体并重新加载原路径的服务器授权结果；可见/聚焦和可见时 30 秒检查账号及角色，网络失败保留下一次检查。组件卸载取消请求和订阅，旧响应不能重新加载新页面。
- 新增五项协议测试验证同账号不刷新、切换/退出/降权刷新、通知幂等与中断在途请求、隐藏标签不轮询、传输失败重试、不并发探测、卸载后迟到响应无效以及 pagehide/BFCache 行为。Web 共 15 项测试、TS 7 类型检查和客户端/SSR 构建通过。
- agent-browser 独立会话 brioche-identity-qa、命名 loopback PostgreSQL brioche_browser_qa（55439）、API 3003、Web 5175 使用原始合成 WAV 协议课程。测试账号仅在临时数据库设为 operator；未改正式内容的人工审校状态。
- 两标签真实登录与退出：预览使用课程范围的私有音频 URL；包装原生 Audio 只记录实际 pause 调用，未替换播放或时钟。播放时观察 currentTime=0.103217、paused=false，另一标签退出后记录 currentTime=0.54851、paused=false 时调用原生 pause，随后来源清空/paused=true。原标签私有正文移除，重新加载显示“请先登录”。
- 第一次退出按钮在屏幕外，点击未生效，不记为通过；滚动到实际按钮后复测成功。一次 CLI URL 等待观察超时，随后读取同一浏览器确认实际已登录，没有据此重启进程。已关闭测试浏览器/3003/5175/55439 服务和专用容器。真实 iPhone、屏幕阅读器、服务器降权的浏览器流程和账号学习录音仍待验收。

## 2026-10-06：账号学习与私有录音版本/权限验收

- agent-browser 会话 brioche-learning-audio-qa，独立命名 PostgreSQL brioche_browser_qa（loopback 55439）、API 3003 与 Web 5175，复用原始合成 WAV。页面登录、开始学习和继续均触发真实 UI 与数据库 API；没有直接请求学习提交接口替代页面。
- 点 Bonjour 的原生媒体从标注区间播放至 0.612659 秒后暂停；头像整句播放至 1.004249 秒并展开“您好！”；全文至 8.000478 秒停止，三者共用一个实际 Audio 元素。仅包装构造/原生 pause 读取实际时钟，不模拟播放。SQL 此时 learning_sessions.version=2、last_step_id=step-read、只有 step-discover 确认、attempts=0，朗读没有创建虚假确认/答题活动。
- 阅读全文播放中通过“继续”进入理解表达，原生 pause 在 0.648006 秒时调用，来源随后清空且 paused=true；SQL version=3、last_step_id=step-explore、step-read 被正常确认。390×844 DOM 无横向溢出，查看本机 `.local/audio-browser-qa/account-reading.png`，头像/正文/译文排列正常，词与整句同步高亮可见。
- 临时数据库把测试账号升为 operator 以打开私有预览，然后另一个标签聚焦期间服务器将其降为 learner；返回预览标签触发实际身份检查。原生 pause 在 0.732537 秒时调用，来源清空；正文不存在，页面显示“仅内容管理员可以预览”。未发送 storage 通知，覆盖服务器侧角色变化路径。
- 为验证 revision 切换，仅在隔离数据库复制协议课程生成 unpublished revision 2，不伪造课程审校或发布。预览 GET 表单从版本 1 切到 2，旧录音在 0.342283 秒时停止/清来源；新播放 URL 为 `/api/v1/operator/lessons/a1-bakery-buy-breakfast/revisions/2/audio/<sha>.wav`，实际 currentTime=0.11269、paused=false。SQL revision 2 保持 published=false；预览操作没有增加学习步骤/练习记录。
- 键盘从播放器 Shift+F10 打开速度面板，初始焦点为选中的 1× radio；ArrowDown 移至 1.25×，Enter 确认关闭并将焦点返回 playback-line，新播放实际 playbackRate=1.25。再次 Shift+F10、Escape 关闭并返回焦点。只覆盖此调速流程，不代表完整键盘/屏幕阅读器验收。
- 已关闭会话、3003/5175 服务及专用 PostgreSQL 容器，未修改用户开发服务。一次点击“继续”在屏幕外未触发，滚动后成功；一次 SQL 查询使用不存在的聚合字段失败，依据真实表结构改查 step_progress 后取得证据，不把失败命令计入通过。真实 iPhone、有法语声音设备回退、完整背景/BFCache 和可访问性继续待验收。

## 2026-10-06：收藏与复习响应丢失恢复

- 隔离 PostgreSQL brioche_browser_qa（loopback 55439）、API 3003、Web 5175 与 agent-browser brioche-review-qa。使用受限 browser_fixture 的未审校协议课程和测试账号，未创建真实课程审校或正式用户记录。页面打开 bonjour、手动加入复习，经实际接口生成 version=1 的到期卡片。
- 页面 fetch 包装先发送原请求、等待真实成功响应，再丢弃响应并抛 transport 错误；没有伪造保存结果。选择“记住了”后页面进入“确认上次复习”，sessionStorage 保留 cardVersion=1、rating=familiar 与原幂等键；SQL review_attempts=1、review_cards.version=2/stage=1。
- 同一标签刷新已实际完成，页面提示“上次复习保存尚未确认”。卡片虽已不在到期队列，pending 仍恢复。点击“重试保存”发送完全相同的版本/评分/幂等键，页面显示已保存 1 个表达且 pending 清空；SQL 记录仍 1 条、版本仍 2，未重复排程。
- agent-browser 在刷新后多次控制观察超时；doctor 确认 daemon 存活，Chrome 本地 DevTools /json/version 返回 200。未重启/清空测试浏览器；使用同一已确认进程与页面的 CDP 连接读到实际刷新结果，随后以 DOM button.click 触发真实 React 控件和真实网络操作。该回退不是原 CLI 自动化通过证据。未打开任何用户页面或读取 cookies/token。
- 收藏同样注入实际成功后的响应丢失：保存请求为 saved=true/version=0/固定来源 revision=1，页面保留原幂等键；SQL saved_items.saved=true/version=1。从页面头像进入个人页，再进入“未确认保存”，SPA 离页后原收藏操作仍列出。
- 首次确认时响应丢失注入还在生效，原操作再次重放、数据库版本仍 1，页面继续保留待确认项；此尝试不算恢复成功。随后用同源空白 iframe 的未包装原生 fetch 移除故障注入，保留实际接口/cookie/CSRF 请求；点击“确认原提交”重放同一 body，页面显示没有待确认保存、存储项清除，SQL 收藏版本仍 1。实际进入收藏页后显示 1 个 bonjour，390px DOM 无横向溢出。
- 本次只验收上述响应丢失/刷新/SPA 离页路径；两标签版本冲突、会话失效、暂停/恢复与完整键盘/屏幕阅读器/iPhone 仍待补验。没有修改业务代码，因此未重复运行已通过的构建和协议测试。

## 2026-10-06：作者课程原文件语义定位

- 新增 author_json::Document，在严格 JSON 读取/重复字段/深度/2 MiB 检查之后索引原字节位置，最多 100000 个值；索引不替代解析器。JSON pointer 使用 ~0/~1 转义，键经 JSON 解码，数组逐项定位；缺失字段回退最近父值。显示行/列均从 1 开始，列按 Unicode 字符计数，定位依据未投影的原文件。
- 公开 DTO、editorial 与 assetRefs/audioRefs 的 Value 反序列化错误保留 JSON pointer；正文词汇/语法锚点、角色/叙述者、步骤/词汇/语法/复习/完成引用和重复领域 ID 返回路径。已有 flow/audio 的带路径错误连接到相同索引；私有判分不一致目前定位整个 `/serverOnly/grading`，未假称逐条规则定位完成。
- 三项新增索引测试覆盖 CRLF/Unicode 列、转义引号/括号、Unicode 转义 key 与 ~1/~0、嵌套数组、根 primitive/空容器、最近父值、实际索引值与严格 JSON 结果一致，以及 100000 上限。
- 新增 CLI 测试用例分别修改正文锚点、步骤块、复习知识、完成步骤、revision 类型、editorial status、audioRefs revision，并独立由原文本计算行列核对 stderr；另外构造超出录音时长的 cue，确认 `/audioTracks/0/cues/0` 与原容器行列。CLI 使用不可连接的数据库 URL，验证失败发生于离线检查。
- Rust workspace 38 项单元测试、5 项 CLI 测试、fmt 与 all-targets clippy 通过；5 项 PostgreSQL 测试保持显式 ignored，本轮未运行，不算数据库验收。未变更公共 DTO/Schema、Web 或数据库结构。Windows 编译 CLI 时暂时停止持有二进制的本机 API，完成后通过 pnpm dev:api 恢复；完整 release/私有判分规则/部分 block 内语义位置继续补齐。

## 2026-10-06：私有规则与目录语义定位

- Grader 新增 author-only 的详细诊断入口，实际规则校验只实现一次，原 from_source 将错误转换为不透明 GradeError::InvalidContent。定位覆盖缺失/多余规则、kind 不匹配、空反馈、无效正确选项、空 accepted/无效单项、排序数量/未知或重复 token；规则 key 使用 JSON pointer 转义，拒绝重复题目 ID。排序规则检查保持 HashSet 线性查找，不引入逐 token 扫描全表。
- ReleaseManifest 的离线/运行校验同样共用：原 validate 保持 AppError::InvalidInput，author 路径输出 root/level/unit/lesson 的 ID、标题、版本、重复与限量位置。空 release 仍允许；最多 20 个等级、5000 个课程引用及原有标识符/文本/revision 限制保持。check-release 接入原文位置索引，未变更清单序列化、源内容哈希、数据库或公共契约。
- 新增规则单元测试覆盖以上语义路径、~1/~0 key 转义、运行错误不暴露详细原因以及重复题目拒绝；两项目录测试覆盖无效字段、第二个重复课程/单元、限量和空目录兼容。新增 CLI 测试独立计算原文行列，核对正确选项、排序 token、Unicode 空白 accepted、release schema/课程 ID/revision 类型错误；语义私有规则诊断不回显测试答案值，仍在不可连接数据库 URL 下运行。
- workspace 41 项单元测试、6 项 CLI 测试、fmt 与 all-targets clippy 通过。独立 PostgreSQL brioche_author_validation_qa（loopback 55439）实际运行两项 learning 集成测试，账号固定版本/所有权/幂等/完成与 release 原子切换/回滚/撤回通过；其他 PostgreSQL 测试未在本轮重跑。
- 专用容器已删除，Windows CLI 构建后恢复 pnpm dev:api，3001 API 与 5173 Web 健康检查均为 200。部分正文流程仍只有 block 级位置，导入与发布 CLI 运行错误尚未全面映射源文件；作者工具维持部分验收。

## 2026-10-06：正文流程字段定位

- 正文流程语义错误从容器细化到原 JSON 字段：对话/短文的句子与语块 ID、空句子的 segments、角色固定快照的名称/头像、角色版本/语言、步骤类型/标题/块引用、题目 prompt/options/tokens/template，以及完成策略。重复角色、speaker 与其他数组引用指向第二次出现的项；不可达教学块指向其 id。练习采用实际扁平结构，路径不引入 exercise 层。
- 解释目标按正文 block、句子、语块分别检查，以 HashSet 查找明确错误的 blockId/entryId/segmentId；保留原来的有效性要求，不执行作者代码或改动公共结构。
- 新增表驱动测试覆盖 25 种字段/重复项错误，扩充真实 CLI 测试，独立按原 JSON 文本计算行列，覆盖角色快照、解释目标三级引用、步骤、空白文本和题目字段。命令使用不可达数据库地址，证明这些作者检查不依赖数据库。
- Rust workspace 42 项单元测试、6 项 CLI 测试、fmt 与 all-targets clippy 通过；5 项 PostgreSQL 集成测试本轮保持 ignored，没有记为本轮数据库验收。API 构建后恢复，3001 API 与 5173 Web 健康检查均为 200。未修改 DTO/Schema、Web 或数据库结构；导入与发布 CLI 的运行错误位置映射仍未全面完成。

## 2026-10-06：课程导入与目录 staging 原文件诊断

- import/release-stage 在数据库连接前保留原 JSON 位置索引，检查参数、引用/审校类型及目录结构语义；旧 --publish 在连接前拒绝。课程投影、私有规则、未登记视觉/录音 revision、超出数据库范围和重复 revision 接入作者位置；数据库写入异常不回显 SQL，也不把响应错误当作未提交的证明。
- stage_author 与原 stage 共用单一事务实现、发布条件和锁顺序。作者入口定位重复 release id、缺失/撤回课程 revision、未审校课程、目录/固定投影不匹配、私有规则或媒体发布校验失败；原 stage 将详细诊断转换为相同 AppError，不向 HTTP 返回作者信息。媒体内部原因目前归于具体目录课程项，未假称每个文件故障已有独立字段位置。
- 新增离线 preflight 测试使用不可达数据库地址；新增真实 author_runtime CLI PostgreSQL 测试，按原 CRLF 文本独立计算行/Unicode 列，覆盖上述登记、判分、版本、目录和审校故障，损坏素材后拒绝并恢复文件，成功导入/staging、重复批次与撤回拒绝。核对失败后 release/entries/audit 没有新增，成功 staging 仍不激活目录、不改变 generation、不公开课程；仅测试 schema 使用合成 reviewed 标记。CI 已增加该显式 ignored 集成测试命令。
- Rust workspace 42 项单元测试、7 项离线 CLI 测试、fmt 与 all-targets clippy 通过。专用 PostgreSQL 容器 brioche-author-runtime-qa、数据库 brioche_author_runtime_qa 中实际运行新作者 CLI 测试和现有两项 learning 测试，三项均通过，覆盖原有学习与发布事务/回滚/撤回行为；identity/postgres/recording 三项本轮未运行。未更改公共 DTO/Schema、Web 或数据库结构。素材/录音导入包与媒体内部细分诊断、正式教学审校和设备体验仍待完善。

## 2026-10-06：Docker 数据库与媒体实际备份恢复

- 新增 Node 运维工具 `scripts/backup.mjs` 的 backup/verify/restore。custom-format dump 以有界二进制流写入；不可变媒体登记在 dump 后读取，复制所有登记的图片/音频并核对 SHA-256，最后写入完整 manifest。只读/无网络 helper 使用实际数据库容器 image ID；不会复制环境秘密或创建自动调度。输出目录不覆盖，文件/目录有界并检查常规类型、大小、哈希及对象名，不接受路径穿越/重复媒体项。恢复要求相同 PostgreSQL major、新数据库/新媒体卷、操作标签，使用单事务与 no-owner/no-acl，写入 UID 10001 并重新校验哈希，不切换应用或线上目录。流传输也检查源文件被改写，失败保留现场。
- 真实应用演练使用仅 loopback 暴露的 brioche-backup-qa / brioche_browser_qa：运行全部 10 个实际迁移，创建合成账号及记录，保存三个学习步骤、一个练习尝试、收藏、复习加入和自评、登录会话；登记四个测试 SVG 与一个合成 WAV。实际备份 database.dump 70395 字节与五个对象，恢复到新数据库 brioche_restore_qa 和新媒体卷；核对迁移数、active release/generation、练习/复习记录。
- 以恢复副本在隔离 API 3004 运行：备份前 cookie 仍可访问所属账号，重新登录成功；完整 progress、收藏列表、复习历史与之前逐项一致，复习 stage=1/version=2/dueAt 保留，概览续学与周活动一致；恢复音频整文件 200 的实际哈希和 bytes=0-99 的 206/100 字节均通过。只使用协议账号与未审校合成内容，不作教学或生产上线证明。
- 负例实际拒绝已存在数据库、已存在媒体卷、被改动的 dump 和同一备份输出目录，核对拒绝前后目标不存在/原 manifest 不变。另加入 100000 行合成 payload，使 dump 达 2260481 字节，恢复后行数与按 id 聚合的内容哈希一致；发现并修复 pg_restore --list 提前关闭 stdin 的 Windows EOF/pipe buffer 路径，只在工具成功退出后接受目录读取的提前关闭，正式恢复仍校验完整传输。
- `pnpm test:ops` 的两项单元测试及显式 BRIOCHE_BACKUP_DOCKER_TEST=1 的真实 Docker 测试均通过，后者含大流恢复、目标拒绝、损坏校验和“哈希正确但非 PG archive”拒绝，并清理自己创建的容器/卷。CI 已接入显式 Docker 测试。手动演练 API/容器/具名卷已清理，开发 API/Web 保留；私有演练文件只在忽略的 .local 下。Rust/Web/DTO/数据库结构未改，本轮没有重跑它们的完整验收。
- 文档补充实际备份/恢复步骤，并修正 tracker 原先把备份演练写成已配置的过宽表述。本次证明样本能恢复；生产规模与 RPO/RTO、保留/异盘加密副本、监控和外部入口仍待验收，不宣称全部运维完成。

## 2026-10-06：完整 Compose 构建与 HTTP 30075 链路

- 实际执行多阶段 Rust release 与 React Router client/server 构建、pnpm deploy 生产依赖、Docker image export；固定 Rust/Node/Debian/PostgreSQL 的官方 registry manifest index digest，既有 Traefik digest 保留。核对映射与维护方式写入 infra/images.md，同步 PostgreSQL 的 CI/运维测试引用。固定后重新 build/up，缓存与运行均通过；仅 Linux/amd64 验证，不代表全部平台或完全离线/逐字节复现。
- 使用独立 Compose project brioche-compose-qa、QA image tag、专用数据库/媒体卷和协议账号，不使用用户生产数据。初始运行 production/database 模式，10 个迁移完成，migrate exited 0，PostgreSQL/API/Web/Traefik 均 healthy；API 与 migrate 实际使用同一 image ID。API UID 10001、Web UID 1000，媒体卷可写；inspect/config 只有 Traefik 30075:8080 的 host binding，API/Web/PostgreSQL 未映射端口，绑定不指定宿主 IP。
- 经真实 Traefik HTTP 30075 请求 /health、/api/health、/api/ready、空生产目录、首页/个人 SSR 与静态资源均成功。容器 CLI 生成仅测试的私有邀请，不打印 token；跨 Origin accept-invite 403、正确 Origin 注册成功、二次消费 400、账号/SSR 中身份一致、private/no-store、HttpOnly/SameSite=Lax 且当前 HTTP origin 不含 Secure、退出后 me 401、新登录成功均验证。未配置证书/HTTPS/跳转或修改用户外部入口。
- 将此前真实应用备份恢复到新的 brioche_compose_restore 和独立 external 媒体卷，手动通过隔离 override 接入 QA stack；使用实际生产容器与非 root 媒体权限读取，旧会话/新登录、完整学习 progress、收藏/复习历史、版本/到期/周活动/续学、音频整文件 200 与单段 Range 206/100 字节校验通过。SSR 显示恢复账号与服务器实际当前步骤/题目；原 exercise 请求以相同 version/body/key 重放，仍一个尝试且 progress/version 不变。10 个迁移保持，无正式审校内容被发布。
- PostgreSQL 固定 digest 后 `BRIOCHE_BACKUP_DOCKER_TEST=1 pnpm test:ops` 的三项检查通过，含真实大流与拒绝案例；Prettier/Compose config/diff 检查通过。没有改动 Rust/Web/DTO/数据库结构，本轮未重跑其完整单元/浏览器验收。QA stack、数据库与媒体卷清理，开发 3001/5173 保留，镜像留作本机构建缓存；仅 .local 保留私有 QA artifact，未推送 Docker registry。
- 修正部署文档中“媒体/恢复尚未接入”及“容器资源上限已经配置”的旧宽泛表述。基础镜像与 Compose 样本链路现有实际证据；容器容量预算、应用生产镜像记录、异盘/加密副本、主机自启动、外部域名/HTTPS/端口映射与公网验收仍待完成。

## 2026-10-06：A1 首三个单元内容包校验

`docs/content/a1` 新增 11 份原创作者草稿，与原有面包店示例组成 3 单元×4 课的固定 revision 目录。每课包含正文、解释、词汇、语法、三类练习、可选任务与回顾；最后一课以纯短文读取虚构发车时间。11 份新文件逐一运行现有 `check`，目录运行 `check-release`，均通过；原有示例沿用已有校验。

新增 `curriculum` 集成测试（无数据库）：以正式作者投影与 Grader 核对 12 个文件和目录 ID/revision/等级/单元、共享知识定义一致、公共投影移除私有字段，36 题的正确答案均判为正确、合法错误答案均判为错误。已在本机运行通过，现有 `cargo test --workspace` CI 自动包含此测试。

这些证据仅证明作者结构与判分规则一致，不能证明教学内容正确、素材授权或可发布。课程全部保持 draft；未导入数据库、未执行 release-stage/activate，development fixture 未替换。此时新增插图引用尚无资产文件，角色与旧插图授权仍待确认，正式录音尚未制作。逐课人工审校与来源记录见 `content/a1/README.md`；完整 A1/A2、真实 iPhone 和生产公网验收继续待完成。

## 2026-10-06：已登记媒体版本的作者定位与原子性

- 图片、角色、录音登记在原有 content_state 行锁内预先核对整批版本。已登记版本分别报告 `/assets/<index>/revision`、`/characters/<index>/snapshot/revision`，CLI 使用原文件位置索引输出实际行/列，不暴露数据库 SQL。保留数据库唯一约束、不可变登记和原事务/审计边界，没有修改 HTTP 错误或公开 DTO。
- 扩展真实 PostgreSQL 作者 CLI 测试：CRLF 原文件中，第二项分别重复已登记图片、角色、录音，核对精确路径/行列及原因；第一项为合法新版本，失败后媒体/角色/审计计数保持原值。两个并发 CLI 导入同一新录音，仅一个成功；失败者仍定位 `/assets/0/revision`，登记及审计各只增加一次。全部使用专用临时 schema、合成授权信息和测试录音，不作为教学审校或正式素材授权证明。
- 32 项服务端单元测试、9 项作者 CLI、2 项课程包检查、all-targets Clippy、fmt check 通过；独立 PostgreSQL 的作者运行时、迁移/发布/唯一约束、录音不可变/原子性/固定 revision 三个集成套件通过。临时 schema 与文件由测试清理，专用 PostgreSQL 容器及匿名卷清理，开发 API 恢复。
- 文件哈希对象仍在登记事务之前写入，失败可能留下不被引用的对象，不能宣称文件系统同步回滚。基础设施错误、发布内部更细诊断、完整内容/设备/运维验收继续待完成，作者发布工具保持部分验收。

## 2026-10-06：表达删除与历史分页的键盘焦点

- 在旧构建的真实收藏页用键盘展开并取消最后一条收藏，复现 activeElement 退回 BODY。SavedList 现保留条目标题 DOM 引用，在删除后的 layout effect 聚焦下一条、没有下一条时聚焦上一条、清空时聚焦空态说明；空态不进入常规 Tab 顺序。SavedRow/ManagedCard 展开时用稳定 useId 将 aria-controls 关联到实际详情，折叠时不引用不存在的元素。
- 历史与表达列表使用共用 cursor 焦点 hook：初始加载不强制聚焦；同页 cursor 改变后聚焦 h1 并立即滚到标题。该跳转不使用平滑动画。
- 新 Web Docker 构建后通过 Traefik 30075 验证：用真实收藏 API 准备 3 个示例课表达，键盘 Enter 展开/取消中间条目、末尾条目、最后条目，分别核对焦点在下一标题、上一标题、空态说明；清空后下一次 Tab 到复习记录。Saved/Managed 的 aria-controls 都指向实际详情；Managed 展开后 Tab 到暂停复习。320/390px 无横向溢出。
- 为专用恢复数据库增加 22 条合成历史行，仅作分页显示夹具，不用它们证明学习事实或真实复习排程。真实 SSR/API 第一页 20 条、第二页 3 条；用键盘进入更早记录后 h1 获得焦点，再 Tab 到我的表达，浏览器返回后 h1 获得焦点且恢复 20 条。初始文档 activeElement 为 BODY，未自动抢夺焦点。表达列表实际多页数据尚未实测，不能仅凭共享 hook 判为通过。
- TypeScript 7 类型检查、17 项已有 Web 协议测试、本机构建及 Docker 客户端/SSR 构建通过，实际 QA Web 镜像 ID 为 `sha256:2ae6a9d478b327e90ae2bd9f39c0d21e247e6d4dcbcd036f6fcc111807b47c7b`。无新评分/权限/数据库逻辑；专用浏览器、容器和卷清理，开发服务保留。完整屏幕阅读器、错误路径、真实 iPhone 与完整可访问性继续待验收。

## 2026-10-06：构建后 SSR 的恢复重放、原生离页确认与收藏冲突

- 对 243a3a6 的 Web 构建 Docker 镜像，使用 Traefik HTTP 30075、真实 Rust API 和隔离恢复数据库，不经过 Vite。正常暂停把卡片版本 2→3；恢复 PUT 实际 200、版本 3→4 后丢弃响应，标签页保留完整原请求（版本 3、suspended=false、原 idempotencyKey）。刷新后条目显示已恢复但禁止新写入；点击重试实际 200，请求体与原请求完全相同，pending 清除，数据库仍为版本 4/非暂停。
- 使用专用 agent-browser 会话与该 Chrome 的独立 CDP 观察器，捕获真实 `Page.javascriptDialogOpening` 的 beforeunload。CLI 自动确认在本机出现读取超时，改由同一个浏览器目标的原生 `Page.handleJavaScriptDialog` 操作，不移除应用的 beforeunload 监听。另一个待确认请求分别选择取消和接受：两次均观察 opening/closed 与实际选择；取消时 document timeOrigin 不变，接受后 timeOrigin 改变，两次原 sessionStorage 请求内容都完整保留。刷新后显式提交清除 pending。这是桌面 Chromium 原生事件/选择与恢复验收，不能代表 iPhone/Safari、BFCache 或背景行为。
- 同账号两标签取消收藏实测：A 成功取消，B 旧版本 PUT 409、最新状态 GET 200，数据库只更新一次。修复前 B 的收藏列表仍残留未收藏条目；Bookmark 新增读取结果回调，收藏列表在确认 saved=false 时移除条目，toast 说明其他设备取消收藏。构建新 Web 镜像后重测：B 列表条目为 0、toast 存在、pending 为 0，收藏版本 3→4 且 saved=false，没有额外更新。
- 最终 Web 实际镜像 ID 为 `sha256:e3dcc3e7a44ac28f505507f968f234b411cd4a38b5498484d079a68023afe92b`，仅供本机隔离验证，没有发布生产镜像。TypeScript 7 类型检查、17 项 Web 协议测试及 Docker 内客户端/SSR 构建通过；浏览器验证直接使用新构建，测试没有模拟 API 成功或修改评分/进度。专用浏览器、容器和卷清理；真实 iPhone、完整可访问性与其他故障组合继续待验收。

## 2026-10-06：暂停响应丢失、会话失效与登录返回

- 使用当前 Web 源代码、隔离 Compose API/数据库及恢复后的合成账号。暂停 PUT 实际 200 后丢弃响应，页面保留原 cardVersion/suspended/idempotencyKey，禁用新写入并显示重试。SPA 转入个人页及未确认保存页，再次实际提交成功但响应继续丢失；核对两次请求体完全相同。恢复响应后第三次确认清除 pending，数据库版本始终只从 2 变为 3，状态暂停，没有重复更新。
- 恢复 PUT 的响应丢失也在真实 API 验证：版本 3→4、暂停变为 false，浏览器保留原版本 3 的恢复请求。刷新后的恢复重放尝试遭遇 agent-browser 读取超时，原生离页确认/刷新重放没有取得完整证据，继续待验收，未当作通过。后续独立样本再次验证恢复写入 5→6；这些为合成验收数据，没有上线意义。
- 登录返回允许列表补上 `/pending-saves`；原来该页要求登录后落到个人页，现成功登录回到未确认保存。真实页面提交登录后 URL 与标题核对通过。新增两项测试覆盖已有站内路径、未确认保存以及外部 URL、协议相对地址、反斜线、编码地址、嵌套路径与任意其他端点拒绝。
- 会话失效验证：先暂缓身份 GET，在专用恢复数据库中将浏览器会话设为过期；暂停 PUT 实际返回 401，页面保留 1 个原请求并禁用新操作，数据库维持版本 6/非暂停。恢复身份 GET 并触发 focus 后，同一活跃会话最终返回 `/login?next=/library`，私有卡片为 0，学习草稿为 0，数据库仍未变。此测试仅对新注册的 beforeunload 监听做 QA 排除，避免自动化浏览器的离页确认阻塞，不证明原生确认弹窗；服务端会话过期、写入拒绝与身份检查均为真实行为。
- 延迟/超时的自动化调用只在收到具体句柄最终输出后记录证据，未按超时推断页面已失败。TypeScript 7 类型检查、客户端/SSR 构建及 17 项 Web 测试通过；专用会话、5175 与 Docker 资源清理。恢复刷新重放、完整可访问性、真实 iPhone 与背景行为继续待验收。

## 2026-10-06：复习冲突后的读取失败与恢复

- 使用 agent-browser 两个同账号标签页、当前 Web 源代码 5175 和隔离 Compose API/PostgreSQL；数据库与媒体从既有合成验收样本恢复到新目标。没有使用正式账号、生产数据或激活课程草稿。
- 先复现管理页问题：标签 A 暂停卡片，标签 B 用旧版本操作收到 409，再注入最新卡片 GET 失败，B 原来的写操作仍可点击。现管理页在读取失败时禁用暂停/恢复，并提供独立重新读取；持续失败保持禁用，实际 GET 200 后显示最新状态、允许新操作。数据库版本从初始 2，经三次成功暂停/恢复变化至 5，冲突与读取重试没有额外更新。
- 自评页加入相同的旧队列写入保护，并对读取最新队列/下一批的迟到响应保留页面 generation 检查。隔离数据库将样本卡片设为到期（仅用于测试）：两标签读取版本 6，A 成功自评后版本 7，B 旧版本自评冲突且队列 GET 失败。卡片和三个自评按钮均禁用，原 pending 已清除；持续失败保持禁用，恢复网络后点击重新读取显示无到期表达。数据库版本保持 7，复习记录保持 2 条（含恢复样本原有 1 条），没有把失败的自评显示为成功。
- 浏览器故障注入只改变 GET 传输行为，写操作仍请求真实 Rust API；核对实际数据库版本与记录数量。CLI 可见性/选择器造成的未触发点击和等待失败不计为成功证据；最终流程滚动到实际控件后操作并核对结果。
- TypeScript 7 类型检查、Vite 客户端/SSR 构建、现有 15 项 Web 协议测试通过。它们不替代上述浏览器回归；会话失效、暂停/恢复提交响应丢失、完整可访问性及真实 iPhone 仍待验收。专用浏览器、5175 与隔离 Compose 资源清理，开发服务保留。

## 2026-10-06：Compose 资源预算与 HTTP 网关验证

- Compose 入口保持 Traefik，仅映射 `30075:8080`，监听所有宿主机接口；API/Web/PostgreSQL 没有宿主机端口，未配置 TLS、证书或公开 dashboard。HTTPS 由用户外部处理。
- 为 PostgreSQL、迁移、API、Web、Traefik 配置可由环境变量覆盖的 CPU/内存/PID 上限，实际 Docker inspect 核对全部生效。四个常驻容器健康，迁移退出 0；整个演练没有 OOM 或重启。迁移上限不限制镜像构建过程。
- 使用 bcc40ff 应用代码完成 Linux Docker 构建。API 镜像 ID 为 `sha256:c79bbe6ab99dd82f5324cfa7439a0455f0ed458ce35439d927c3fc66d0c190eb`，Web 为 `sha256:b03b7402a160290d4dc952649f2afc72ec35ba0c7e3df4d0722fe4436cf307d5`，仅为本机隔离验证镜像，未发布生产镜像。
- 独立项目 `brioche-resource-qa` 验证空生产目录、Origin 拒绝、一次性邀请、HTTP session cookie、登录/退出、认证 SSR 与静态资源。将已有隔离样本恢复到新数据库和新媒体卷，核对进度、收藏、复习记录/版本、学习概览、续学、提交幂等重放、媒体完整哈希及 206 分段读取。
- 先执行 1200 次请求的短突发，再对恢复后的账号及实际发布课程进行 12 并发、15 秒小样本读取负载。Traefik 最初 128 MiB 上限下采样约 98 MiB，因此默认提高到 256 MiB。最终重复负载 6049 次请求全部成功，p50 27 ms、p95 69 ms、最大 140 ms；实际 Traefik 内存上限 268435456 字节。Web 在这组负载中接近一个 CPU 配额。
- 这不是生产容量结论：没有覆盖长时间、多账号写入、大内容目录或完整音频流量。生产容量、监控告警、异盘加密备份及 RPO/RTO、公网 HTTPS 和真实 iPhone 仍待验收。演练资源按项目范围清理，开发服务保留。

## 2026-10-06：新场景插图与离线素材检查

- 制作初次交谈/城市出行两张 640×470 SVG，复用现有品牌配色与 Camille/Léa 外观；源文件仅在 `docs/content/a1/assets`，没有添加 Web public 路由或公开未登记素材。新清单包含实际哈希、尺寸、替代文本、来源与待确认授权，仍 planned/rightsConfirmed=false；没有导入数据库、伪造授权或激活课程。
- 新增 `asset-check <file> <MIME>`，无需数据库，格式、大小、解码与安全 SVG 白名单复用正式导入函数；只输出实际元数据。作者 CLI 使用不可连接的数据库地址验证两图成功、错误 MIME/不支持 MIME 拒绝且不输出成功 JSON。8 项作者 CLI 测试通过；32 项服务端单元测试、2 项课程包检查和 all-targets Clippy 通过。新课程包测试核对两图清单哈希与尺寸，CI 会自动包含。
- 使用 agent-browser 独立浏览器会话和仅含两图的临时本机页面实际渲染，390px/640px 无横向溢出，两图加载完成；核对 640px 截图的构图，显式尺寸后的浏览器 naturalWidth/naturalHeight 为 640×470。截图仅留 ignored `.local/scene-qa`；这不是正式课程发布或真实 iPhone 验收。
- 检查结束关闭专用浏览器与临时静态服务，开发 API 3001/Web 5173 保留且 health 200。素材授权/登记、法语审校、正式录音和完整 A1/A2 继续待完成。


家与日常内容进展（2026-10-06）：补齐第四单元四份原创草稿（房间对话、早晨短文、在家学习对话、每周习惯短文），扩展目录共 16 课，保留原 12 课试点清单。新增室内场景 SVG 与真实哈希清单，均未登记/未审校。结构与 release 检查、三个 curriculum 测试通过，正式 Grader 覆盖扩展目录 48 题的正确和合法错误答案；不据此证明语言审校、正式录音或课程发布完成。A1 后两个单元与 A2 继续待制作。

### 学习进度冲突后的读取失败（2026-10-06）

- learning-session 原逻辑收到 409 后吞掉最新进度 GET 失败，解除 saving 后可继续用旧 version 写入。现保留同步 stale 标记，阻止 write；readFailed 同时禁用学习提交、提示、步骤移动，显示重新读取进度入口。重新读取失败继续保留草稿和锁定，成功后接受最新进度；卸载后迟到响应不更新状态或触发回调。
- 在独立 Vite 5177 测试页运行实际 React hook 与 privateRequest，Chromium 会话 brioche-learning-conflict 注入响应：初始 version 1 提交 409，最新记录读取失败；保存按钮 disabled，直接调用 write 未增加原有 1 次写请求。再次读取失败时 writes=1/reads=2，仍 locked。恢复读取得到 version 4，下一次写请求携带 version 4，成功返回 version 5，pending 清除。
- 再次冲突后延迟 GET，确认保存中的 blocked/readFailed、答案草稿保留；卸载并释放响应后 root 为空、写请求保持 3 次，调用旧 hook write 不新增请求。浏览器 CLI 对等待 Promise 的 eval 曾超时，随后单独释放同一个已在等待的读取，没有重启测试或将超时视为成功。
- TS 7、现有 17 项 Web 测试、client/SSR build、diff check 通过。测试页和浏览器为本机隔离资源，网络为合成故障注入；不声称真实双标签数据库、完整学习页或 iPhone 验收完成。专用浏览器与端口已清理，开发服务保留。

### 排序语块的键盘焦点（2026-10-06）

- 原排序按钮选中后在词库禁用，移回时句子按钮被移除，两种操作都会丢失可用焦点。提取共用 OrderEditor，账号 ExerciseEditor（含管理员预览）和 demo 练习都使用它。选词后焦点移到后方可用语块，没有后方时回到前方可用语块；全部选完则聚焦刚加入的句子按钮。移回词库时聚焦该语块的可用词库按钮；用 useLayoutEffect 等待 DOM 更新，禁用状态不强行聚焦。
- 当前句子和词库提供 group 名称，法语只标在语块文字上；隐藏 status 提供当前句子的完整法语，保留题目 fieldset 禁用、草稿保存和服务端判分规则。没有新增操作教学提示。
- Chromium 会话 brioche-order-focus 在真实开发页 /practice/a1-bakery-buy-breakfast，用真实 demo 判分走到排序题。Tab 到词库末尾 Je voudrais 后 Enter，焦点回到 s’il vous plaît.；再 Enter 到下一可选 une baguette,，全部选完聚焦句子末项。连续 Enter/Shift+Tab 移回三个语块，各自聚焦对应词库项，句子清空后焦点仍在 Je voudrais。
- 随后仅用键盘拼出 Je voudrais une baguette, s’il vous plaît.，Tab/Enter 确认，经实际 Rust demo API 返回答对了，全部排序按钮 disabled。320px 无横向溢出，两组可访问名称和法语 status 文本已核对；这不是屏幕阅读器播报或真实 iPhone 验收。账号页面的复用接入通过类型与构建检查，账号故障/保存恢复仍按原验收清单继续验证。
- TS 7、17 项 Web 测试、client/SSR build、diff check 通过。专用浏览器已关闭，开发服务保留。

### Compose 运行巡检（2026-10-06）

- 新增 scripts/health-check.mjs / pnpm health:check：按明确 project label 发现容器，只读取 Compose 服务/项目/oneoff 和状态、health、exit、OOM、restart 字段；Docker 命令无 shell、限制输出及超时，stderr 不打印。五个必需服务检查区分一次性 migrate 和长驻健康容器，遗漏/重复/非就绪/OOM 均失败，排除 oneoff=True。
- 入口并行 GET /api/health、/api/ready、/health、首页，核对 200 与有限 JSON status/HTML；不跟随重定向，请求及正文读取有超时，正文最多 512 KiB。不输出地址凭据、HTTP 正文、容器环境变量或健康日志。可选 statfs 检查指定宿主文件系统可用字节；未指定时 disk=null，不能据此声称 Docker 虚拟盘或备份目的地正常。
- pnpm test:ops 共 9 项：8 项通过，原 Docker 备份测试因未显式开启而跳过。新增测试包含参数/凭据拒绝与 CLI exit=2、缺失/重复/不健康/OOM/迁移失败/跨项目、磁盘不足与秘密隔离；真实 Node HTTP 覆盖重定向、错误 JSON/status、超大响应、无响应头与头后停滞正文超时。未把跳过的备份演练计为本轮通过。
- 独立 brioche-health-qa Compose 使用已有测试 API/Web 镜像、新临时 PostgreSQL/媒体卷和 QA HTTP 30076；未更改正式 Compose HTTP 30075。首次巡检 5 服务+4 HTTP+宿主文件系统均 healthy，CLI exit=0；确认项目/服务标签后仅停止该项目 Web，巡检 Web not-ready、/health 和首页 502，exit=1，API health/ready 仍正常；恢复健康后再次 exit=0。该证据仅覆盖运行探测，不是生产数据/业务全链路/正式域名验收。
- 首次组合停止/恢复命令被自动审查拦截且未执行；随后核对实际项目标签，通过明确 Compose project 的 stop/up 完成同一个隔离故障演练。结束以该项目 down --volumes 清理，实际查询容器和项目卷均为空；开发 5173/3001 保留。没有安装定时任务、接入外部告警或发送任何通知。


购物餐饮内容进展（2026-10-06）：补齐第五单元四份原创草稿（水果重量与数量、购物清单短文、餐馆菜单与点餐、成分选择与用餐后结账），五单元目录共 20 课；原 12/16 课清单保持不变。新增水果摊与餐馆桌面两张 SVG，真实哈希/尺寸/安全格式检查通过。四项 curriculum 测试核对三个目录、共享知识、私有字段剥离及全部 60 题的正确/合法错误答案。全部课程仍为 draft，素材 planned/rightsConfirmed=false，无正式录音或发布。A1 最后单元和 A2 仍待制作，人工法语/译文/教学及画面审校继续待完成。

补充：购物餐饮插图通过离线浏览器图像展示检查，320/390/900px 均加载并按比例缩放、无横向溢出；已查看 390px 实际截图。未绕过登记或将图片放入 Web public。curriculum 同时新增跨课程同角色 revision 的固定快照一致性检查，四项测试通过；这仍不替代角色库登记或人工图像审校。


认识与约见内容进展（2026-10-06）：补齐第六单元四份原创草稿（三位角色介绍朋友、咖啡邀约、约见确认短文、拒绝与改期），A1 六个规划单元共 24 课；新增六单元目录，原 12/16/20 课清单保持不变。新增公园场景 SVG 与真实哈希/尺寸清单；五项 curriculum 测试核对四个目录、共享知识、固定角色快照、私有字段剥离及全部 72 题的正确/合法错误答案。所有课程仍为 draft，素材 planned/rightsConfirmed=false，无正式录音或目录激活；不宣称完成官方 A1 等级。A2 与人工法语/译文/教学审校继续待完成。

补充：公园和室内插图在独立离线浏览器页核对 320/390/900px 按比例缩放与无溢出，390px 实际截图已查看。未通过公共媒体路由或 Web public 绕过登记；不替代人物/素材正式审校与真实 iPhone 验收。专用浏览器已关闭，开发服务保留。


### 2026-10-06：媒体发布诊断与失败原子性

- 隔离 PostgreSQL 容器 `brioche-publication-diagnostics-qa`，仅本机 55437；author_runtime 和 recording 各用专用 schema，与生产和开发数据库无关。真实 CLI 的 CRLF release 文件核对课程条目的准确行列；图片损坏/移除后分别得到 `/media/0/sha256` 哈希不匹配与缺失/不可读消息。新导入的测试课程改变角色名（同时更新正文 speaker）或引用未登记角色 revision，分别定位 `/cast/0` 与 `/cast/0/revision`。每次失败均无 release、entries、audit 新增；恢复后 staging 正常且不激活。
- 录音集成测试在正常 staging 前损坏/移除实际 MP3 文件，核对 `/audio/0` 两类消息和三个表不新增；恢复后继续既有登记、发布、授权、撤回、范围与腐坏测试并通过。
- 首次测试尝试 UPDATE 登记描述，被不可变触发器正确拒绝；已改为新课程草稿，无关闭触发器或修改登记版本。首次角色草稿只改 cast 名称被正文一致性校验拒绝，已同步 speaker 后验证真正的发布快照门禁。
- `cargo test -p brioche-server --test author_runtime --test recording -- --ignored`：2 项 PostgreSQL 集成测试通过；`cargo test -p brioche-server --lib --test author_cli`：32 + 9 项通过。诊断不复述底层 SQL、系统路径或私有答案。音频来源/解码元数据等分支保留实际门禁，但本轮没有对每个诊断分支单独做故障注入。
- 临时数据库容器删除（含本次匿名卷），开发 API 恢复；无 production release 或公网配置变更。


### 2026-10-06：真实表达库多页与保存焦点

- 新建隔离 `brioche-library-pagination-qa` PostgreSQL（仅本机 55438），API 3002、React Router dev 5177；原开发 3001/5173 保留。独立原创协议词条加至测试课程，素材与课程的 ready/reviewed 声明只限隔离协议测试，不作为语言审校或正式发布证据。通过正式邀请、收藏、复习加入 API 创建 23 条收藏/卡片，两个列表 GET 的 20+3 共 23 个唯一 ID、无遗漏。
- Chromium 实际页面键盘 Enter 翻页、浏览器返回，核对行数 20→3→20 和标题 H1 焦点。第二页暂停后返回再进入仍显示已暂停，恢复后卡片状态与实际 API 一致。三条第二页收藏依次取消，焦点转到下一条/最终 status；发现原提示误表示全库空，已改为当前页空态和返回收藏列表。重新加入一条再从新代码页面取消，验证 status 焦点与新文案、Tab 到返回入口、Enter 后第一页面仍 20 条且无下一页。收藏取消不移除 23 张复习卡。
- 发现暂停/恢复 native disabled 期间焦点落到 BODY；用 aria-disabled/aria-busy + 点击门禁修复。对实际 fetch 注入 8 秒发送延迟，窗口记录一次 preferences 请求；保存期间重复 Enter 两次仍只有一次请求，按钮焦点保留；保存后仍聚焦“暂停复习”，实际 API 版本 6→7、suspended=false，排除重复更新。未对这项改动重新注入完整 409/401 故障组合，也未声称屏幕阅读器通过。
- 320/390px 第二页复习与收藏第一页面无横向溢出；390px 复习展开和分页空态截图实际查看。`pnpm typecheck`、17 项 `pnpm test:web`、`pnpm build` 通过。专用 browser 关闭、API/Web 监听进程按已核对路径/启动参数停止，新建 PostgreSQL 含匿名卷删除；开发 API health 返回 ok。未改变用户/生产数据。


### 2026-10-06：A2 周末与出行四课草稿

- 四课原创文本/中文译文/解释和练习，正文 140/164/189/171 个空白分隔词。两篇 article 保持纯段落，两个 dialogue 保持 8 轮、固定角色和独立情境身份；三类题各一题。官方语言参考在 A2 README 记录，仅核对结构，不等于人工审校或转载第三方例句。
- `check` 四份源文件和 `check-release` 联合目录通过。六项 curriculum 测试通过：新联合目录两级七单元 28 课、84 题正确/合法错误判分；A1/A2 共用词条和角色 revision 快照一致；DTO 投影剥离私有字段；A2 draft/正文形式/120–250 词检查。A1 四个既有目录与素材库存仍检查通过。
- 本轮没有登记、导入或激活课程，没有浏览器 operator 预览，也没有录音/真实 iPhone 证据。插图复用原 A1 城市 SVG 和清单；素材仍 planned/rightsConfirmed=false，没有重复登记素材或修改正式 revision。说明四课为独立练习场景，日期/报价/班次不是真实运营信息。人工法语/译文/难度/语法支持说明与插图适配仍须审校，A2 其他五个单元待制作。


### 2026-10-06：学习双标签 CAS、完成与焦点恢复

- 新建隔离 `brioche-learning-tabs-qa` PostgreSQL（本机 55439）、API 3002、React Router 5177；只使用合成协议账号/已登记测试素材与测试课程。同一 Chromium 的两个独立标签加载同一实际学习会话 version 4，不共享 sessionStorage 草稿。A 正确选项保存为 version 5、1 次尝试；B 原选项和填空 une 草稿继续保留。B 提交 version 4 得到实际 409，仅最新 GET 注入传输失败；重新读取再失败后数据库仍 version 5、1 次尝试、0 提示。选择/填空/排序、确认、提示、步骤切换被禁用。
- 恢复 GET 后读到 version 5，展示题目已有新提交但草稿保留；B 使用 version 5 保存原选择，数据库 version 6、2 次尝试。A 的提示 version 5 得到真实 409→实际 GET 200，填空草稿 une 保留且没有新增提示。明确再确认提示后写 version 6→7，再提交填空 version 7→8，数据库 hint_used=true、判分正确；选择题保留第一正答与第二错误尝试。排序实际键盘组句/提交→version 9、4 次尝试，继续实际步骤 practice/apply/recap→12，complete→13，自动复习卡保持 3。这里验证“完成与掌握分开”，未把最新错误尝试强行改成正答。
- B 旧 version 6 提交后真实 409→GET 完成状态，页面不再显示学习写按钮，数据库仍 version 13、4 次尝试、1 提示、3 张复习卡。发现正常完成和旧标签同步完成会让焦点落 BODY，新增完成确认状态 effect；新 repeat 会话两个标签同读 version 10，一处完成到 11，另一处 complete version 10→409→GET，双方焦点均为 H2“本课已完成”，没有额外尝试/复习卡。repeat 准备脚本首次未确认前置步骤就提交被 409 正确拒绝，修正为正式前置步骤顺序后继续；未绕过门禁。
- 第三个会话实际 complete 已提交 version 10→11 后注入响应丢失；刷新后 SSR 已完成但客户端恢复“确认上次保存”。重试前后比较原 path/method/完整 JSON body（包含相同幂等键，但不输出键），匹配=true、实际 replay 200、pending 清除、焦点 H2；数据库 version 11、3 次尝试、复习卡总数 3 不变。该项测试有意移除当前任务捕获的 beforeunload listener，仅排除原生弹窗，未宣称原生确认或 iPhone 行为；旧的桌面原生确认已有独立记录。
- 390px 完成页截图已查看，320/390px 无横向溢出；`pnpm typecheck`、17 项 `pnpm test:web`、`pnpm build` 通过。专用两标签浏览器关闭，QA API/Web 监听进程按已验证路径/参数停止，临时 PostgreSQL 含匿名卷删除，开发 API health=ok；未修改生产数据、用户入口或正式课程。完整屏幕阅读器、iPhone、所有学习故障组合仍待验收。


A2 一起生活进展（2026-10-06）：新增分配家务、共同空间规则、生活习惯、比较住处四课原创作者草稿，两段八轮对话与两篇六段短文，正文分别 181/156/158/208 个空白分隔词。复用 art-home-morning revision 1 与 Camille/Luc 固定快照；保留原 A1 与 A2 pilot 目录，新增联合目录包含 32 课、96 道题。七项 curriculum 测试通过，核对两级目录、共享知识/角色一致、私有字段剥离、96 道题正确及合法错误答案和八课 A2 长度/形式；四个新单课及新 release 的 CLI 结构检查、cargo fmt 与 diff 检查通过。课程仍 draft，素材仍 planned/未确认授权，未访问数据库、导入或发布课程；其余四个 A2 单元、逐课预览、人工内容审校、真实设备与生产验收仍待完成。


A2 日常事务进展（2026-10-06）：新增预约服务、填写并核对信息、解释借阅卡问题、询问后续处理方式四课原创作者草稿，三段八轮对话与一篇六段说明，正文分别 186/162/201/206 个空白分隔词。复用城市场景与 Camille/Luc revision 1；保留之前目录，新增三单元 A2 联合目录，共 36 课、108 道题。八项 curriculum 测试通过，包括固定顺序、跨级共享知识/角色、公开投影、正式 Grader 的正确及合法错误答案；新四课/new release CLI 校验和格式检查通过。表格为阅读说明，不收集真实个人信息；流程、日期和时限为虚构设定，收件确认与最终答复区分。课程保持 draft，未导入或发布，素材/人工审校/正式录音继续待完成；A2 另三个单元及设备/生产验收仍待推进。


学习撤回恢复进展（2026-10-06）：真实隔离 PostgreSQL/API/学习页复现硬撤回后 POST 410 只显示错误却保留练习和操作。现把已收到的 410（以及记录不可用的 404）设为不可继续状态；同步阻止写入/重读/原请求重试，停止播放器、收起已渲染课程内容和操作，聚焦状态标题，只清理该用户/该会话/该 revision 的步骤、答案与 pending，其他会话/版本/用户草稿保留。没有声称服务器撤回后无请求就实时推送。

隔离实测：正式 CLI 撤回后实际 POST 410；另一设备实际提示把 version 4→5，旧 POST 409 后延迟 GET，正式撤回后放行实际 GET 410；另一个旧 POST 409 的第一次 GET 注入传输失败，保留/锁定选择与填空，随后正式撤回、手动重读实际 GET 410。三条路径均收起可见正文、零学习操作按钮、H1 焦点、本会话草稿为零；直接撤回路径还核对其他 scope 草稿保留及 speechSynthesis.cancel 调用一次（不声称该设备真实法语发声）。数据库确认撤回路径没有新增尝试/完成/额外提示；两个冲突场景只保留另一设备显式保存的 1 条提示/version 5，直接撤回仍 version 4/零提示。正常未撤回样本补验旧提交 409→GET 200，保留填空草稿并按 version 5 再提交到 6，只有 1 次正确尝试。17 项 Web 测试（增加限定 scope 清理断言）、TS 7、SSR/client build、diff 检查通过；320/390px 无溢出，390px 截图已查看。新 404 分支沿用同一不可用处理，但本轮没有人为制造真实 404；完整屏幕阅读器/iPhone 和其他故障组合仍待验证。


A2 身体与状态进展（2026-10-06）：新增描述不适、预约就诊、表达情绪、读接待指引四份原创草稿，两段八轮对话/两篇六段短文，正文 206/209/175/173 个空白分隔词。联合目录 A1 24 + A2 16 课、120 道题，原目录保留。身体位置/持续时间、礼貌请求、状态形容词/原因、肯否定指令有独立练习；不推断疾病或严重度，不给出诊断、治疗、分诊或等待时限判断，任务使用虚构人物。新增原创接待室 SVG（640×470、2497 字节，SHA-256 bc9f8b9fc00c17faade6b0682ea5c371940ebe7165471325c1d0f789202e60fe），A2 独立素材清单保持 planned/rightsConfirmed=false；预约/接待课使用新图，朋友交谈/情绪短文复用室内图。十项 curriculum 测试含跨级知识/角色/投影/120 题正确与合法错误判分、16 课 A2 正文形式长度、两份素材清单真实哈希尺寸；四单课、新 release、asset-check 和格式/diff 检查通过。离线浏览器插图 320/390/900px 按比例缩放无溢出，390px 截图已查看，专用浏览器已关闭。未访问数据库或导入/激活，全部仍 draft；operator 逐课预览、人工语言/译文/教学/画面审校和正式录音待完成。A2 工作学习与经历、表达与协商两个单元及完整设备/生产验收继续待推进。


A2 工作学习与经历进展（2026-10-06）：新增简述实践经历、安排协作、说明进度、讲述昨天四课原创作者草稿，一段八轮对话与三篇短文，正文分别 166/210/171/191 个空白分隔词。复用固定 Camille/Luc 与室内图，保留之前目录，新增五单元 A2 联合目录，共 44 课、132 道题。十一项 curriculum 测试通过，核对两级目录、知识/角色一致、公开投影、所有正确及合法错误判分、20 课 A2 正文长度/形式和素材清单；四课、新 release CLI 校验、fmt/diff 检查通过。重点区分已完成/尚未完成/计划，si 条件与间接疑问，以及本课非代动词 être 过去分词主语配合；未完成过去时支持表达仍待难度审校。没有导入、发布或声称内容审校完成，全部 draft/无正式录音；A2 最后单元、逐课预览、人工审校和设备/生产验收继续待完成。


A2 表达与协商进展（2026-10-06）：新增评价体验、说明偏好、提出替代方案、澄清误会四课原创草稿，一篇六段短文与三段六轮对话，正文 179/168/166/171 个空白分隔词。保留之前全部目录，新增 catalog.full.release.json（a1-a2-full-draft-v1），A1/A2 各六单元 24 课，共 48 课、144 道题。复用固定 Camille/Luc 与室内图；个人看法不冒充客观品质，偏好不评判他人，建议经过双方确认才成为约定，澄清分别核对日期/时刻/入口。十二项 curriculum 测试通过，覆盖跨级目录、知识/角色一致、公开投影、144 题正确与合法错误判分、24 课 A2 正文形式长度和素材清单；四课与新 release CLI check、fmt/diff 检查通过。没有数据库导入或发布，全部仍 draft；语言、译文、教学难度、operator 逐课预览、正式素材/录音、辅助技术/iPhone 和生产验收继续待完成。
