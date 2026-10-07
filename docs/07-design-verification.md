# 设计验证记录

## Chef 共享契约真实抽取（2026-10-08）

Chef `2d96053de488eb0255fe75e8a63d16863a537e86` 实际包含原 Rust 契约与生成器，法语兼容样本仅作 test fixture。独立20项契约测试、Clippy及重新生成TS/Schema无差异通过，远端CI `37650258408` completed/success。暂保留原内部包名与v1法语字段，不代表粤语或独立身份服务完成。

Brioche `4c657c9` 删除本地契约crate，改用固定 `framework` Git子模块的path依赖；外层workspace明确exclude framework，避免Cargo自动把框架加入产品workspace。最终 `pnpm contracts` 从框架生成产品快照/作者Schema，git diff无差异；Brioche完整常规Rust工作区测试与Clippy、TypeScript检查和28项Web单元通过。数据库ignored测试本轮未执行，无SQL/数据库语义变更。初始化后的框架子模块独立20项测试及两workspace fmt检查也通过。

从GitHub在全新私有目录clone --recurse-submodules，实际检出上述固定Chef提交，Cargo metadata成功，不依赖开发机相邻Chef目录。产品CI已配置递归checkout和框架测试，保留原生成物漂移检查；新产品CI `37651002731` 当次仍in_progress，不提前称通过。

最终workspace exclude版本的Docker API实际构建成功，三覆盖up --wait --no-deps server成功。运行镜像`sha256:a0d8500d8eb59bf25487a8d79dc0f761a80c8216931fa4de6e54fc350edbe9ab`，healthy/0restart；真实release-status仍为48课正式录音目录、generation10。五服务/四HTTP健康检查通过。没有账号/课程/媒体迁移或付费合成，不改变既有公开契约，HTTPS和TTS覆盖保留。

## 完成提交的离页恢复与远端完成（2026-10-07）

隔离 Chromium 浏览器新增完成课程故障组合：先成功确认必需步骤，完成请求返回 503 后经原生离页保护离开，再返回并重放保存的请求。实际核对路径、POST 方法、全部请求字段及幂等键完全相同；收到完成回执后移除待确认草稿，焦点落到「本课已完成」。另一组合让完成请求返回 409，读取另一处已完成的进度，核对不再发送新的完成请求且保留完成页面。新增测试通过（1 项，50.94 秒）。

随后与步骤/完成原请求重试、多步骤恢复、冲突重读及撤回、等待中的离页焦点组合共同复跑，5 项浏览器测试全部通过（116.03 秒），28 项 Web 单元测试通过。没有运行完整浏览器套件，也没有改动运行时代码或重新部署。

这使用受控 API 模拟失联和远端完成，不是生产账号写入或真实两个设备并发；既有 PostgreSQL 幂等与版本事务测试仍提供服务端证据。真实 iPhone、完整故障组合与完整目标验收继续待完成。

## 管理弹窗名称与渲染后焦点（2026-10-07）

实际发现课程操作、账号权限/邀请/重置、会话、链接、角色声音、配音片段、试听、音色核对、参考录音撤销及试听离页确认弹窗虽有可见标题，却缺少弹窗名称关联。十类弹窗现在以useId生成实例标题ID并通过aria-labelledby关联，名称随当前操作变化；不将整个表单或一次性链接用作名称。

浏览器回归先复现名称缺失，补齐后进一步复现课程导入焦点丢失：showModal在点击处理器中运行，React随后按fileSession替换文件输入框，原生初始焦点落在被卸载的旧字段。九个管理路由改用useCommittedDialog，在同次React状态提交后的layout effect打开；后续输入、异步状态更新或关闭不重新打开，也不更改写入参数、重试或权限检查。定向原生键盘验证批准弹窗Escape返回原入口、Enter再次打开，以及目录创建→课程导入→邀请/重置/角色→会话操作后名称和初始焦点正确。

验收从已构建的真实SSR页面操作，核对实际dialog:modal、当前标题、弹窗内焦点及Chromium原生无障碍树的dialog名称；快照结构节点不在交互refs中，按实际snapshot树读取。受控API保留原精确重试、取消和审校声明测试，不操作生产账号或调用收费TTS。该证据不等于真实屏幕阅读器/VoiceOver或iPhone Safari验收。

最终版本通过TS7类型检查、SSR/client构建、28项Web单元、29项SSR和整套22项真实SSR浏览器回归（385.91秒），新增名称/初始焦点检查与键盘Escape/Enter往返均成功。初次检测复现名称缺失；标题修复后的快照refs断言不适用于结构节点，改读实际无障碍树后又定位真实导入焦点问题，修复并完整复验。测试不是屏幕阅读器、实际iPhone或生产管理员账号操作。

生产仅更新Web，HTTPS+TTS三覆盖实际构建/启动，镜像`sha256:dda9c5aaf7137e251d164d8dd32c1caaceef9580b21adc31628b9450a8b411e8`，API保留目录预检镜像；healthy/0restart/noOOM，五服务/四HTTP健康检查通过。两个HTTPS入口48课目录、首页/课程SSR与新版dialog模块实际200；匿名后台401且private/no-store。无迁移、生产账号或内容写入、收费外呼，未恢复HTTP端口。前序源码ee14266及文档54db507完整CI均实际completed/success，本批提交CI另行核对。

## 2026-10-06：本地整套 release 课源离线检查

- check-release 原本只检查清单。新增真实 CLI 回归先因 --sources 不受支持失败；现可显式传 1–20 个来源文件/目录，逐引用复用单课检查，再核对 ID/revision/等级/单元。目录仅查 `<lessonId>.lesson.json` 且解析后不得越出该目录，别名例课可直接传文件；不递归发现无关 JSON。同一实际文件去重，不同同名课源失败，显式课源未被清单引用也失败。
- 不可连接数据库条件下，最新 A1/A2 联合目录全部 48 课检查通过；缺失/歧义源定位清单 lessonId，课源身份/版本/等级/单元、选项类型、私有规则错误定位原源字段行列，覆盖中文/CRLF。全部通过才输出课数，报告继续说明登记媒体、人工审校与 staging 仍需要完成；不返回私有答案。
- 37 项 server 单元、23 项作者 CLI、13 项 curriculum（48 课/144 题正确与合法错误判分）、Clippy -D warnings、cargo fmt/diff 通过。公共 DTO/课源/数据库/部署未变。这是离线作者流程补齐，不替代数据库 release-stage 或完整内容验收。

## 2026-10-06：可选填空提示与键盘焦点

- 课程允许 hintZh 为空或只有 Unicode 空白，但实际 ExerciseEditor 和 demo practice 仍显示提示入口。新增真实组件 Chromium 回归先复现 NBSP/窄 NBSP 提示仍有按钮；现两种练习界面按 trim 后是否非空展示入口。有内容的提示仍可展开；键盘回归再复现按钮消失后焦点落 BODY，现仅在用户主动请求提示后将焦点移至展开内容，服务端预先记录的提示不会在初始渲染时抢焦点。
- 独立 PostgreSQL 课程使用合法空白提示，明确确认前置必做步骤后直接请求提示。首次测试错误地把 LearningSession 当 LearningState 读取，修正 progress 包装后实际复现 HTTP 200（期望 404）。现空提示与无此提示统一 NotFound，重复请求不改变进度、不创建 exercise_hints，随后合法作答 hintUsed=false；正常提示/重放/版本/跨账号/完成仍通过现有完整学习集成测试。
- 20 项独立 Chromium 回归全部通过（约 255 秒）、27 项 Web 协议、6 项 SSR、TypeScript 7、客户端/SSR 构建、两项真实 PostgreSQL 学习集成测试、Clippy -D warnings、格式检查通过。浏览器使用合成请求与独立会话，数据库使用专用临时容器/独立 schema，容器与其匿名卷已按精确身份清理；常用服务和用户浏览器未改动。真实 iPhone、辅助技术、教学内容及生产门槛继续待验。

## 2026-10-06：课程块嵌套类型错误定位

- 扩展真实作者 CLI 回归先复现对话 turns/segments/text 类型为整数时仅报告 /blocks/1 及整个块位置。serde_path_to_error 包住 internally tagged enum 仍丢失内部路径，因此以同一字段宏生成严格 wire struct，显式拆 type/exerciseType 后解析。普通 Block Deserialize 与作者带前缀解析共用实现；未知字段继续拒绝，public DTO 仍保持原平铺形式。
- check/import 在数据库前分别验证对话语块 text、角色 displayName、choice options/text 和 order tokens/text 的准确字段行列，覆盖 CRLF/中文源。所有块未知字段回归改为要求字段值位置；嵌套未知键中的 / 和 ~ 通过 JSON Pointer 转义单元测试。各示例块往返保持相同 JSON，重新生成公共 TS、公共/作者 Schema 无差异。
- 19 项公共契约、37 项 server 单元、21 项作者 CLI、13 项 curriculum 通过（48 课/144 题），Clippy -D warnings 与格式检查通过。未修改课程源、数据库或常用服务；缺失字段依然按最近存在父节点定位，其他完整语义、人工内容、设备与生产门槛未因此宣称完成。上一提交 1aca7bb 的 CI job 112176811052 已观测 completed/success；新提交另行触发 CI。

## 2026-10-06：练习步骤与完成策略一致性

- 现有设计要求练习只进入 practice、必做题属于必做 practice，但原 validate 只验证引用存在与块可达。新增公共契约回归实际复现把 practice 改为 read 仍通过；修复后按原步骤 blockIds 项拒绝。移除 practice 的必做身份但保留必做题，按 completion.requiredExerciseIds 项拒绝；可选 practice/可选题和同题在其他 practice 再次回顾仍合法。
- 离线 check 和 import 的数据库前预检共用该固有语义检查；不提前验证登记媒体占位描述，实际导入后仍完整验证。CLI 使用不可连接数据库、CRLF 和原中文课程源，证明两种错误在访问数据库前准确报告原字段行列；公开 DTO、运行提交/确认 API 和课源未修改。
- 18 项公共契约、37 项 server 单元、21 项作者 CLI、13 项 curriculum 通过（48 课/144 题结构和正确/合法错误判分），Clippy -D warnings、cargo fmt 与 diff 检查通过。这是作者数据约束与离线验证，不替代 PostgreSQL 全链路、人工教学审校、设备或生产验收。

## 2026-10-06：选择题等价重复选项校验

- 新增公共契约回归在修复前失败：不同 ID 的相同显示文字通过 validate。现在选择题经 NFC、空白和法语撇号归一化后的文字必须唯一；大小写和重音保留，排序重复语块继续合法。归一化从 Grader 移至公共 Rust 契约，原 grading::normalize_text 入口保留重导出，公开 DTO/答案字段未改变。
- check 与 import 预检共用此项固有语义校验，不提前要求作者占位媒体满足登记描述；完整导入仍在素材 hydration 后验证全课程。CLI 回归覆盖原样重复、NBSP/多空白、组合重音与弯撇号，使用不可连接数据库证明在访问前失败；CRLF/中文文件按原第二个选项 text 精确报告行列。第一次 CLI 测试借用的 helper 使用 fixture+production，import 因模式限制失败；改为 database 模式后再验证实际语义诊断。
- 17 项公共契约、37 项 server 单元、20 项作者 CLI、13 项 curriculum 通过，含最新 48 课/144 题结构及正确/合法错误判分。课程源、审校状态、登记媒体与部署未变；这不替代人工教学审校、设备或生产验收。

## 2026-10-06：排序题的等价重复语块

- 真实 Grader/公开投影的合成排序题为 la porte de la maison，两个 la 使用不同稳定 ID。新增回归在修复前失败：交换两个 la 后输出文字相同，但 result.correct=false。前端 OrderEditor 显示文字，内部 ID 仅用于选择/去重，学习者无法区分此种隐藏身份。
- 保留原 ID 集合、个数、重复/遗漏/未知检查；目标 correctTokenIds 仍在作者校验中要求每个原 token 恰好一次。实际判分将每个 ID 映射为 NFC/空白/撇号归一化文字，保持大小写与重音，逐位置比较提交/目标向量，不拼接文本或接受客户端分数。既有 private rules/公共 DTO 不变；异常目标引用保留 InvalidContent，异常输入保留 InvalidAnswer。
- 修复后原顺序与重复 la 互换均正确，porte/de 互换仍错误，重复同一 ID、未知 ID、缺失语块仍 InvalidAnswer。组合重音/NFC、空白与弯/直撇号等价通过；去掉重音和改变大小写不等价。37 项 server 单元、19 项作者 CLI、13 项 curriculum 通过（含最新 48 课/144 题正确与合法错误判分），Clippy -D warnings、cargo fmt 和 diff 检查通过。未新增数据库写入、作者源变更或课程发布；此为判分实现与离线课包验证，不代替真实账号新场景的数据库/浏览器、设备或人工教学审校。

## 2026-10-06：作者答案与网页填空的共享边界

- 原作者 accepted 与运行判分仅限制 4096 UTF-8 字节，实际 ExerciseEditor、demo practice、validAnswer 则限制 1024 UTF-16 单位。两项新增 server 回归在修复前失败：1025 ASCII/513 个非 BMP 字符的 accepted 未被拒绝，运行输入也被当成普通错误答案。首次 Rust 测试断言错误地要求 GradeResult 实现 PartialEq，改为比较 GradeError 后才计入上述失败证据。
- Rust 公共契约定义并生成 answer-limits.ts，两个输入框和草稿读取导入同一常量，服务端对实际输入按 encode_utf16 和字节上限检查。作者规则提前校验非空归一化答案、原 4096 字节上限与 canonical 表示的 UTF-16 上限；canonical 保留大小写，避免 İ 等 lowercase 扩展错误地使可输入答案被拒绝。NFC 可把组合重音转成可输入的等价表示，空白/撇号归一化规则不变，不移除法语重音。错误仍为作者字段诊断，HTTP 继续返回原不透明 GradeError。
- 单元测试覆盖 ASCII/é 1024、非 BMP 512、组合重音的 NFC 表示、大小写扩展、带多余空白但短 canonical 的答案，以及过界运行输入与过界作者答案。实际 check/import 子进程在不可达 DATABASE_URL 下拒绝三种超界源，中文/CRLF 原文件 accepted/0 值行列正确、stdout 为空且无数据库连接错误。19 项作者 CLI、35 项 server 单元、16 项契约、13 项 curriculum（48 课/144 题正确/合法错误判分）通过，Clippy 无警告。
- 27 项 Web、TS 7、client/SSR build、6 项 SSR 和格式检查通过。实际 ExerciseEditor 的 Chromium 原生 keyboard inserttext 将 1025 ASCII 截为 1024、513 非 BMP 截为完整 512，确认提交保留精确输入；另输入 e+组合重音，提交仍为原两单位字符串。定向浏览器 1 项通过（40 秒），完整 19 项 Chromium 回归通过（275 秒），teardown 已关闭专用浏览器与临时 Vite；此证据不代表人工教学审校、真实 iPhone 或正式录音验收，未修改课程状态或发布数据。

## 2026-10-06：课程不可用分支的独立原请求保护

- 实际 Learning/LearningProvider/MemoryRouter 配合受控进度 PUT 与两个合法当前账号 owned 原提交。首轮定位用例用了不存在的 footer 选择器，修正为实际 learning-actions 后才取得应用失败证据：PUT 返回 410、标题聚焦“课程已撤回”、学习操作已移除，owned 存储仍为两项；qa.navigate('/login') 实际进入 /login，没有确认。
- 正常与 unavailable 的 section 现在都把同一个 PendingNavigation 放在首个子节点，使用学习 pending 或账号 owned pending 聚合状态；不可用分支保留该组件的身份和导航状态，仍只注册一个 blocker。原 removeUnavailable 只清理固定会话的草稿，独立原请求继续保留；修复不增加重放或撤回副作用。
- 定向回归通过（41 秒）：首次撤回后导航被拦截，Escape 回撤回标题；确认一个独立请求后再次导航仍被拦截，确认最后一个关闭提示/取消旧导航，仍停当前页并聚焦撤回标题，合成 beforeunload 不再阻止；下一次明确导航成功。只有一次进度写入，owned 目标请求为零（确认通过同一 clearPending/通知模拟其他控件的迟到确认）。最终完整 18 项 Chromium 回归通过（238 秒），包括加强后的弹窗已打开→410→提示仍打开→Escape 聚焦撤回标题。TS 7、26 项 Web 测试、6 项构建 SSR 测试、client/SSR build 和 Prettier/diff 通过，测试 teardown 已关闭专用随机浏览器会话与临时 Vite；此证据不证明真实数据库撤回、原生刷新弹窗、iPhone 或辅助技术验收。完整目标继续推进。

## 2026-10-06：未确认保存页的账号隔离与列表同步

- 两项新浏览器回归在修复前失败：首次确认后的实际焦点为 BODY，aria-busy 缺失；切换账号、释放旧成功响应后新账号列表由 1 条变为 0。测试使用实际 PendingSaves、LearningProvider 与 MemoryRouter，三个不同账号 scope 的合成原提交及受控 HTTP 响应，不连接用户账号或数据库。
- 页面现以账号 ID 为 React key 固定保存生命周期，同步 ref 拒绝重入，按钮改 aria-disabled/aria-busy 保留键盘焦点；只有当前挂载实例能改变 busy/error。成功与目标接口的确定拒绝仍按原 key/idempotencyKey 清理，卸载不取消已经发出的服务端请求。列表订阅草稿变更通知，始终重新读取 pendingOwned 的账号/端点/字段校验结果；提交入口也重新核对当前原请求，避免已确认行在 React 提交前再次发送。
- 修复后原生三次 Enter 只产生一次请求、BUTTON 焦点/busy=true；离页提示 Escape 返回 H1，503 保留原请求，重试 path/body 完全相同，确认后焦点在剩余按钮。模拟先前控件确认剩余条目后列表清空、空态获焦点、合成 beforeunload 不再 preventDefault、明确导航成功，无额外写入。初次测试对存储枚举顺序作了错误假定，导致确认已经移除的条目；改为根据实际首次请求选择剩余条目，未据此前超时宣称通过。
- 新账号在旧请求仍挂起时可开始确认自己的原提交；释放旧成功后，新列表仍一条且自己的 busy=true，只有新请求成功才清空并聚焦空态。两项定向回归通过（47 秒），最终源码的完整 17 项 Chromium 回归通过（226 秒），TS 7、26 项 Web 测试、6 项构建 SSR 测试、client/SSR build 与 Prettier/diff 检查通过。测试 teardown 已关闭自己的随机浏览器会话与临时 Vite，不触碰用户浏览器或共享服务。真实设备/辅助技术、课程人工审校与正式录音、生产验收继续待完成。

## 2026-10-06：表达卡片收起后的未确认保存

- 源代码核对发现 Bookmark/Enroll 的 beforeunload 依赖控件挂载；表达库收起卡片会卸载两者，原请求仍在 sessionStorage，但 Library 与公开 Lesson 没有 SPA 离页确认。新增 usePendingOwnedWrites，以 pendingOwned 的完整校验和当前账号 scope 读取 Boolean 快照，使用无 payload 的本标签通知与 storage 通知更新；SSR 快照为 false，不读取服务端浏览器存储。每路由一个 PendingNavigation；账号学习/复习合并原有 guard，避免多个表达控件分别注册导航拦截器。
- 新增实际 Library + LearningProvider + Bookmark/Enroll 装配的 Chromium 原生键盘回归，受控 fetch 暂挂 PUT 收藏与 POST 加入复习。收起卡片后正文控件为零，合成可取消 beforeunload 仍被 preventDefault；只释放加入复习成功后仍留一个原请求，跳登录被拦截，Escape 回到“我的表达”标题。收藏 503 后显式历史离开，存储 path/method/body 与原请求相同；返回展开后的明确重试使用完全相同 body/idempotencyKey。再次收起并打开离页确认，释放原重试成功后存储清空、提示关闭、旧跳转取消、标题获焦点，beforeunload 不再被拦截；下一次明确跳转成功。只发三次目标写请求，没有自动补发。
- 定向回归通过（1 项，47 秒），完整 15 项 Chromium 回归通过（211 秒），26 项 Web 测试、6 项实际构建 SSR 测试、TS 7、client/SSR build 和 Prettier/diff 检查通过。浏览器使用独立随机会话及临时 loopback Vite，teardown 关闭；受控数据与响应不证明 PostgreSQL 的提交结果、原生刷新弹窗、真实 iPhone 或辅助技术验收。完整内容审校、正式录音与生产验收继续待完成。

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

录音语义定位进展（2026-10-06）：课程录音描述的 assetId/revision/durationMs/sha256/url/creditZh 分别给出字段指针；过多 tracks、重复 block、空/超量 cues、未使用录音也指向对应集合或条目。时间轴区分 endMs 不晚于 startMs、越过录音时长、整句阅读顺序重叠、子区间早于/晚于父区间、缺少父 segment；保留原 cues 数组索引，不能用正文顺序冒充源位置。CLI check 通过现有 Document 索引映射原文件行列，新增九个实际子进程故障样本，包含中文和 CRLF、使用不可达 DATABASE_URL，证明确切字段值定位且不连接数据库；既有整段 cue 定位测试改为断言 endMs 值位置。新增契约测试验证乱序 cues 与子/父时间轴位置，原录音接受/拒绝规则不变。11 项契约、32 项 server 单元、10 项作者 CLI、12 项课程测试通过，Clippy 无警告；开发 API 在重新链接后恢复，/api/health 返回 200。此轮只改作者诊断，没有新录音登记、正式内容发布或生产上线；完整作者工具、人工审校和设备/生产验收仍待完成。

作者预览交互进展（2026-10-06）：隔离 PostgreSQL/API/Web 的 operator 草稿预览，合法地在两个步骤复用同一正文块，实际复现两个 knowledge-title ID 重复，第二个弹窗的 aria-labelledby 解析到第一个空标题，辅助树名称退化为整段内容。ReadingBlock 改为 React useId 实例标题，SSR/客户端关联稳定；第二个弹窗现在仅名为 baguette，全页无重复 ID，modal 初始焦点在关闭按钮、Escape 返回原词按钮。作者预览明确关闭 personalActions，不渲染收藏/加入复习组件；仍可查看与朗读，并保留三类判分预览。实际 operator 草稿题目判分后，数据库 learning_sessions/exercise_attempts/review_cards/saved_items 均为零；未发布草稿和测试素材只在隔离库。另建 synthetic reviewed 对照课程并在隔离库激活，正常账号学习页保留操作，实际收藏/加入复习各产生一条记录，未产生题目尝试。320/390/900px 弹窗在视口内、无横向溢出，390px 截图已查看，reduce 的动画 none/过渡 0s；TS 7、17 项 Web 测试和 SSR/client build 通过。专用浏览器、两个 QA 服务和临时 PostgreSQL 已清理，常用 API 健康 200。该检查不代表真实屏幕阅读器或 iPhone 验收，全部课程人工审校/录音与生产验收继续待完成。

练习反馈与重试进展（2026-10-06）：隔离 PostgreSQL/API/真实账号学习页复现确认后焦点落 BODY。ExerciseEditor 在本次提交确认后聚焦 feedback（tabIndex=-1），重试后聚焦当前题目的答案控件；初始已保存结果与外部最新记录不主动抢焦点。确认保存的恢复通知也可触发反馈焦点，完整恢复组合仍待验收。继续实测发现结果的 type=button 重试按钮在状态更新时被 React 复用为默认提交按钮，点击重试实际产生额外尝试。现为 retry/submit 使用不同 key，提交按钮显式 type=submit；新 DOM 节点避免浏览器默认动作把重试当成提交。实际键盘选择、填空、排序各自确认后焦点在该题反馈；重试分别回到 radio、保留 une 的 input、已排序语块按钮。修复前重试试验累计 4 次尝试；修复后重试保持 4，明确提交错误选择增至 5、明确提交填空增至 6、明确提交排序增至 7，各类后续重试不新增。作者草稿另合法复用填空块，两个 input 改 React useId 独立关联标签，DOM 无重复 ID；第二题实际预览判分焦点正确，重试回到第二 input，fetch 计数从 1 保持 1，数据库尝试仍为 7。320/390px 无溢出，390px 聚焦反馈截图已查看；TS 7、17 项 Web 测试与最终 SSR/client build 通过。专用浏览器、3002/5177 两个服务、临时 PostgreSQL 已清理，常用 API 健康 200。未发布正式内容；会话失效/请求失败更多组合、辅助技术/真实 iPhone、人工审校与生产验收继续待推进。

## 长词响应式与减少动态效果回归（2026-10-06）

独立 Chromium/Vite 装配使用实际 Lesson、Learning、Reviews、Profile 和 ExerciseEditor 组件，合成课程/身份、受控 HTTP/TTS，不连接用户常用浏览器或数据库。加入长法语词 anticonstitutionnellement、长标题、38 字符昵称及三类题目边界数据，仅作为技术压力样例，不作为 A1 教学内容。修复前 320px 对话词按钮从 x=78 延伸至 321.625；修复阅读词按钮后，学习页标题文字仍使 document.scrollWidth=372（视口 320），元素盒边界并未显示该文字溢出。现在标题、单词、选择标签、填空题句、排序词块和昵称按可用宽度换行，保留完整词按钮/词块及原始点击、朗读与答案值。

新增持久浏览器回归检查 320/390/768/1440px：等待页面有限动画结束、确认实际组件已显示，分别核对控件盒、文字 Range、父容器与视口边界及文档宽度，避免将 overflow:hidden 的裁剪误判为通过。对话/短文切换保留键盘操作；长词 Enter 仍完整交给受控 TTS，320px 排序词块 Enter 移入句子后仍在视口内。另启用浏览器 prefers-reduced-motion，原生 Enter 展开实际复习卡片，确认 document.getAnimations() 为零、transitionDuration=0s，然后恢复测试会话媒体设置。

最终全 22 项 Chromium 回归通过（295.96s），27 项 Web、6 项 SSR、TS7 类型检查与 client/SSR 生产构建通过；测试自动关闭独立浏览器和随机端口。此证据覆盖组件装配，不冒充完整生产页面壳、真实 iPhone、安全区/系统键盘、法语系统声音或辅助技术验收。完整教学审校、录音、设备与公网生产验收仍待完成。固定前一提交 68a361d 的远端 Check 37439244244 已实际读取为 completed/success；本次提交的远端结果须单独确认。

## 表达库响应式与恢复焦点（2026-10-06）

把实际 Library 收藏卡片纳入 320/390/768/1440px 控件/文字 Range/文档边界回归。修复前 320px 的长 lemma 文字右边界为 359.15625，卡片右边界仅 300；现在 heading 的文字 flex 子项可收缩并换行，右侧 SVG 保留空间。原生 Enter 展开、受控 TTS 完整提交长词、再次 Enter 收起且焦点保持标题均通过。语音观察采用实际受控 speak 记录；Library 装配没有阅读装配的 playback 状态探针，最初等待该未更新字段超时不计为产品失败或成功证据。

增加实际 Library 复习管理卡片装配及受控 preferences PUT/固定卡片 GET。修复前保存 503 后原生键盘重试，disabled 使焦点落 BODY；现重试与重读采用 aria-disabled/aria-busy，同步写锁和读取锁阻止重复请求。原键/完整 body 重试不变，成功确认后返回暂停/恢复操作；明确 422 后聚焦错误 alert。409 后恢复 GET503，再次手动 GET503 时按钮焦点保留，明确重读成功后返回卡片标题，不自动重发偏好。三个重复 Enter 只产生一个新增 GET。撤回处理仍沿用原不可用标题与服务端规则。

继续加入迟到 GET 用例，实际复现已经移到“回看来源课程”的焦点被成功读取拉回旧标题；现用户离开读取/重试按钮时取消本次焦点恢复。最终回归确认保持来源链接焦点。完整 23 项浏览器先通过（328.04s）；上述最后焦点调整后，对表达库离页保护、四档响应式和管理恢复三项受影响回归重新全部通过（121.55s），最终 TS7、client/SSR build、6 项 SSR 与格式/diff 检查通过，27 项 Web 在本轮通过。所有资源为自动清理的随机 Vite/独立 Chromium，课程/身份/HTTP/TTS 均为合成受控协议，不代表真实 PostgreSQL、iPhone、系统声音或辅助技术验收。

固定前一提交 30da2a3 的 Check 37441523834 本轮实际读取为 completed/success；本次提交的新 CI 仍须另行确认。正式教学审校、录音、完整设备与生产验收继续待完成。

## 素材与录音整包离线预检（2026-10-06）

新增 assets-check <bundle.json> <source-directory> 与 audio-bundle-check 同参数，在数据库/fixture 模式检查前返回，不需数据库或 actor，不写源文件、MEDIA_ROOT、登记或审计。严格 JSON/原位置索引与导入元数据校验沿用现有工具；在 blocking worker 按顺序处理至多 500 项，一次只保留一个有界原文件。图片目录边界/读取/哈希/解码与实际尺寸、录音目录边界/读取/完整解码/哈希与真实时长，均提取为正式 import 与离线工具共用的文件检查；正式导入仍重新读取并执行原有写文件、登记事务与重复版本/角色引用验证。

新 CLI 回归先复现未实现命令进入模式检查并失败；实现后使用合成 96×96 SVG 与 100ms 单声道 WAV，合法包成功，哈希/图片宽度/录音时长/缺文件六种故障均定位中文 CRLF 清单原值行列。测试使用不可用数据库和 production+fixture，证明命令不访问这些配置分支；每次确认媒体目录未创建、源文件字节保持不变。清单 ready/授权数据仅为 synthetic/LicenseRef-TestOnly，不改现有 planned 内容或声明正式授权；角色登记引用/正方形头像、版本冲突和课程时间轴/审校仍属于 import/release 验证。成功信息明确 not registered or published。

37 server 单元、24 作者 CLI、13 curriculum 共 74 项通过，Clippy（所有 target、-D warnings）、fmt/diff 通过。独立 PostgreSQL 的 author_runtime 与 recording 两项真实集成通过，覆盖原文件诊断/导入与 staging 原子性、图片登记、录音不可变/整批登记与固定 revision；专用容器 brioche-bundle-qa-9ae33ab6ea4e（仅 loopback 临时 49500）按精确 ID/名称核对后连匿名卷清理，初次失败的临时合成文件也按路径/内容核对清理。共享开发服务、用户浏览器、生产数据未修改。

固定前一提交 9f85c01 的 Check 37443645612/job112203129839 本轮已实际确认 completed/success；本次整包预检提交的 CI 须另行确认。

## 同包角色头像形状预检（2026-10-06）

新增作者 CLI 回归先实际复现：角色引用同包 640×470 场景图时 assets-import 元数据仍通过，进入不可用数据库后只得到 database connection failed。AssetBundle 的共用元数据检查现按精确 avatarId/avatarRevision 查同包记录，发现长方形立即定位 /characters/0/snapshot/avatarId；assets-check 与 assets-import 均在文件/数据库访问前报告中文 CRLF 原值行列，错误理由 avatar must be square。包内不同 revision 不替代目标版本；包外版本及纯角色包继续留给登记数据库查询，没有误拒已有合法头像。

新单元回归验证合法包、确切长方形 revision 拒绝、同 ID 不同 revision 延后、较新方形版本可用以及纯角色包合法。38 server 单元、25 作者 CLI、13 curriculum 共 76 项通过，Clippy 所有 target/-D warnings、fmt/diff 通过。扩展真实 PostgreSQL author_runtime 回归：纯角色包引用已登记方形头像成功，仅新增一条角色/审计，素材数量保持；另一纯角色包引用已登记场景图仍由数据库形状检查拒绝，角色/素材/审计数量保持，原文件头像引用位置正确。完整导入/staging 原子性回归通过。按精确 ID/名称清理专用 brioche-avatar-qa-455993471ddc 容器及匿名卷；未访问用户数据库、修改课程/素材审校状态或开放媒体。

固定前一提交 ac0e090 的 Check 37444617769/job112206323082 本轮轮询确认 completed/success 后才推送，未取消其浏览器回归；本次头像预检提交的 CI 须另行确认。

## 2026-10-06：首页表达卡片与课程搜索

- 在真实 Home/Courses 组件的独立 MemoryRouter 装配中复现：没有 reviewItemIds 对应词汇时仍有空的表达按钮；原生搜索按钮 disabled 让焦点落到 body；暂停实际 Web Animation 的时钟后，第二次原生按键切换留下两段高度效果。最初不控制动画时钟的重复按键检查通过，只表示 CLI 间隔超过动画时长，不能证明快速切换正确。
- 首页仅渲染有内容的表达卡片，保留账号复习和固定会话续学入口；表达卡片独立保存展开状态，按课程/表达重建，取消旧动画和待执行帧，从当前视觉高度开始新效果，卸载清理。减少动态效果下直接展开。
- 搜索使用 aria-disabled/aria-busy 与提交拦截，保留输入节点和焦点。受控待返回结果期间重复 Enter 仅一次读取；输入提交后的结果不重建输入；等待期间继续输入的草稿不被旧结果覆盖；空结果明确显示 0 堂课程，查看全部同步清空输入并恢复课程列表。
- 320px 实际文字 Range 复现首页长 lemma 伸到 400.69px；为表达、续学标题及课程列表补充换行。响应式矩阵加入首页/目录，在 320/390/768/1440px 核对控件、实际文字、父容器与文档边界；减少动态效果检查加入首页展开。
- 全 26 项 Chromium 回归通过（365.35s），27 Web/6 SSR、TS7、client/SSR build、定向 Prettier 与 diff 检查通过。独立浏览器和随机 Vite 由测试自动清理；合成账号/目录与受控 loader、语音和动画时钟不替代生产 SSR 搜索/数据库、真实时间的设备动效、iPhone 或辅助技术验收。
- 固定前一提交 05d71a6 的 Check 37445542394 本轮确认 completed/success；本轮提交的远端 CI 仍须另行确认。课程人工审校、正式授权/录音、真实设备与生产门槛保持待完成。

## 2026-10-06：复习记录分页与窄屏日期

- 新增真实 History 组件的独立 MemoryRouter 分页装配：最新页两条、旧页一条、末尾空页及尚无历史的首屏。原生键盘进入旧页/空页时现有游标焦点 hook 回到 H1；修复前空旧页仍提示「完成一次复习后，记录会显示在这里」，实际断言失败。
- 旧页现在提供「返回最新记录」文字入口，分页导航上下排列并命名。末尾空页与初次空态分别提示，状态语义保留；返回最新后两条历史恢复且焦点回 H1。受控记录另核对撤回词汇隐藏/中文 lang、原自评，以及 UTC 23:30 在提交时区 Asia/Shanghai 显示次日 07:30。
- 新增记录页 320/390/768/1440px 的控件/文字 Range/父容器/文档边界检查。修复前长 lemma 的 flex 最小宽度把「下次 10/9」挤到 335.44px；结果列表正文现在可收缩换行，日期保持单行且不被挤出。共用样式同时适用于已有复习结果列表。
- 生产构建 SSR 新回归通过真实 React Router request handler + 专用受控 HTTP 后端：实际 History loader 转发编码游标，仅带会话 Cookie、只 GET、响应 private/no-store；服务器渲染空旧页和返回最新链接；未登录 302 到 /login?next=/review-history。该测试不以合成游标/后端代替 PostgreSQL 游标验证。
- 全 28 项 Chromium 回归通过（383.97s），27 Web/7 SSR、TS7、client/SSR build、定向 Prettier/diff 检查通过；自己的随机 Vite/浏览器自动清理，未操作用户浏览器、账号或生产部署。真实 iPhone/辅助技术、课程人工审校、正式授权/录音及生产门槛仍待完成。
- 前一提交 86b1bac 的 Check 37447935855/job112217173020 先确认为 browser 步骤运行中，后实际 completed/success；本轮新提交的 CI 另行确认。

## 2026-10-06：账号表单等待与请求生命周期

- 真实 Account/LearningProvider/MemoryRouter 装配先复现三处问题：原生 disabled 让登录等待焦点落到 body；密码恢复成功后焦点也在 body；离开等待中的 CSRF 页面后释放响应，实际仍新增一次登录 POST（1 !== 0）。这些不是通过源码推断的通过项。
- 输入等待时只读，按钮以 aria-disabled/aria-busy 和同步锁保留焦点并拒绝重复提交。失败保留邮箱/密码；焦点仍在表单时聚焦 role=alert 的错误，已经移到独立入口时不由迟到错误抢走。成功密码恢复清空密码/token，并聚焦成功 H1。
- authRequest 接受可选 AbortSignal，与既有 CSRF/POST 截止共同使用；读取 CSRF 后再检查取消状态。Account 卸载会取消当前请求，旧请求的成功/失败/finally 不更新已卸载或已经换链接的编辑器。新的邀请/恢复 fragment 先取消旧等待并清空密码、昵称和反馈，再读取新 token/email，移除地址 fragment；已有身份通知仍由成功账号响应触发。
- 四项新增原生键盘回归通过：等待焦点/只读输入/重复 Enter、401/429/503 失败及表单外焦点；离页取消 CSRF 且没有晚发 POST；恢复 fragment 清理、精确 token/password 请求、成功标题/输入撤除与新身份 nonce；新邀请替换期间旧 POST 被取消，迟到成功不导航，新请求只使用新 token/email/昵称/密码，400 后保留输入。
- 新 SSR 回归使用真实生产 build/request handler：匿名登录含正常表单；没有客户端完整链接时邀请/恢复表单保持关闭；访客模式不显示密码输入；全过程仅 GET。共 8 项 SSR 通过，HTTP 后端为隔离受控适配器，不表示真实 PostgreSQL 账号验收。
- 全 32 项 Chromium 通过（424.56s），27 Web/8 SSR、TS7、client/SSR build、定向格式/diff 检查通过。自己的临时 Vite/浏览器自动清理；测试只用明确标记的合成凭据/受控响应，不连接真实账号。取消已经发出的 POST 不证明服务端回滚或 cookie 未变化，身份仍需服务器重新授权；真实 iPhone/辅助技术、课程人工审校、正式录音/授权和生产门槛仍待完成。
- 上一提交 139b2ae 的 Check 37449232189/job112221439047 本轮先确认运行中，后实际 completed/success；本轮新提交的 CI 另行确认。

## 2026-10-06：退出操作焦点与身份生命周期

- 真实 Profile/LearningProvider 装配复现：原退出按钮 disabled 使焦点落到 body；合成身份从 Alice 换成 Bob 后，旧退出 POST 的迟到 200 仍执行首页全页导航，Bob 标题实际变成 null。
- 退出现使用同步 ref 锁和独立 AbortController。按钮 aria-disabled/aria-busy 保留焦点，重复原生 Enter 不增加请求；同一资料身份离页或被替换时取消客户端等待，旧成功/失败不再清理草稿、停止新上下文播放、导航或 toast。成功返回后锁保持到离页，失败才开放明确重试。
- 第一项新增键盘回归：受控 503 后按钮焦点与 owner 草稿保留，明确重试的 200 才清理 account-a 前缀草稿并执行真实 window.location.assign('/')；同源重新加载后 account-b 前缀及无关项仍保留，随后清理本次精确合成键。该导航目的页为测试装配首页，不将它当作生产匿名首页/真实 cookie 验收。
- 第二项新增回归：身份替换期间原请求被取消，迟到 200 不导航、迟到 503 不向 Bob 弹提示；在 CSRF 暂挂时以真实链接离开个人页，旧响应不发 logout POST，仍在复习入口。最初的 toast 断言误选不存在的 role=alert，已改为实际 toast 的 hidden 状态并补迟到 503 场景后通过，未将原断言计作提示证明。
- 新生产 SSR handler 回归确认个人页按服务端身份显示资料/退出或匿名登录入口，匿名页面不含合成用户邮箱，保持 private/no-store，转发仅会话 Cookie，渲染全过程只有 GET。HTTP 后端为受控适配器，不代替 PostgreSQL 会话撤销或跨标签实际 cookie 检查。
- 全 34 项 Chromium 通过（446.57s），27 Web/9 SSR、TS7、client/SSR build、定向格式/diff 检查通过；独立随机 Vite/浏览器自动关闭，没有访问真实账号或操作部署。取消已经发出的 POST 不表示服务端回滚，身份仍由服务器重新确认；真实 iPhone/辅助技术、人工内容审校、正式素材/录音授权与生产门槛继续待完成。
- 上一提交 8c55050 的 Check 37451038196/job112227392494 本轮实际确认 completed/success；本轮新提交 CI 另行确认。

## 2026-10-06：固定版本预览入口同步

- 真实 AuthorPreview 的独立 Chromium 装配复现：编辑三个预览字段后，从目录打开另一课程第 2 版，正文已切换但表单仍显示 draft-release、draft-lesson、第 9 版。新增用例在修复前实际失败。
- 预览参数变化时同步批次/课程/版本三个输入值，保留输入 DOM；焦点与滚动转到已加载课程标题，只有目录时转到批次标题。初始渲染不抢焦点，同参数重渲染不重置正在输入的草稿。
- 原生键盘回归覆盖编辑后目录跳转、提交另一批次、打开该批次第 3 版，以及 MemoryRouter 返回目录；逐次核对全部字段与活动标题。合成 loader 不代替实际权限与数据库。
- 新生产 SSR handler 用受控 HTTP 后端另验证匿名 401、learner 403 且无 operator 读取；operator 按批次及确切版本读取，private/no-store、Vary Cookie、只转发 session cookie；不属于所选批次的版本 404 且不读取课程，非法批次 400 且不读取预览。此证据不表示数据库发布或正式内容审校完成。
- 本轮受影响的三个 Chromium 用例通过（58.63s，包含多正文与迟到语音回归），27 Web、10 SSR、TS7、client/SSR build、定向 Prettier/diff 通过。本轮未重新执行全部 35 项浏览器用例；独立浏览器与随机 Vite 已清理，没有操作用户服务或生产。
- 固定上一提交 d44dc39 的 [Check 37452861794](https://github.com/Ronbb/brioche/actions/runs/37452861794) 本轮实际读取 completed/success；新提交另行确认。完整教学审校/正式录音、真实设备/辅助技术与生产门槛继续待完成。

## 2026-10-06：预览判分的身份与课程生命周期

- PreviewExercise 旧 ownerRef 仅过滤返回结果，未按身份重建 pending/草稿/反馈。新增实际 AuthorPreview、LearningProvider、ExerciseEditor 的 Chromium 装配，在旧管理员判分 POST 暂挂时切换到另一 operator；修复前新身份的 fieldset 持续 disabled，回归在等待恢复作答处实际失败（63.43s）。首次测试装配没有练习块，等待 radio 超时，已补合成选择题和 practice 步骤；该首次失败不作为缺陷证据。
- 现按 operator ID、lesson ID/revision、exercise ID 固定内层练习实例，换身份或题目卸载旧实例。useLayoutEffect 清理关闭活动状态并取消专用 AbortController；旧成功/失败不能更新新实例、toast 或反馈焦点。新实例保留独立同步提交锁。
- privateRequest 新增可选 signal，保持未传信号的既有调用/超时语义，组合外部取消和超时；开始前及 CSRF JSON 读取后再次核对取消状态，避免旧引导响应继续发 POST。新增单元回归让 CSRF JSON 忽略网络取消后迟到返回，证明取消后零判分 POST；预先取消也不访问网络。
- 身份回归验证新 operator 的选择清空、控件可用、旧请求 signal aborted；新判分暂挂时释放旧成功，仍处于新请求等待且无旧反馈，新请求成功才显示当前反馈并聚焦。另一回归使用目录键盘链接换到另一课程固定版本，CSRF signal aborted，迟到引导无 POST、无 toast、无选择或等待残留。它们使用合成身份/受控 HTTP，不证明真实登录 cookie、数据库权限或服务器回滚。
- 初次修复后两项 author 回归通过（50.40s，含前一轮参数切换）；新增取消断言和课程切换后最终两个受影响用例通过（45.81s）。28 Web、10 SSR、TS7、client/SSR build、定向格式/diff 通过。本轮未运行完整 37 项浏览器；自己的随机 Vite/Chromium 自动清理。已发请求的服务端处理不因客户端取消而保证回滚；正式内容审校/授权/录音、真实 iPhone/辅助技术和生产门槛继续待完成。
- 固定 f410336 的 [Check 37453982397](https://github.com/Ronbb/brioche/actions/runs/37453982397) 本轮实际读取仍 in_progress，不提前声称远端绿色。

## 2026-10-06：三类练习等待与重试的确认焦点

- 真实 AuthorPreview/ExerciseEditor 装配新增选择、填空、排序三题。原生键盘选择后点击确认，修复前请求已发且暂挂，活动元素已不是确认按钮，新增回归实际失败 false!==true（33.37s）；该首次失败只证明选择题等待焦点丢失，不声称三题基线都执行完成。
- 共用 ExerciseEditor 的确认按钮不再因 blocked 原生禁用；无答案仍 disabled，blocked 使用 aria-disabled/aria-busy。表单原有 ready/blocked/completed 守卫保留，底层预览同步锁仍保留，答案 fieldset 等待时仍禁用。不改变判分/私有规则、原请求协议或真实进度写入。
- 三类原生键盘回归逐题确认实际提交 revision1、确切 exerciseId、choice=bonjour、text=une、order=[bonjour,luc]；等待按钮焦点和忙碌标记保留，连续 Enter 只有一次请求，503 后控件恢复且焦点仍在确认。明确再按 Enter 的 body 与原请求完全一致，成功反馈聚焦。受控判分不代替数据库写入/幂等或真实辅助技术。
- 三项受影响 author 浏览器回归通过（66.45s）；补充上述确切答案断言后，新三题用例最终单独通过（52.26s）。28 Web/10 SSR、TS7、client/SSR build、定向 Prettier/diff 通过，本轮未重跑全部 38 浏览器。测试自己的 Vite/Chromium 自动清理。ExerciseEditor 为真实账号学习与管理员共用；此浏览器用例以管理员受控请求装配，不将它当作实际账号完整学习链路。
- 固定 6ac433d 的 [Check 37454719659](https://github.com/Ronbb/brioche/actions/runs/37454719659) 本轮读取仍 in_progress，最新远端绿色尚未确认。正式审校/素材与录音、真实 iPhone/辅助技术及生产门槛继续待完成。

## 2026-10-06：首屏覆盖滚动条实际接入

- 根 Layout 原 html 只有 lang，整个 Web app 没有启用 overlay-scroll；既有 CSS 的隐藏原生滚动条规则因而未匹配，不能把 CSS 文件已存在当作页面已生效。新增生产 SSR handler 回归在原构建实际失败，首屏 HTML 为 html lang=zh-CN、无该 class。
- Layout 直接在服务端根 html 加 overlay-scroll，SSR/客户端使用相同类，无需挂载后改变滚动条模式。沿用原覆盖指标、原生滚动能力、fixed 布局、隐藏零高度滚动条与键盘交互；未改 OS、用户浏览器或正在运行的服务。
- 新独立 Scrollbar/现有应用 CSS 装配，四档 320/390/768/1440px 均验证 document.clientWidth 与 scrollWidth 等于视口宽、scrollbar-width none、track fixed；原生键盘 End/Home 对应 aria 最大值/0且保留轨道焦点；按钮切换2400/200px内容后轨道隐藏/显示，页面宽度不变。
- 第一轮浏览器错误是用默认等待可见的 selector 去等 hidden 元素，发生等待超时；改为读取 hidden 属性后最终回归通过（52.74s）。不将该工具断言问题算作另一应用缺陷。组件装配按 SSR 应有类名启动，不等于完整生产浏览器页面壳或 iPhone/辅助技术验收；实际根类另由生产 SSR handler 的匿名/私有个人页证明。
- TS7、client/SSR build、全部11项SSR、定向格式/diff通过；本轮不重跑全39浏览器或无关Web单元。自己的随机Vite/Chromium已自动清理。ddcf170 的 Check37455236784 本轮读取仍 in_progress，最新远端绿色未确认。完整内容人工审校/素材与录音、真实设备/辅助技术、生产门槛仍待完成。

## 2026-10-06：访客三类练习与离页生命周期

- 访客 PracticeSession 为独立实现，原确认按钮 pending 时原生 disabled，结果 status 无焦点目标、答错重试只移除结果。新增真实 Practice/LearningProvider/MemoryRouter 的三题受控装配，修复前首题判分暂挂时确认按钮失焦，实际回归 false!==true（33.61s）；不声称基线后三题流程都执行完成。
- 有答案的确认按钮在 pending 使用 aria-disabled/aria-busy 保留焦点，原同步 busy 锁阻止重复请求；答案 fieldset 仍禁用，无答案仍 disabled。反馈增加独立 ref/tabIndex，当前结果提交后聚焦；答错明确再试返回首个可用答案控件，原选择/文本/排序保持。下一题与回顾保持原标题焦点、停止播放与演示不保存账号的边界。
- active 与请求取消在 layout 清理时失效；成功 JSON 后重新核对活动/取消，失败或 finally 不再更新卸载页面。取消不表示服务端处理回滚，demo 判分仍由 Rust fixture 接口完成；生产旧 practice 路由仍按真实课程入口重定向。
- 三题原生键盘回归通过（60.92s）：连续 Enter 仅一请求，503 后保留确认焦点及答案，明确重试提交 revision1/确切题目与 choice=bonjour、text=une、order=[bonjour,luc]；受控答错聚焦反馈，明确再试返回答案，再次正确后进下一题，最终回顾三题/三正确/标题焦点。额外离页回归通过（48.37s）：暂挂判分离页 signal aborted，返回新会话选择清空；新请求暂挂时释放旧200/503均无结果/错误/toast、仍等待新请求，当前响应才显示当前反馈。
- TS7、client/SSR build、全部11SSR、定向Prettier/diff通过；本轮未重跑全41浏览器或无关Web单元。自己的随机Vite/Chromium自动清理，未访问用户账号或改变部署。受控HTTP/合成课程不代替真实fixture服务、数据库、iPhone或辅助技术验收。
- 固定d1dab5d的 Check37455801225 本轮实际读取仍in_progress，不提前声称远端绿色。完整人工内容/正式素材与录音、设备/辅助技术、生产门槛继续待完成。

## 2026-10-06：生产 SSR 页面壳与客户端路由检查

- 新增独立 `pnpm test:browser:ssr`，前置 `pnpm build`；加载实际生产 server/client 构建和真实 Layout/RouteFocus/Scrollbar/页面组件，以两台临时 Node HTTP 服务提供生产 SSR、构建静态资源和受控公开课程 API。课程从示例作者源只选公共字段，去私有规则/editorial，不连接数据库或真实账号。CI 在构建、SSR和组件浏览器测试后运行该检查。
- 最终本机 Chromium 一项通过（44.80s）：实际首页320/390/768/1440px根clientWidth/scrollWidth均等于视口、顶部栏不越界、根滚动条样式none；390px原生Enter使用跳到正文并聚焦真实main；头像进入/profile、品牌回首页和目录链接进入/courses均在真实hydrated路由聚焦main h1，浏览器内标记保留证明并非全页重载。
- 390px真实语速dialog在视口内、焦点在modal、根覆盖滚动条隐藏；Escape返回原速度按钮。核对未捕获浏览器异常与测试服务器异常均为空。不是纯组件装配，但API为受控fixture，这只补代表性公共页面壳/路由证据，不声称全部页面四档宽度、真实SSR服务部署、Rust/database、iPhone/屏幕阅读器或原生声音全部完成。
- 定向Prettier/diff通过，浏览器与两台随机loopback服务器自动清理；未触及用户5173/3001、Docker或浏览器。此轮只增加测试/CI/协作说明，应用源未改，不重新运行无关Rust/Web或完整41组件浏览器。
- 固定155b3b0的 Check37456413593/job112245009700 本轮先确认运行，后读取已至pnpm test:browser步骤（26），无失败，仍in_progress；正式终态和最新提交CI另行确认。完整内容/录音、真实设备/辅助技术与生产门槛仍待完成。

## 2026-10-06：身份失效时先卸载私有页面再刷新

- 增强生产SSR/client页面壳装配，受控HTTP API按本次专用浏览器的合成shell-a/shell-b Cookie返回Alice/Bob，真实IdentitySync走同源HTTP读取；不接真实认证/数据库。实际资料编辑器输入Unsaved Alice后切合成Cookie并派发可见窗口focus通知，后端确实收到浏览器身份读取shell-b。
- 初始Cookie操作未先打开所属页面导致Invalid cookie fields，已显式关闭那一个专用会话，并将opened设置提前保证失败也清理；编辑按钮/输入定位器也按真实DOM更正。随后等待新页面/尝试dialog状态存在时序超时，未把这些工具等待计作应用缺陷。最终用实际location.reload触发的beforeunload监听发送仅含三个布尔/计数值的同源测试beacon，从Node侧读取：修复前实际得到guarded=true、oldProfile=true、dialogs=1，新增断言明确失败（35.49s）。不依赖浏览器在原生离页确认期间能执行eval，也不声称该证据证明原生提示的视觉或手势验收。
- IdentitySync现先沿用停止播放/旧owner草稿清理，再flushSync通知Layout。Layout卸载旧路由children并重建匿名LearningProvider，旧资料modal、播放器/上下文与其离页监听同步清理；中性账号更新标题获得焦点，提供重新加载入口，再发起真实reload。仍由SSR重新验证会话，不采用刚收到的客户端身份直接授权；网络读取失败和原通知/检查协议不变，客户端清理不等于服务器写入回滚。
- 最终两项生产页面壳Chromium通过（51.44s）：原首页四档宽度/SPA焦点/速度modal保持；新身份用例beforeunload beacon为guarded=false、oldProfile=false、dialogs=0，真实重新加载后Bob可见、旧输入不存在、零打开资料modal、零浏览器/测试服务器异常。临时Cookie/证据存储、专用Chromium和两台随机loopback服务清理；没有操作用户服务或生产。
- TS7、client/SSR build、28Web、11SSR、定向Prettier/diff通过。本轮未重跑全41组件浏览器；此用例是实际生产页面构建加受控HTTP身份/人为focus事件，不冒充实际登录/双标签cookie轮换、数据库、iPhone或辅助技术。更多失效场景及正式教学/录音/生产门槛继续保留。
- 155b3b0 的 Check37456413593/job112245009700 与4b95aeb的 Check37457007400/job112246969535 本轮均实际确认completed/success；后者包含新生产SSR浏览器步骤。绿色对应这两个固定提交，本轮身份修复的新CI另行确认。

## 2026-10-06：错误状态与课程恢复入口

- 新增实际生产 SSR 回归，受控课程 API 分别返回 404/410/503；修复前三个状态共用“暂时无法打开”，标题断言实际失败。根 ErrorBoundary 现分别显示页面不存在、课程撤回、服务不可用；404/410 返回课程目录，5xx 提供原地址重新加载及目录入口，操作纵向排列。401/403 使用明确身份/权限文案，400 保留服务端输入校验说明；不把路由内部诊断直接作为 404 页面正文。
- 扩展实际 SSR/client/Layout 的独立 Chromium 装配。第一轮错误地在首页加载前注入 503，首页本身也读取课程，入口未出现；修正为首页完成后注入故障，该失败不计应用缺陷。直接打开错误页后确认恢复 Link 已 hydration，再执行原生 focus/Enter；目录实际到达但焦点停 BODY，等待标题焦点仍超时，证明根错误恢复后的 Layout 重新挂载漏掉路由焦点。RouteFocus 现把挂载时的 PUSH/REPLACE 视为导航，仍保留首屏 POP 的初始焦点策略及 modal/目标已聚焦保护。本轮未新增 POP 回退组合的验证。
- 最终三项生产页面壳浏览器测试 60.87s 全部通过：503 后原生 Enter 重载显示正文；404/410 不显示旧正文，目录 Link 原生 Enter 后标题聚焦；既有四档首页宽度、390px 跳到正文/语速弹窗与身份切换 beforeunload 收敛继续通过，浏览器无未捕获错误。28 Web、12 SSR、TS7、SSR/client build 与目标格式/diff 通过，400 作者编号错误正文另有 SSR 断言。独立随机 HTTP 端口与 Chromium 会话自动清理，没有操作用户常用服务或真实账号。
- 后端是受控 API；这些结果不替代真实 PostgreSQL 撤回/服务故障、iPhone、辅助技术或所有页面验收。本轮未重跑全41项组件浏览器。73b8c5e 的 Check37459238918 本轮读取为 in_progress，不能称为终态成功。完整内容人工审校、正式录音、真实设备及生产门槛继续保留。

## 2026-10-06：根错误页历史后退恢复焦点

- 在实际生产页面壳错误恢复用例中补充目录→故障课程→浏览器 back。上一提交仅在挂载时识别 PUSH/REPLACE；新增 POP 回归在“目录路径且标题聚焦”的等待处实际超时，不能把上一轮链接返回成功当作历史后退已完成。
- 现在根 ErrorBoundary 退出后在下一动画帧对已提交页面执行共用焦点恢复；RouteFocus 恢复普通首次挂载不抢焦点的策略。共用函数仍优先保留目标页面已有焦点及打开的 modal，使用 preventScroll。错误恢复不再依赖导航类型；没有改变服务端授权、内容状态或课程请求。
- 最终三项生产 SSR/client/Layout Chromium 66.43s 全部通过：404和410各自直接打开→链接返回目录、从目录打开故障课程→实际 browser back，均恢复目录标题焦点；503重载、既有首页多宽度/语速modal与身份失效收敛继续通过，未捕获浏览器错误为零。28 Web、12 SSR、TS7、build、目标格式/diff通过；独立会话/随机端口自动清理。本轮未重跑全41组件浏览器；测试仍为受控后端，不证明真实数据库撤回、iPhone或辅助技术验收。
- 固定73b8c5e的 Check37459238918 本轮已实际读取 completed/success；该状态不证明最新1024479或本轮改动的远端CI。人工内容/正式录音、真实设备与生产门槛继续保留。

## 2026-10-06：批次激活的本地作者诊断

- 新增真实 CLI/隔离 PostgreSQL 回归，期望 generation 冲突能指出参数与当前值；修复前 release-activate 仅输出 Error: request failed，诊断断言实际失败。首次默认 target 构建也因运行中的 brioche-server.exe 无法替换而失败，不计功能证据；随后复用独立 target/author-qa 完成构建，没有停止用户开发服务。
- content::activate_author 与既有运行时 activate 现在共用 activate_impl 和 ReleaseFailure，事务锁、版本核对、媒体再验证、发布标志/active release/audit 的更新及 commit 顺序不变。CLI 能定位 release-id 不存在或含撤回版本（含课程ID/revision）、expected-generation 冲突（含 expected/current）、非法操作参数，以及固定课程ID/revision的媒体字段诊断。DB故障仍只给验证状态后重试的通用信息，不输出 SQL/连接秘密；运行时接口仍仅返回原 AppError。
- 真实隔离 QA：generation 9 vs0、缺批次、篡改媒体对象、硬撤回四类CLI失败分别有准确诊断；失败后 active_release 为null、generation0、published0、audit保持1。恢复媒体后CLI成功激活原批次，active release/generation1/audit2；随后的撤回和拒绝再激活保持audit3。既有目录原子切换/回滚/硬撤回、迁移发布及录音登记三项PostgreSQL回归也通过，共四项。38服务端单元、25作者CLI、13课程测试（48草稿/144题）、Clippy全部target及格式/diff通过。本轮没有修改课程审校状态、正式素材授权或生产数据，也未改Web，未重复浏览器检查。
- 隔离资源：容器 brioche-author-activate-1791288897837 仅随机loopback58722的临时PG，测试成功的schema/媒体按既有teardown清理；baseline失败遗留schema和临时媒体。自动审批拒绝含删除临时目录/容器的清理命令，原因仅返回 blocked by policy；未绕过，随后 docker stop 成功，保留停止容器和失败测试媒体目录用于恢复。该清理限制不阻止代码验证或提交。真实设备、内容人工审校/正式录音及生产门槛继续保留。

## 2026-10-06：课程撤回的本地作者诊断

- 在作者运行时回归中补真实 content-withdraw CLI 的 generation 冲突、revision0、课程不存在及重复撤回。修复前冲突只输出 Error: request failed，诊断断言实际失败；首轮复用停止容器后沿用旧随机端口58722导致PoolTimedOut，只是装配失败，重新读取docker port的49472后才取得真实baseline。
- withdraw_author与既有withdraw现在共享withdraw_impl/ReleaseFailure，保留FOR UPDATE、乐观generation、发布标志更新、不可逆撤回插入、generation及审计的同一事务，运行时AppError状态不变。CLI明确lesson-id、revision、expected-generation（expected/current）、actor/reason或课程版本不存在/已撤回；包含课程ID/revision定位，没有SQL/连接信息。
- 隔离PG实际CLI验证：冲突/revision0/不存在后withdrawals0、published1、generation1、audit2；合法撤回成功后withdrawals1、published0、generation2，重复撤回失败且generation/audit不再增加。撤回后原release激活继续被拒绝，新staging也拒绝，audit最终3。四项PG回归（作者CLI、迁移发布、录音、目录原子切换/回滚/硬撤回）、38server单元/25作者CLI/13课程、Clippy all-targets、fmt/diff通过；使用独立target/author-qa，本轮无Web变更和浏览器复跑。
- 复用的隔离容器brioche-author-activate-1791288897837已docker stop，未改变用户开发/生产服务；沿用上一轮删除被自动审批拒绝后的保留策略，没有再次尝试删除。baseline新增失败schema author_runtime_1791289304716528400及其临时brioche-media目录仍保留，成功测试资源依现有teardown清理。完整作者语义、人工教学审校/正式录音、真实设备及生产门槛继续保留。

## 2026-10-06：账号步骤确认与完成提交的等待焦点

- 新增实际Learning/MemoryRouter/受控HTTP Chromium回归，覆盖steps/read PUT与complete POST两段503→原提交重试→确认成功。修复前步骤提交刚进入saving就因原生disabled失焦，实际焦点断言false!==true，后续流程未运行，不把未执行部分计入baseline证据。
- 学习页步骤确认/完成按钮的等待状态现使用aria-disabled/aria-busy并保留焦点；未满足步骤/题目条件、初始草稿恢复未完成仍原生disabled。useLearningSession显式暴露restored供控件区分初始化和异步等待，原blocked/pending/busy/stale检查不变，完成按钮也显式核对blocked。重新读取/重试/已完成页待确认按钮保留等待焦点，仍由hook的同步busy锁保护。
- 两项定向浏览器62.16s通过：每个初次提交重复Enter只有一请求，pending焦点/busy=true；503后原按钮保持聚焦，重试路径/方法/完整body含idempotencyKey均相同；完成返回成功后才显示本课已完成并标题聚焦，pending清空；既有学习离页/返回原请求恢复也通过。装配记录真实privateRequest目标路径及method，没有假装写入真实数据库。独立随机Vite/Chromium已自动清理，无未捕获页面错误。
- 28Web/12SSR、TS7、SSR/client build及目标格式/diff通过，本轮未重跑现42项全组件浏览器或三项生产页面壳。固定b19b763的Check37463008841本轮读取in_progress，不能称终态成功。更多跨步骤恢复/冲突组合、真实数据库与设备、完整内容人工审校/正式录音和生产门槛继续保留。

## 2026-10-06：多步骤确认恢复后的推进

- 补合成的阅读→回顾两步骤账号课程，实际Learning/MemoryRouter Chromium在步骤PUT收到503后确认离页、返回读取sessionStorage原请求、重试200。修复前pending已清除但标题仍是阅读，实际断言阅读!==回顾；原步骤onSaved回调不在持久化请求中，恢复无法调用它。
- useLearningSession现对精确本会话/课程步骤PUT且成功结果含对应confirmedStepId生成步骤确认（step ID + 原幂等key），页面layout effect按key只消费一次，推进到该步骤的下一步并沿用既有audio.stop、步草稿及标题焦点。正常步骤写入也使用同一确认路径，练习自身onSaved保持原逻辑；GET冲突恢复/失败/未确认结果不产生推进确认，已完成页不再推进。
- 三项定向Chromium72.92s通过（单步骤确认/完成503重试、恢复多步骤推进、原离页待确认恢复）。补正常推进断言后，多步骤用例再42.46s通过：恢复重试body/key相同、不追加第三写请求，标题回顾聚焦；回看阅读不会消费旧回执或自动新增请求，主动重新确认才以version2和新key发第三次PUT并再推进到回顾。没有假定响应丢失必然已commit；装配是受控HTTP，不代替真实数据库幂等证明。独立随机Vite/浏览器已自动清理。
- 首轮TS7发现泛型结果局部变量推断为Result，显式标注LearningState恢复上下文收窄后类型通过；该类型声明无运行时变化。28Web/12SSR、client/SSR build、目标格式/diff通过。现43项组件浏览器及三项生产页面壳本轮未全量重跑；多账号/冲突组合、真实设备、人工内容/正式录音与生产验收继续保留。

## 2026-10-06：账号进度冲突后的读取与撤回收敛

- 补真实Learning/MemoryRouter的受控GET学习记录适配器及多步骤浏览器回归，不修改生产应用逻辑。步骤PUT返回409后自动GET；GET503保持当前阅读和重新读取入口，被明确拒绝的原pending已清除。原生Enter重新读取，等待期间重复Enter不额外GET或PUT；成功GET带version7/已确认read，只更新进度，不生成步骤推进回执。用户主动再次确认才使用version7/新幂等key发PUT，返回version8后标题推进回顾。
- 扩展至第二步骤的PUT409→GET410：原正文/learning-actions消失，当前会话的所有sessionStorage草稿清理，课程已撤回标题聚焦；总3 PUT/3 GET，没有因撤回重发写入。首次冲突用例39.59s通过，扩展撤回后的最终用例43.57s通过，独立随机Vite/Chromium自动清理、未捕获页面错误为零。该证据是实际组件/键盘与受控HTTP，不替代真实多账号/PostgreSQL冲突或撤回、iPhone及辅助技术验收。
- TS7及目标格式/diff通过。仅修改测试装配，本轮没有重新构建生产应用或重复28Web/12SSR，现44项全组件浏览器及三项生产页面壳未全量复跑。固定b19b763的Check37463008841本轮已实际确认completed/success，不能扩展为最新df8e78e或本轮提交CI成功。完整内容审校/正式录音、更多账号/设备与生产门槛继续保留。

## 2026-10-06：工程入口文档与完整CI核对

- README仍声明学习进度/账号复习尚未实现、所有命令均为后续目标，docs索引也仍自称v0.1提案。对照当前代码、工程说明及验证记录后校正入口：说明已接入学习/续学、练习、收藏复习、概览、发布/回滚/撤回和媒体预览；明确人工审校、正式录音、真实iPhone与生产验收未完成，链接08/09/07作为实现和证据入口。
- 索引新增当前Rust生成的public-lesson.schema链接，将早期Schema和阶段进展明确标为历史参考，保留原规划范围与原始记录。两个入口的本地Markdown链接均实际检查存在，diff检查通过。本轮只改文档，不追加应用测试或重新构建。
- 固定42bc05d的Check37465072084/job112273939244已实际读取in_progress，先rustup show、随后推进到PostgreSQL identity测试，观察无failed步骤；仍未到终态，不冒充完整Linux浏览器成功。该确切句柄继续待确认；整体目标和剩余人工/设备/生产门槛不变。

## 2026-10-06：组件浏览器测试的草稿隔离与弹窗收敛

- 固定42bc05d的Check37465072084/job112273939244实际终态为completed/failure，组件浏览器44项中42通过、2失败：进度冲突用例等不到阅读步骤的继续入口，待确认保存用例发现残留qa-session的step草稿；后续生产页面壳步骤未运行，不能宣称完整CI成功。
- 新增Learning多步骤弹窗回归，覆盖步骤PUT成功、PUT409后GET503、PUT410：各自在离页确认弹窗打开时返回结果，检查弹窗关闭、路由留在原页、焦点落到回顾/阅读/撤回标题、pending清除且不追加写入。最初共用sessionStorage的循环中成功分支留下回顾步骤，下一个冲突分支实际焦点为回顾而非预设阅读。早期等待阅读超时曾被误判为应用焦点问题；临时PendingNavigation修改没有解决，最终已完整撤回，不把测试污染记为产品缺陷。
- 独立测试的beforeEach现清理本套件拥有的Chromium sessionStorage；新用例各结果之间也隔离草稿。同一用例内保持真正的离页/返回/原提交恢复，未放宽正文、焦点、请求body/key或账号隔离断言。四项顺序定向回归85.43s全部通过，包含恢复多步骤、冲突读取、新弹窗结果、待确认保存。
- 格式化后的完整pnpm test:browser实际45/45通过（642.11s），没有未捕获页面错误，独立随机Vite/Chromium自动清理。共享弹窗的学习/复习离页、复习撤回、折叠收藏、独立表达待确认、身份替换等均在完整套件中通过。目标Prettier及diff检查通过。本轮最终只改测试和协作/验证文档，没有运行时改动，未重复构建、TS7、28Web/12SSR或三项生产页面壳；修复提交的完整Linux CI须另行确认。
- 受控浏览器证据不替代实际数据库/多账号并发、真实iPhone/辅助技术、法语内容人工审校、正式录音及生产环境验收，完整目标仍未达成。

## 2026-10-06：最终 Docker 启动要求与部署说明校正

- 用户明确要求最终在Docker中启动，已写入AGENTS及部署/工程说明：收尾后实际Compose build/up，核对migrate退出0、四个长期服务healthy、HTTP30075和巡检，最终保留应用运行。此要求不授权修改外部HTTPS/DNS/路由器。
- 对照当前main/config/Compose/.env示例移除部署文档的旧提案变量SESSION_COOKIE_KEY、CONTENT_SOURCE_ROOT、REGISTRATION_MODE，补实际APP_ENV/CONTENT_MODE/API_BIND/ADDITIONAL_APP_ORIGINS/POSTGRES_PASSWORD及origin用途。核对当前Compose up帮助支持wait/wait-timeout、巡检路径及.env忽略规则；生产空目录可启动供私有预览，不以启动代替内容审校。
- 当前根.env不存在，docker ps未见brioche应用栈；本轮没有创建秘密或启动/停止容器，最终启动仍待执行，不用先前隔离演练冒充运行状态。文档/diff检查通过，未改运行代码、不追加应用测试。

## 2026-10-06：知识笔记卡片、无箭头操作与本机 Docker 运行

- 按用户新反馈将解释/文化/词汇/语法/日常提示统一为LessonNote笔记卡片：分类色签、暖橙/淡蓝/淡绿纸卡、层叠边缘、圆形SVG加号旋转、内容到达动效和品牌色键盘焦点。沿用原生details/summary，不隐藏标题或把说明改成操作教学；练习回顾的日常卡也复用。reduced-motion禁用过渡及展开动画。移除全部Web操作入口的长箭头SVG和对应Icon分支，首页复习入口改为简洁文字下划线交互。
- TS7、生产client/SSR构建、28Web/12SSR及三项生产页面壳67.08s通过。独立生产SSR/受控公共API的一次性QA44.19s覆盖320/390/768/1440原生Enter/Space展开收起、标题保留焦点、无横向溢出、390px语法卡、reduced-motion、无箭头练习入口与原生链接跳转，未捕获错误为零。已查看390px折叠/展开截图，临时QA/截图在.local并被忽略，服务器/Chromium已关闭。第一次借独立浏览器读取开发页虽完成原生卡片检查，但记录到客户端动态模块加载错误，不计完整通过；后续该模块HTTP200，未重启用户开发服务。临时生产QA搬到.local后的两次模块导入装配错误也不计产品证据。本轮未重复全45组件浏览器，完整设备/辅助技术继续待验。
- 原测试隔离提交7dfe94d的完整Linux Check37468521427/job112285595048实际completed/success，不能扩展为本轮界面提交CI成功。
- 按用户要求实际构建并启动Compose project brioche，部署参数使用独立.local/docker.env，系统随机数据库秘密不输出、不入Git，Git/dockerignore均排除该文件，不覆盖开发.env。当前本机LAN origin及localhost/loopback额外允许origin仅在本机配置。开始前确认无brioche项目容器/卷和HTTP30075占用，未停止其他应用。
- Docker build实际成功；API/migrate使用同一镜像sha256:653f883bc9a409b1016e0a408e2605849b37805922a0e46526981b6c1a394db4，Web镜像sha256:9edae829b48d3014fc268accd60502ea4dc4250f8a846fcc5ab125c2d7a84a0c。up --wait实际完成，migrate exited0，PostgreSQL/API/Web/Traefik四个长期服务healthy、零重启；只Traefik发布0.0.0.0/[::]的30075→8080，无TLS。API/Web/Traefik非root，数据库/API/Web无宿主端口。
- HTTP loopback及本机LAN origin各执行health:check：五个服务状态及/api/health、/api/ready、/health、首页均healthy/exit0。容器与新数据库/媒体卷保留运行，此次不是启动后清理的隔离演练；实际手机跨设备/公网与认证学习全链路尚未验收。生产目录为空，没有发布48课draft或伪造审校/素材/录音授权。首个管理员邮箱已向用户询问，尚未签发邀请；人工内容/正式录音、真实设备及完整生产门槛继续保留。

## 2026-10-06：金额语法说明修正与 Docker 静态资源核对

- Docker brioche巡检再次五服务/四HTTP healthy、零重启；实际profile SSR引用/assets/root-D_LCGZua.css，经Traefik读取含note-card、note-card-toggle和reduced-motion规则，证明运行栈提供本轮新样式。正式/api/catalog levels仍为空，不把样式可用当作课程发布。
- 辅助审阅发现a1-bakery-pay的grammar-price-euros与explanation-main重复写“超过一欧元”即复数；依OQLF小于二数量后名词单数及法兰西学院un euro/des euros核对，现两处同步改为1,20 euro / 2 euros并说明边界。本次只修这两处价格解释，不冒充完成四课或全48课人工审校；报告与来源见content/a1/bakery-price-review.md。editorial保持draft，无导入/发布/数据库变化。
- 实际作者CLI check付款课通过，13项curriculum全部通过，包含全A1/A2结构/共享词汇/144题判分与素材清单检查；未改运行时代码，没有重复Web/SSR或Docker重建。付款草稿位于作者目录，不嵌入当前生产镜像的fixture。目标JSON格式与diff通过。
- 固定界面提交ea4f99e的Check37470856262/job112293640020本轮实际in_progress，读取到cargo clippy步骤且未观察失败；不是终态成功。首个管理员邮箱仍待用户回复，内容/素材授权/正式录音、真实设备和生产门槛继续保留。

## 2026-10-06：A1语法说明与共享词汇辅助审阅

- 按full-A1清单实际读取24课48条grammar说明/例句（前三与后三单元分别读取），检查基本变位、冠词介词、复数、否定提问、时间与计划的说明边界。辅助审阅范围和修改依据记录于content/a1/grammar-review.md；不是全部正文/文化/难度/录音或人工审校验收。
- social-introduce-friend的grammar-mon-ami不再模糊写“省音相关搭配”，依OQLF明确元音/哑音h开头阴性名词使用mon（mon amie），并增加与Mon ami对应的可朗读例句。word-aimer的食物场景限定错误套用到交友课，现购物清单/交友课两处改为同一通用说明，保持共享ID及既有判分不变；依据法兰西学院词典。
- 两个课源实际作者check通过，13项curriculum通过；补可朗读例句后再运行对应check和13项课程测试、目标Prettier/diff全部通过。检查全部A1源仍24个draft，没有导入/发布/修改数据库；未改运行代码或重建Docker，现有应用栈继续保留。
- 首次PowerShell字符串命令因弯撇号触发解析错误，没有执行任何修改；改用apply_patch完成文本编辑，该解析错误不计应用缺陷。固定ea4f99e的Check37470856262/job112293640020本轮两次读取均in_progress且处于组件浏览器步骤，未观察失败、不宣称完整CI成功。人工审校/素材录音、管理员邮箱、真实设备与生产门槛继续保留。

A2语法辅助审阅（2026-10-06）：读取24课48条grammar说明/例句，修正三课共享顺序词说明对过去叙述/操作步骤的不适配，以及两课共享devoir说明的家务范围；共改四课，仍draft。四份作者check、带sources的完整48课release检查、13课程测试与JSON格式/diff通过；初次重复sources flag调用错误已修正。报告见content/a2/grammar-review.md。ea4f99e完整LinuxCI37470856262/job112293640020本轮已实际确认completed/success，不作为后续内容提交CI证明。当前Docker继续运行；管理员邮箱、完整人工审校/正式录音、真实设备及生产门槛仍待完成。

导入前语义定位补齐（2026-10-06）：新增回归实际复现必需教学文本等非法内容在check可定位、import却先进入环境检查；现PublicLesson.validate_intrinsic统一课程自身的ID/引用/教学文本/步骤/练习等检查，import连接数据库前复用。audio/登记描述仍延后补齐，完整validate与发布媒体校验保留。38server单元、25作者CLI、13课程、19公共契约及Clippy/fmt通过，生成契约无差异；未改HTTP契约或审校/发布数据。Docker API/migrate已按当前源构建并up --wait更新，迁移exit0、四服务healthy，loopback/LAN HTTP30075巡检均通过、零重启；设备、正式内容/录音与生产门槛继续保留。

本批运行API/migrate实际inspect为同一镜像 sha256:88d91a6124b759c80b24ce88fe7319fccbb537222ebc57c476d4ab572ffee9d0；Web/Traefik/PostgreSQL保持原服务，未改外部DNS/HTTPS或用户开发服务。

导入前私有规则引用校验（2026-10-06）：回归复现check可定位未知正确选项/排序语块，import却先报环境错误；现import前置语义检查对原课源调用同一Grader::from_author_source，核对规则与真实练习块及非空反馈，不输出私有答案。补三项import字段场景，38server单元、25作者CLI、13课程与Clippy/fmt通过；完整48课离线目录通过。本轮未改公开契约/Web/课程数据，未重跑此前19契约/浏览器或PG事务；实际Docker API/migrate已构建并up --wait更新，迁移exit0、四服务healthy，loopback/LAN HTTP30075健康、零重启。eb7fd8f固定提交CI37473650814本轮实际in_progress；完整人工内容/录音、真实设备及生产门槛继续保留。

本批API/migrate实际inspect镜像一致：sha256:81beb61fa6aef7c080c3c74797bef4c59af72c1b7a0a392b7933c9afdf22c6f7。现有生产目录仍空，未导入或发布草稿。

导入修复数据库复验（2026-10-06）：当前1e29f5d实际源码使用既有独立QA PostgreSQL容器，随机隔离schema运行author_runtime ignored集成1项通过。覆盖真实作者CLI合法登记/导入、失败定位、批次staging/激活/撤回及失败事务边界，证明新前置检查没有阻断合法登记描述替换流程；非当前应用数据库或正式课程验收。QA容器已停止，既有失败baseline保留，不尝试此前被拒的清理。补03作者前置/完整媒体检查职责说明。eb7fd8f CI37473650814/job112303321065实际仍组件浏览器进行中；1e29f5d CI37474160289/job112305087514实际workspace test进行中，fmt/clippy已成功。完整内容/录音、设备与生产门槛继续保留。

2026-10-07 原生逐词对齐迁移验证：官方HF转换Qwen3-ForcedAligner-0.6B-hf固定revision及6个SHA-256文件通过；Transformers5.19、CPUtorch2.14.1+cpu、Accelerate1.15等58精确依赖，OSV查询0告警。移除旧qwen-asr包装与Web UI依赖，原生推理不调用上游时间插值；直接保留80ms量化分类及异常。Python3.14.2和独立3.12.12各15标准库用例、最终Rust20契约/51server及工作区常规/Clippy/fmt、真实隔离PG后台4项通过。初次复制模型清单保留旧mtime导致Cargo未重新嵌入内容，旧revision拒绝测试收到200；touch清单并重新编译后4项全过。专用PG容器及匿名卷已删除，Web无变更，未重跑浏览器或新增迁移。

完整8段旧法语试听命令完成，7段范围检查通过，1段保留实际重叠（前词0.72秒结束、后词0.56秒开始），输出始终reviewRequired且退出码2。首次处理器参数出现弃用提示，改用processor_kwargs后完整复验，预测与首次完全一致，无弃用提示。私有测试包不等于真实生成/人工审听回执，未据此登记音频或发布课程。原hf-xet下载进程经同句柄确认live；官方HTTP206字节范围及4MiB读取速度实际验证后，明确改用私有有界分段下载，原进程定向终止并确认exit1、新下载核对6SHA后exit0，未因单纯观察timeout盲目重启。所有下载/推理/测试句柄现均终态。

实际部署：数据库及5媒体backup/verify通过；server镜像重建，以HTTPS+TTS三个Compose文件up--wait，四长期服务healthy、migrateexit0，两HTTPS health/ready/Web/首页巡检healthy。实际API镜像dd0fa2496bde0944f4efaad7f2c388543e1c655be1ce11dc69001616dfc91bb2；Web保留e43ee1a472991e9551ae8ee7608a88866ef1623eabc7afa820da61c365ebc300。Traefik无宿主映射/30075零监听；生产只读24迁移/音频0/试听0/声音2/published6/generation1/计划0/片段0/对齐0/审核0保持。没有付费调用、账号变更、人工批准或新课发布；正式六课语音覆盖、录音包组装与新release及后台分页/运维/真实设备验收仍待完成，完整目标保持active。

## Chef 后端抽取实际验证（2026-10-08）

Chef `900375c` 实际完成共享后端/迁移/CLI/回归测试抽取，`brioche-courses` `7a84c18` 固定原例子与跨平台 LF。依赖按 Brioche 原锁文件保持版本；第一轮缺少课程 example、第二轮旧 checkout SVG 换行导致失败，固定课源 pin 与精确字节后最终工作区20契约+59引擎单元+25作者CLI+13课程+2配音计划测试通过，Clippy全部targets与公共导出diff通过。隔离 PostgreSQL18.6 实际11项 ignored回归全部通过，覆盖admin4/author_runtime1/identity1/learning3/postgres1/recording1，测试库容器已停止清理。没有真实生产账号修改或收费调用。Chef远端CI37654153831实际completed/success。

Brioche `1cceb05` 实际推送：移除原迁移/业务/测试副本，Rust仅保留薄启动和Schema导出；95文件变更、30552行删除。固定Chef与课程递归子模块在全新远端checkout取得，Cargo metadata仅本产品启动member，不依赖相邻本机Chef。最终产品fmt/workspace test/Clippy/生成diff通过，固定子模块工作区常规测试再次通过，TS7及28Web测试通过；只改合成录音fixture引用，未重跑完整浏览器，不扩大为iPhone验收。

DockerAPI实际release构建完成，HTTPS+TTS三覆盖up server --wait实际healthy/0restart。当前实际Image `sha256:aad948cf294f73d1ff53cdec70b140777ca13bc2311642785e907fbe8a3b986c`；Web保持旧镜像。release-status仍48课正式录音目录/generation10，五service/四HTTPhealth健康，D盘空闲420444209152bytes。两域目录48、courses/ready200、匿名admin401/no-store通过。没有数据库schema/内容/账号迁移、DNS更改或恢复30075；独立身份与双产品隔离及Web抽取继续进行。Brioche新CI37654716488当次in_progress，前3bfcd84及4c657c9真实green，不将旧CI结论推广新提交。
两HTTPS全部48固定revision公开课源进一步实际读取200、id/revision与目录一致、无serverOnly、48/48正式audio字段存在，证据.local/private/chef-backend-20261008/all-courses-verified.json。本次只核对课源/录音描述，没有重新下载407音频或声明人工听感。

## Chef 共享 Web 抽取实际验证（2026-10-08）

Chef 真源已推送至 `1f285538b83a89f7e5bd41e64b2704a385abaf2e`，Brioche 实际产品提交 `cb94ea32bf524305ce8c5e54623560ed605c8526` 固定引用它。通用路由、学习/复习、播放器、身份界面、管理员界面及通用 Web 回归已迁入 Chef；产品保留品牌配置、主题、图标、入口和部署装配。本次产品253文件变更、31320行删除。原148个契约文件与固定框架提交逐字节一致后删除副本；依赖沿用原锁文件外部版本，没有技术栈升级。

最终使用 React Router 外部 appDirectory 与 TypeScript rootDirs，Vite 仅配置产品别名与 React 去重；未保留试验中的 preserveSymlinks。初次外部类型生成/fixture目录深度配置失败已修复，独立 Chef fixture 和全新远端递归产品 checkout 的 frozen install、TS7、SSR build 均实际终态成功，未依赖邻接本机仓库。Chef 与 Brioche 均最终通过28单元及29 SSR测试。Brioche实际5项SSR浏览器回归通过，覆盖后台/上传/手机外壳/中等宽度阅读；最终12项精选交互浏览器回归通过。首次12项中部分录音错误toast因慢速并发操作消失导致1失败；单项重跑通过，改为观察实际DOM toast消息后最终12项全部通过，未放宽媒体/发音断言。Chef独立fixture修复图标路径后实际1项SSR浏览器外壳测试通过。不是完整浏览器或真实iPhone验收。

最终 Docker Web image 为 `sha256:364b5cd844385a5379c53fb246b65ae02ca9c2eb4be2033dcdd56db78d163148`，HTTPS+TTS三覆盖 up --wait 已实际 healthy/0restart。API保持 `sha256:aad948cf294f73d1ff53cdec70b140777ca13bc2311642785e907fbe8a3b986c`；五服务/四HTTP healthy，release仍48课正式录音/generation10。两个既有HTTPS入口首页、目录、品牌SVG、CSS均200，匿名admin401/private no-store，目录各48课。实际390×844匿名Chromium首页截图已查看，品牌与主卡片无可见溢出。没有内容/媒体/账号/数据库迁移、付费生成、DNS或不安全端口变更。证据仅在本机私有目录 `.local/private/chef-web-20261008`。

远端 Chef CI37658166111 与 Brioche CI37658317014 在本次收尾读取时仍in_progress，不能声称这两次完整CI已通过。独立身份服务、按产品隔离数据库/授权/草稿、语言中立契约及粤语课源/Hargow实际入口尚未完成；课程与运维源码继续迁出产品，原设备/异盘恢复验收仍待完成。

## 独立课程仓库消费验证（2026-10-08）

Brioche `a7bb574` 实际公开推送，151文件变更、138883行删除。原134份content及5份example共139份文件的Git对象哈希与固定课程仓库逐一一致后移除，定位README替代旧路径。产品直接固定 `curriculum` e61e0f1759de66d768616c7a4ddda01b40d71f1b 和 Chef78f33e874ed8ecf3b2981de22d8c3f92eceacb3a；课程repo新增独立CI，框架提供通用精确release解析工具，产品CI与文档作者命令更新，不依赖Chef兼容样本作为正式课程。

第一次向旧CLI传递content父目录，实际失败且退出1：CLI本身不递归，也不能从多份历史源猜测revision。新增共享工具递归选择精确id/revision，缺失版本、不同内容的同版本与错误目录归属都拒绝；聚合到临时目录后调用实际Rust check-release --sources，退出后清理，不读写数据库。选择器实际回归验证旧版不能满足新版、完全一致副本可共存、不同副本/归属拒绝。产品本地实际48精确revision通过；全新远端递归checkout再次实际48通过、gitclean。原课程发布/媒体/账号没有重写，私有证据记录48来源哈希，不含作者答案。

此前共享Web的完整远端CI已实际读取completed/success：Chef37658166111的contracts和web两job、Brioche37658317014的check全部通过。这证明对应1f28553/cb94提交，不能推广为新78f33e8/e61e0f1/a7bb574已绿。

课程CI首次37660495743失败在第二次checkout（Git退出1，构建/校验未执行）；改为完整Chef SHA并在其目录运行固定Rust工具链，课程caceab58e4d6244f58ccf4be2367dce4416e7263的独立CI37661255990实际completed/success，checkout、Linux构建与全48验证均成功。产品a95bf534cbcd02552ead23eac967bf9334411369已公开推送并固定该课程提交。本地再次48及选择器回归通过。

产品API Docker在移除无用课源COPY后实际完整release重建成功，三覆盖up server --wait已终态healthy/0restart；实际API Image `sha256:65415f4c20466d96843c2857a438fba092c6bcc355d52b6b8ebc4811ea2ef28e`。release-status仍48课正式录音目录/generation10，2026-10-07T17:44:32.282Z五服务/四HTTP健康、D盘空闲415664394240bytes。Web镜像保持此前已验收版本。本次没有数据库迁移、内容导入、媒体/账号写入、提供方调用或域名改动。Chef78f33e8 CI37660317607及Briochea95bf53 CI37661317751最近读取仍in_progress；课程CI成功不代表另两全套CI已通过。

## 共享运维/TTS/对齐工具抽取（2026-10-08）

Chef af1789a6a7e74ea2dc22d0da23ee60ad1e620016 已实际推送，通用备份/恢复/加密/健康/Qwen工具、离线对齐、模型runtime/aliases元数据及通用回归迁入框架。Brioche5a10b54实际推送，30文件+64/-4148；保留六个Node和两个Python兼容入口，QwenAPI仅转发export，没有业务副本。旧命令继续有效，import无副作用；Node错误和退出码来自实际共享子进程。对齐模型/输出工作区默认cwd，可CHEF_WORKSPACE_ROOT指定，固定模型元数据来自框架源码，产品JSON副本与无用Docker COPY移除。Docker排除Python缓存，未复制任何私有模型、TTS环境、录音/备份或未测试retention草稿。

Chef和固定产品子模块实际通用Node回归31通过、1 Docker跳过；另外显式开启隔离Docker演练，真实大二进制数据库/媒体备份恢复及不安全目标拒绝1通过（32.243秒），全部演练资源按随机UUID自动清理。Python对齐19项在独立Chef和产品调用工作区均通过，覆盖真实源哈希/原始预测/损坏模型/越界输出，未运行收费合成或下载模型。产品兼容回归实际1通过，确认旧import/export、凭据隐藏及精确退出码；旧Python --help实际成功。全新远端递归5a10b54检出包含fixedChefaf1789a/coursecaceab5，兼容回归与Python入口再次通过、gitclean，不依赖邻接仓库。frozen offline pnpm安装通过，没有外部依赖升级。没有Web或Rust业务变更，不把工具检查当语言/设备验收。

Docker入口补齐在产品17568ae另行提交推送；之前5a10b54尚含已删除产品JSON的COPY，不作为独立Docker构建完成证据。最终工作树三覆盖API release build已实际终态成功，up --wait healthy/0restart，实际Image `sha256:5960df7e2b56894f166ba16b45ef89d22980d66cda11297b0ad1858743d76fd0`；Web保持既有364b5c镜像。旧产品health入口实际转发到框架巡检，2026-10-07T18:02:51.384Z五服务/四HTTPhealthy，Dfree415643598848bytes，release仍48正式录音/generation10。无数据库迁移/生产数据写入、DNS或不安全端口更改。Chef af1789a 的完整CI37662516117实际completed/success，contracts与web两job均成功；单独Graph Update成功不代替完整CI。全新远端17568ae Docker构建另外执行，终态另记。

远端独立checkout17568ae的Linux Docker构建实际终态0，验证镜像434cc719仅用于构建核对、未切换生产。与生产实际brioche-server二进制SHA256逐字节相同：14046506c869971f595c84902cfb48a916dbdf62307466376665be5ccb1e3fb4。两既有HTTPS目录实际各48，checkout gitclean。Brioche17568ae最新完整CI37663681970仍in_progress，不把Chef全绿推广至本产品。

## 独立身份进程隔离验证（2026-10-08）

共享框架Chef 4feb66d2586e926f37aa9dad3f56b592629ee863 actual normal push成功。新增独立identity二进制/镜像和固定产品服务器会话，不迁移生产。新PG测试使用任务自建loopback随机端口专用库，既有11项及新增1项实际全部通过：共享账号/两产品Cookie、跨产品重命名拒绝、实时角色、单产品logout、全局reset、旧会话只允许Brioche、范围save/delete拒绝及过期会话。Rust工作区20契约+61lib+25CLI+13curriculum+2speechplan通过；Clippy所有目标与fmt通过；Schema/TS再导出无生成diff。首编译digest类型错误修正、首新测试误断言删除Cookie必须HttpOnly后改为只检查有效Cookie；最终完整测试通过，不把先前失败计成功。

独立Linux Docker release构建终态0，镜像sha256:d00d4b8186e706498cc13260b3c5a84cf5930f50f931009f863e150246b14b16；实际启动于隔离任务网络、非root chef、0restart。真实HTTP /health、/ready、/api/v1/auth/csrf为200；内省无客户端凭据与有效客户端匿名均401/private,no-store；不存在的学习/后台404。容器和网络按com.chef.task=identity-service-20261008核对后stop/removal全部成功，隔离数据库也清除，没有读取生产.env、账号或私有声音。产品生产仍使用旧已验证5960df7 API/364b5c Web，不改HTTPS入口、DNS、端口、课程发布及媒体。

账号-only响应不含settings/passwordHash，服务密钥仅digest且恒定时间比较，错误凭据/产品在数据库读取前拒绝（断开数据库unit实际通过）。底层仍复用旧users.settings解码和全局role；学习消费者、产品成员授权、学习设置与数据库角色隔离尚未完成。最新Chef完整CI37667231222和Brioche既有17568ae CI37663681970读取均in_progress，不能称其已成功。所有本机build/test/export/push/cleanup句柄已经终态；后续文档push需另确认。

## 产品学习设置脱离账号表（2026-10-08）

Chef 33b025365ffdf70ea9ab4ee23253eeb4a6fec13c已实际提交推送迁移28：旧users.settings及原version完整归入Brioche product_user_settings，复合键为product_id/user_id，每产品有独立设置版本；账号表移除学习设置，账号版本独立。Brioche旧组合profile接口仍保持v1载荷，settings修改只推进产品版本；复习时区、学习日历均改读产品设置。独立identity登录/邀请/账号/内省不再依赖或初始化学习设置。

13项实际专用PostgreSQL回归全部通过，新增真实27→28迁移/回滚、Brioche原设置/版本和密码哈希保留、Hargow默认独立/交叉不影响、同产品并发CAS及账号版本不受设置修改影响。存在Hargow设置时回滚明确拒绝，测试验证数据仍保留；测试中显式删除合成Hargow数据只为完成可逆性检查，不能作为生产回滚方式。学习设置表临时改名时账号/内省仍200；独立账号流程未创建任何产品设置行。Rust常规工作区/fmt/Clippy/生成diff通过。全量PG通过后新增identity表不可用断言，定向测试和最终Clippy再次通过。专用Docker PostgreSQL在任务标签核对后清理。

没有更新Brioche框架pin、没有迁移生产或重新发布课源。旧生产二进制仍读users.settings，必须与新API版本协调切换，禁止先执行drop列。Chef最新CI37668255797与前identity提交CI37667231222真实读取均仍in_progress。学习API私有内省消费者、产品角色/schema/最小权限、课程/目录/媒体及学习事实隔离、客户端范围与语言中立/Cantonese/Hargow入口仍未完成；目标保持进行中。

## 学习 API 内网身份接入（2026-10-08）

Chef42bcabb49dca09b7ac05e05fd27e559b0e975d65已实际normal push，learning_identity消费者连接独立账号内省，learning/reviews/library/dashboard共享提取已验证账号；旧本地路径兼容。服务器IDENTITY_INTERNAL_URL启用新模式，配置错误/失败不回退；固定Brioche，尚未隔离的Hargow业务路由拒绝。新模式不暴露本地登录/账号管理/后台，仍待产品权限、独立账号编辑/管理员接入与完整双服务部署，Brioche生产pin保持原值。

实际隔离PostgreSQL13项全部通过（35865的PG阶段），新增真实TCP调用独立identity Router：profile200、错误CSRF settings403/正确200、收藏200、另一产品Cookie401、关停上游后503；设置只更新Brioche不影响Hargow。独立HTTP测试覆盖错误产品/无效JSON/超大响应/503/有效JSON重定向拒绝/2秒超时/401、重复或缺失Cookie不外呼及不返回Set-Cookie。自动重试/环境代理/重定向禁用，32并发/4KiB响应有界，身份和权限不缓存；账号服务核验原方法+Origin+会话CSRF，学习不获取CSRF秘密。

首次HTTP测试因未安装rustls provider失败，补显式ring初始化后通过；PG全部通过后的Clippy因nestedif失败，修复后最终全workspace（20contract/62lib+1PGignored/25CLI/13curriculum/2speechplan）和Clippy/fmt/生成diff在59031终态0。最后命令启动分支防止无效Unicode配置或fixture模式忽略remote配置，随后fmt与全目标Clippy ad46e6终态0。隔离容器chef-consumer-tests-20261008在任务标签确认后stop/remove32be98终态0。没有production env/DB/账号/provider调用、域名/端口/dev变更，没有新的生产Docker部署，不把Rust Router TCP联调称实际完整生产Compose验收。

远端Chef独立identity提交4feb66d的完整CI37667231222已真实completed/success；产品设置33b0253 CI37668255797及最新consumer42bcabb CI37670082800仍in_progress，不推广前一全绿。当前学习Backend状态仍含旧密码服务类型、学习设置read仍引用users.id、legacy/admin仍同进程，schema/最小权限和完整移除本地identity依赖尚待实施。完整goal还包含所有业务产品隔离、语言契约、粤语真实课源/Hargow入口、真实设备及异盘备份运维验收，保持进行中。

## 产品成员授权与管理员权限核心（2026-10-08）

Chef46e51943c8efd6646dcebcd5d51463122edb14d8实际提交推送migration29/product_memberships与审计、独立身份成员读写API和内省membership必填字段。现有账号role只归入Brioche，Hargow不继承；旧B邀请在B入口接受时创建B成员，H入口不将旧global operator邀请变成H管理员。消费者profile现在读取产品membership.role。身份服务修改成员在account-admin锁内重新核验actor产品权限、固定expectedVersion、每产品lastoperator及真实actor/理由/版本审计；回滚拒绝丢弃范围授权或审计。

真实专用Docker PostgreSQL14项最终全部通过（46410终态0含Clippy），新增真实28→29迁移、Brioche原权限/默认Hargow、不继承globalrole、grant隔离、并发一个CAS成功、actor撤销后拒绝、最后管理员保护、5条实际审计与rollback拒绝。身份HTTP确认Hargow内省membership learner以及global operator修改H授权403。常规workspace/fmt/生成diff59958实际0（62lib pass+2专用PGignored、20contract/25CLI/13curriculum/2speechplan）；最终check/fmt/diff0。首全PG跑到identity-service回滚触发保护失败：测试之前修改合成globalrole但未恢复，补复原测试账号后完整14PG/Clippy重跑通过；不能将首失败当全绿，也未削弱rollback保护。product-settings测试固定只升级28，避免未来29被误当设置迁移回滚。

所有本机句柄终态，专用PGchef-membership-tests-20261008标签product-membership-20261008核对后stop自动remove（c65363终态0，包含failed合成schema一起清除）。没有生产迁移/授权/媒体/provider调用，无产品pin/Compose/域名/端口/dev变更。最新ChefCI37671868670以及consumer37670082800/settings37668255797真实读取均in_progress，不推广较早identity全绿。旧后台仍globalrole/同进程操作，独立后台与全schema/leastprivilege未接入；Hargow首管理员明确bootstrap尚待实施。后续继续后台/账号编辑管理UI、其余产品事实/目录/媒体隔离、语言中立与粤语Hargow真实入口，完整设备和异盘ops验收仍待完成。目标保持active。

## 独立账号资料编辑与公共契约（2026-10-08）

Chef18bbdbeea99ac5c38b2c1fb9398fcb3128eb399e已实际normal push，共享Rust AccountProfile/AccountProfileUpdateRequest/AccountAuthResult与生成TS真源，移除服务手写重复DTO及会混淆产品version的隐式From转换。独立身份/旧组合进程均提供GET/PATCH /api/v1/account，PATCH只接受displayName+expectedAccountVersion，账号ID来自真实session。姓名trim/非空/80字符/控制字符限制，SQL账号profile_version CAS，冲突409；不更改产品prefs/权限/密码。

14项专用真实PostgreSQL与HTTP回归完整通过（68792终态0含Clippy），新增B/H同账号昵称跨入口同步、两产品并发同expectedAccountVersion只有200/409、原产品设置version1不推进、身份流程prefs行数0、空白/超长/控制字符拒绝，以及settings/userId/role注入422。常规Rust工作区49574终态0（20contract/62lib+2专用PGignored/25CLI/13curriculum/2speechplan）/fmt/check均通过。Rust exporter94451终态0，只新增3份TS，无现有v1载荷变化；pnpm严格TS7 typecheck32660终态0。专用PG容器chef-account-profile-tests-20261008标签核对后stop/remove90e9f6终态0，无生产数据/env/provider调用/镜像部署/产品pin/域名/端口/dev变更。

当前共享Web仍是个人资料+学习日常组合编辑，未切到新账号API；新登录AccountAuthResult也需组合读取产品profile，前端交互及对应真实浏览器回归是后续工作，本次不宣称完整账号UI或双服务发布完成。产品后台事务权限与账户管理、完整schema/最小权限、其余产品事实/目录/媒体、语言中立/粤语Hargow、真实设备与异盘ops仍未完成。远端Chef最新CI37673375168和前成员37671868670/消费者37670082800实际读取仍in_progress，不能称其全绿；全部本机句柄终态，goal保持active。

## 共享账号编辑与产品设置拆分（2026-10-08）

Chef 093b70399cf3ba7fa7106f3b2a17b35b53b3e23f 已提交推送共享个人页的独立编辑：摘要入口只修改账号昵称，学习目标/时区入口只修改产品偏好。账号先 GET /account 再以 expectedAccountVersion 保存；学习保存不含 displayName，并继续使用产品设置版本。账号响应不覆盖产品角色/版本，迟到的设置响应保留新昵称。冲突/未确认写入只读回并保留草稿，要求显式重试；账号切换、会话丢失或卸载取消旧请求和迟到的 CSRF。

严格 TypeScript 检查、Web 单元测试、29 项 SSR 及最终串行 10 项个人页浏览器回归通过，包含两资源并行保存、版本/角色不混用、迟到响应、旧 CSRF 取消、过期登录、冲突恢复及离页/退出保护。新增测试首跑曾失败，修正测试的隐藏输入定位并停止在受测源码热更新时运行回归后，最终整组真实退出码 0。产品没有新增页面或账号实现副本，没有更改框架固定提交、生产数据库/镜像/入口。独立登录响应组合、后台身份/产品权限事务、完整产品事实与语言契约、Hargow 和生产双服务装配仍待完成，完整目标保持进行中。

前后端账号接口提交 18bbdbe 的 CI 37673375168、产品成员提交 46e5194 的 CI 37671868670 均已实际读取 completed/success；最新 UI 提交的远端 CI 尚未确认，不能推广旧 CI 结果。

## 产品SSR会话与独立登录验证（2026-10-08）

Chef c05b744004bc49ccc04c708dffc5bb24e185b13e 已实际提交推送。共享 Product 新增可信 sessionNamespace（brioche/hargow，缺省兼容 Brioche）；SSR 身份和私有读取只转发对应产品的精确会话 Cookie，重复、非法或超大值在网络请求前拒绝，身份读取 no-store。Hargow 配置的真实 SSR 模块与隔离 HTTP 测试证明另一产品 Cookie 不会授权当前产品。

修正此前登录待办判断：当前 Account 页面已忽略登录载荷并整页跳转，由根 SSR GET /me 获取产品资料，无需新增客户端账号/学习资料合并。真实构建浏览器测试以无 settings、global operator/version47 的账号登录响应建立会话，个人页正确使用产品 learner/version17，后续偏好保存发送17。现有跨账号替换页面/草稿保护也通过。最终严格 TS7、生产构建、30 单元、32 SSR 和2浏览器测试通过；所有专用服务/浏览器均已退出。该证据不表示 Hargow 真实学习 API、全部数据库产品隔离或生产双服务已经完成。无产品业务副本、固定版本更新、生产数据库/镜像/域名路由变更；完整目标继续进行中。
