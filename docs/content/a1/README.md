# A1 前三个单元：课程草稿包

状态：2026-10-06，12 课，全部待人工审校，未导入或激活线上目录。11 份新课与原来的面包店示例一起组成首版内容包。完整 A1 后三个单元与 A2 试点仍待制作。

每份 `.lesson.json` 是作者源文件，包含私有判分规则，不可直接作为 Web 静态资源或公共 API 响应。运行时通过现有 Rust 投影去掉私有字段；新增课程无需新增专属页面。`catalog.release.json` 只记录课程 ID、固定 revision 和顺序，不读取文件，也不证明课程已登记或已审校。

| 单元 | 顺序 | 源文件 | 交际目标 |
| --- | --- | --- | --- |
| 打开法语的一天 | 1 | [a1-greetings-meet](a1-greetings-meet.lesson.json) | 问候、介绍名字和道别 |
| | 2 | [a1-greetings-introduce](a1-greetings-introduce.lesson.json) | 居住城市和身份 |
| | 3 | [a1-greetings-repeat](a1-greetings-repeat.lesson.json) | 请求重复、放慢语速 |
| | 4 | [a1-greetings-spell-name](a1-greetings-spell-name.lesson.json) | 名字、姓氏和拼写 |
| 早餐与面包店 | 1 | [原有示例](../../examples/a1-bakery.lesson.json) | 买法棍、询价 |
| | 2 | [a1-bakery-order-coffee](a1-bakery-order-coffee.lesson.json) | 饮品与加奶选择 |
| | 3 | [a1-bakery-choose-quantity](a1-bakery-choose-quantity.lesson.json) | 商品数量与订单确认 |
| | 4 | [a1-bakery-pay](a1-bakery-pay.lesson.json) | 总价、银行卡与现金 |
| 在城市里移动 | 1 | [a1-city-find-metro](a1-city-find-metro.lesson.json) | 找入口、理解左右 |
| | 2 | [a1-city-buy-tickets](a1-city-buy-tickets.lesson.json) | 车票数量与用途 |
| | 3 | [a1-city-confirm-direction](a1-city-confirm-direction.lesson.json) | 确认列车目的地 |
| | 4 | [a1-city-read-departure](a1-city-read-departure.lesson.json) | 从短文读取时间和站台 |

新课均有 1–3 项目标、5–8 个目标词汇或语块、2 个语法点、解释/文化说明、单选/填空/排序各一题、可选生活任务和回顾。最后一课只用短文正文，其余新课使用对话；原有示例同时包含两种正文。角色复用 Camille、Luc、Léa 的 revision 1 快照，场景中的身份是课程情境设定。

同一知识 ID 保持完全相同的释义与说明，复习可跨课程去重。变位或复数出现在正文时，仍关联原形的词汇 ID；不是建立另一个独立复习词条。练习答案只保存在 `serverOnly.grading`。

## 校验与预览

对单课运行 `cargo run -p brioche-server -- check docs/content/a1/<lesson-id>.lesson.json`；目录运行 `cargo run -p brioche-server -- check-release docs/content/a1/catalog.release.json`。全包一致性运行 `cargo test -p brioche-server --test curriculum`，核对 3×4 编排、文件与目录对应、共享知识一致、投影去掉私有字段，并用正式 Grader 验证 36 题的正确答案和合法错误答案。

这些检查不访问数据库，也不证明法语教学内容正确。正式预览仍须按 [开发说明](../../08-development.md) 完成素材登记和课程导入；尚未登记的引用不能绕过发布校验。原有 development fixture 仍为单课，不自动替换成草稿包。

## 审校与来源记录

本包对话、短文、中文说明和练习为本项目新写草稿，没有复制第三方教材、题目或图片。以下资料仅供核对语言点与编排，不代表资料提供方认可本课，也不提供其内容的转载授权：

- [TV5MONDE 入门问候与自我介绍](https://apprendre.tv5monde.com/fr/exercices/premiere-classe/les-salutations)：参考问候、名字、字母与身份任务的组织，核对日期 2026-10-06。
- [Larousse pouvoir 变位](https://www.larousse.fr/fr/conjugaison/francais/pouvoir/6963)：核对 je peux / vous pouvez 的现在时形式，核对日期 2026-10-06。

每课以下项目仍未签核，不能把结构检查记录写成审校记录：

- 法语母语或合格教学审校者逐句核对自然程度、问候关系、名字拼写与时间读法；尤其核对 `Lé-a` 的教学呈现和字母 É 的表达。
- 核对中文译文、名词性别、变位/省音/复数、目标难度与前置知识；初学者实际试学确认 12 分钟估计是否合理。
- 确认单选只有一个合理答案、填空提示与反馈适当、排序语块能构成自然表达，正文足以支持题目。
- 价格、时间表、线路和角色信息均为虚构；不把场景设定写成法国普遍习惯或当前出行规则。
- 审校者、日期、源文件 Git commit、修改意见与结果应记录在本文件或专用审校记录，再将对应课标为 reviewed。之后修改需重新审校；已发布 revision 不可改写。

## 素材与录音缺口

| 引用 | 当前状态 | 发布前工作 |
| --- | --- | --- |
| art-first-conversations revision 1 | [640×470 SVG 源文件](assets/first-conversations.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| art-bakery-morning revision 1 | 已有仓库 SVG；示例素材包仍为 planned、rightsConfirmed=false | 确认来源授权后登记，不伪造确认 |
| art-city-morning revision 1 | [640×470 SVG 源文件](assets/city-morning.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| 三位角色头像 revision 1 | 已有仓库 SVG 和示例快照，授权仍待确认 | 核对并登记素材与角色快照 |
| 正式课程录音 | 尚未制作 | 法语审校后录制，记录授权、时长、哈希与正文时间轴，按 audio-check/audio-import 登记 |

新课使用显式 `assetRefs` 固定版本；这些引用不会使未登记文件可用。没有伪造音频或时间轴；缺录音时沿用浏览器法语声音回退，设备没有法语声音时仍可阅读。正式录音与真实 iPhone 验收继续推进。

两张新图源文件位于作者目录，未放入 Web public。它们的真实 SHA-256、MIME、尺寸、替代文本与来源记录见 [场景素材清单](scene-assets.bundle.json)，来源目录为 `docs/content/a1/assets`；清单保持 planned/rightsConfirmed=false，不能直接导入。图形由项目内 SVG 代码绘制，沿用品牌和已有角色外观，不含外部图片、字体或真实运营者标识。`asset-check` 与正式导入复用图片解码/安全 SVG 校验；`curriculum` 测试核对清单哈希与尺寸。浏览器已检查 390px 与 640px 显示，不替代真实 iPhone 或正式素材审校。
