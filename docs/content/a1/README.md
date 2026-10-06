# A1 课程草稿包

状态：2026-10-07，六个单元共 24 课。用户明确指示批准目录前六课，其 revision 1 作者源已记录 reviewed，详见[批准记录](../../10-admin-development.md)；其余 18 课保持 draft。此批准不声称独立专家审校、素材授权或线上发布；当前未导入或激活线上目录。A1 与 [A2 六个单元二十四课](../a2/README.md)的作者源已齐，完整联合目录包含 48 课。

每份 `.lesson.json` 是作者源文件，包含私有判分规则，不可直接作为 Web 静态资源或公共 API 响应。运行时通过现有 Rust 投影去掉私有字段；新增课程无需新增专属页面。[catalog.release.json](catalog.release.json) 保留前三单元 12 课，[catalog.extended.release.json](catalog.extended.release.json) 记录前四单元 16 课。[catalog.five-units.release.json](catalog.five-units.release.json) 记录前五单元 20 课。[catalog.full-a1.release.json](catalog.full-a1.release.json) 记录六单元 24 课。四个目录使用不同 release ID，只记录课程 ID、固定 revision 和顺序，不读取文件，也不证明课程已登记或已审校。

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
| 家与日常安排 | 1 | [a1-home-describe-room](a1-home-describe-room.lesson.json) | 房间、家具与位置 |
| | 2 | [a1-home-wake-up](a1-home-wake-up.lesson.json) | 起床、早餐、出门时间 |
| | 3 | [a1-home-work-study](a1-home-work-study.lesson.json) | 在家学习、开始与休息 |
| | 4 | [a1-home-weekly-routine](a1-home-weekly-routine.lesson.json) | 每周安排与否定表达 |
| 买东西与吃饭 | 1 | [a1-food-buy-fruit](a1-food-buy-fruit.lesson.json) | 水果、重量、个数与总价 |
| | 2 | [a1-food-shopping-list](a1-food-shopping-list.lesson.json) | 食材清单、不定数量与偏好 |
| | 3 | [a1-food-order-lunch](a1-food-order-lunch.lesson.json) | 人数、菜单、餐点与饮品 |
| | 4 | [a1-food-ingredients-bill](a1-food-ingredients-bill.lesson.json) | 询问成分、提出选择、用餐后结账 |
| 认识与约见 | 1 | [a1-social-introduce-friend](a1-social-introduce-friend.lesson.json) | 三位角色间介绍朋友、城市与喜好 |
| | 2 | [a1-social-invite-coffee](a1-social-invite-coffee.lesson.json) | 询问空闲、提出邀约、确认时间 |
| | 3 | [a1-social-confirm-meeting](a1-social-confirm-meeting.lesson.json) | 约见地点、出发时间与明日计划 |
| | 4 | [a1-social-reschedule](a1-social-reschedule.lesson.json) | 拒绝、说明原因、提出并确认改期 |

新课均有 1–3 项目标、5–8 个目标词汇或语块、2 个语法点、解释/文化说明、单选/填空/排序各一题、可选生活任务和回顾。城市末课、起床时间、每周习惯、购物清单和约见确认使用短文，其他新课使用对话；原有示例同时包含两种正文。角色复用 Camille、Luc、Léa 的 revision 1 快照，场景中的身份是课程情境设定。

同一知识 ID 保持完全相同的释义与说明，复习可跨课程去重。变位或复数出现在正文时，仍关联原形的词汇 ID；不是建立另一个独立复习词条。练习答案只保存在 `serverOnly.grading`。

## 校验与预览

对单课运行 `cargo run -p brioche-server -- check docs/content/a1/<lesson-id>.lesson.json`；目录运行 `cargo run -p brioche-server -- check-release docs/content/a1/catalog.release.json`。全包一致性运行 `cargo test -p brioche-server --test curriculum`，分别核对 3×4、4×4、5×4 和 6×4 单元顺序、文件与目录对应、共享知识与角色固定快照一致、投影去掉私有字段，并用正式 Grader 验证扩展包全部 72 题的正确答案和合法错误答案。扩展目录检查命令将文件名换成 `catalog.extended.release.json` 、`catalog.five-units.release.json` 或 `catalog.full-a1.release.json`。

这些检查不访问数据库，也不证明法语教学内容正确。正式预览仍须按 [开发说明](../../08-development.md) 完成素材登记和课程导入；尚未登记的引用不能绕过发布校验。原有 development fixture 仍为单课，不自动替换成草稿包。

## 审校与来源记录

完整包技术预览（2026-10-06）：A1–A2 共 48 份原作者课程在隔离 PostgreSQL 导入，固定版本私有读取、144 次课程范围内图片读取、144 道题各一组正确/合法错误答案共 288 次 operator 判分通过。48 页实际浏览器在 320/390px 完成渲染与 hydration 检查：无横向溢出、无重复 ID、图片成功加载，每页三个练习且没有收藏操作。A2 末课另用实际键盘完成选择/填空/排序，三个反馈正确并聚焦反馈。匿名读取作者课程全部 401、公共草稿读取全部 404；六类数据库计数（学习会话、尝试、复习卡、收藏、release、已发布 revision）均为零。素材仅以明确的隔离测试副本登记，原清单仍 planned/rightsConfirmed=false，无正式发布或人工审校结论。测试服务与数据库已清理，详细证据见 [实现与验收清单](../../09-implementation-tracker.md)。

本包对话、短文、中文说明和练习为本项目新写草稿，没有复制第三方教材、题目或图片。以下资料仅供核对语言点与编排，不代表资料提供方认可本课，也不提供其内容的转载授权：

- [TV5MONDE 入门问候与自我介绍](https://apprendre.tv5monde.com/fr/exercices/premiere-classe/les-salutations)：参考问候、名字、字母与身份任务的组织，核对日期 2026-10-06。
- [Larousse pouvoir 变位](https://www.larousse.fr/fr/conjugaison/francais/pouvoir/6963)：核对 je peux / vous pouvez 的现在时形式，核对日期 2026-10-06。

家与日常单元另参考 [Larousse se lever 变位](https://www.larousse.fr/conjugaison/francais/se_lever/5809)、[étudier 变位](https://www.larousse.fr/conjugaison/francais/%C3%A9tudier/4450)、[bureau 词条](https://www.larousse.fr/dictionnaires/francais/bureau/11702)，核对起床和学习的现在时、bureau 的性别与含义，核对日期 2026-10-06。例句与课程正文为项目原创。

购物餐饮单元另参考 [OQLF 部分限定词](https://vitrinelinguistique.oqlf.gouv.qc.ca/fiche-gdt/fiche/26559622/determinant-partitif)、[Larousse aimer 变位](https://www.larousse.fr/fr/conjugaison/francais/aimer/283) 和 [addition 词条](https://www.larousse.fr/dictionnaires/francais/addition/1014)，核对不定数量、喜好动词与餐馆账单用法，核对日期 2026-10-06。du riz 与 des tomates 的说明分别标明部分冠词与复数不定冠词；不是把所有 des 都归为部分冠词。

认识与约见单元另参考 [Larousse aller 变位](https://www.larousse.fr/fr/conjugaison/francais/aller/314)、[pouvoir 变位](https://www.larousse.fr/fr/conjugaison/francais/pouvoir/6963)、[OQLF 近期将来](https://vitrinelinguistique.oqlf.gouv.qc.ca/24122/la-grammaire/le-verbe/temps-grammaticaux/futur/le-futur-proche) 与 [mon/ton/son 在阴性名词前](https://vitrinelinguistique.oqlf.gouv.qc.ca/24157/la-grammaire/les-determinants/determinants-possessifs/mon-ton-et-son-devant-des-mots-feminins)。核对 je vais/elle va/elles vont、je peux/tu peux/on peut、aller + 原形和 mon amie 的规则，日期 2026-10-06。正文、例句与译文为原创草稿，查词不能替代教学审校。

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
| art-home-morning revision 1 | [640×470 SVG 源文件](assets/home-morning.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| art-fruit-market revision 1 | [640×470 SVG 源文件](assets/fruit-market.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| art-cafe-lunch revision 1 | [640×470 SVG 源文件](assets/cafe-lunch.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| art-park-meeting revision 1 | [640×470 SVG 源文件](assets/park-meeting.svg)已制作，清单仍为 planned | 核对画面、署名与授权后登记 |
| 正式课程录音 | 尚未制作 | 法语审校后录制，记录授权、时长、哈希与正文时间轴，按 audio-check/audio-import 登记 |

新课使用显式 `assetRefs` 固定版本；这些引用不会使未登记文件可用。没有伪造音频或时间轴；缺录音时沿用浏览器法语声音回退，设备没有法语声音时仍可阅读。正式录音与真实 iPhone 验收继续推进。

六张新图源文件位于作者目录，未放入 Web public。它们的真实 SHA-256、MIME、尺寸、替代文本与来源记录见 [场景素材清单](scene-assets.bundle.json)，来源目录为 `docs/content/a1/assets`；清单保持 planned/rightsConfirmed=false，不能直接导入。图形由项目内 SVG 代码绘制，沿用品牌和已有角色外观，不含外部图片、字体或真实运营者标识。`asset-check` 与正式导入复用图片解码/安全 SVG 校验；`curriculum` 测试核对清单哈希与尺寸。前两张图已在浏览器检查 390px 与 640px 显示；室内、水果摊、餐馆和公园图另在离线浏览器展示页检查 320/390/900px 无溢出，并查看 390px 截图；全部新图的正式画面审校仍待补齐，不替代真实 iPhone 或正式素材审校。

## 金额表达修订

[早餐单元金额表达辅助审阅](bakery-price-review.md)记录了付款课 euro 的单复数规则修正及依据。课程结构与判分检查通过不代表整个单元已完成人工审校，所有课源继续保持 draft。

[A1语法说明辅助审阅](grammar-review.md)记录24课48条grammar说明与例句的检查范围、mon amie规则澄清及共享aimer说明修订；不替代全课人工审校，也不更改draft状态。
