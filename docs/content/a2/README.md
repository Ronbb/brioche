# A2 课程草稿包

状态：2026-10-06，「周末与出行」「一起生活」「办理日常事务」三个单元十二课作者草稿，均 `editorial.status=draft`。未导入、未人工审校、无正式录音，未 staging 或激活正式目录；其余三个 A2 规划单元继续待制作。

| 顺序 | 作者源文件 | 正文 | 交际目标 | 正文词数（空白分词） |
| --- | --- | --- | --- | --- |
| 1 | [安排一个周末](a2-travel-plan-weekend.lesson.json) | 计划短文，6 段 | 近期将来、活动顺序、出发准备 | 140 |
| 2 | [询问两晚住宿](a2-travel-book-room.lesson.json) | 预订电话，8 轮 | 入住/离店区间、晚数、早餐与报价确认 | 164 |
| 3 | [购买城际往返票](a2-travel-buy-return-ticket.lesson.json) | 售票对话，8 轮 | 单程/往返、去回时刻、直达与座位 | 189 |
| 4 | [讲述一次周末出行](a2-travel-tell-weekend.lesson.json) | 经历消息，7 段 | avoir + 过去分词、否定、已完成和计划的区别 | 171 |
| 5 | [分配家务](a2-home-share-chores.lesson.json) | 室友对话，8 轮 | devoir + 原形、负责与轮流分工 | 181 |
| 6 | [说明共同空间规则](a2-home-common-rules.lesson.json) | 共同约定，6 段 | il faut、il ne faut pas、事先通知 | 156 |
| 7 | [描述生活习惯](a2-home-shared-routine.lesson.json) | 习惯短文，6 段 | 频率副词、每周一次、重复星期安排 | 158 |
| 8 | [比较住处](a2-home-compare-rooms.lesson.json) | 看房对话，8 轮 | plus / moins + 形容词 + que、aussi…que | 208 |
| 9 | [预约服务](a2-services-book-appointment.lesson.json) | 预约电话，8 轮 | 礼貌请求、具体日期与时刻、取消方式 | 186 |
| 10 | [填写并核对信息](a2-services-fill-information.lesson.json) | 填表说明，6 段 | 姓名字段、联系方式、pour + 原形表示目的 | 162 |
| 11 | [解释遇到的问题](a2-services-explain-problem.lesson.json) | 借阅服务电话，8 轮 | parce que 表示原因、否定、已尝试的动作 | 201 |
| 12 | [询问后续处理方式](a2-services-ask-next-steps.lesson.json) | 资料咨询，8 轮 | 步骤顺序、devoir、收件与最终答复的区别 | 206 |

这四课是独立的虚构场景，不是一份贯穿四课的真实行程：住宿日期、周末往返时刻与一天出游经历分别供练习。每课有八个目标词汇/语块、两个语法点、解释和有范围的场景说明、单选/填空/排序各一题、可选生活任务与回顾。日期未指定年份，住宿正文星期几是虚构日历设定；价格、票程、直达、早餐与座位不是市场信息或运营方条款。A1 知识点复用原有固定条目，新增知识跨课使用同一 ID 和释义。

[联合草稿目录](catalog.pilot.release.json) 保留原 A1 六个单元 24 课，再追加 A2 第一单元 4 课，共 28 课、84 道题。独立 release ID 为 `a1-a2-travel-pilot-draft-v1`，不改变既有四份 A1 目录。完成草稿或结构检查不能宣称达到官方 A2 等级。

[两个 A2 单元的联合目录](catalog.two-units.release.json) 使用独立 ID `a1-a2-two-units-draft-v1`，保留 A1 24 课，追加 A2 八课，共 32 课、96 道题；旧 pilot 目录不变。「一起生活」四课也是独立的虚构情境，不要求把每篇的居住或工作安排当成同一时间线。每课同样有八个目标词条、两个语法点和三类练习。室友规则为协商设定，不是法律、租约或法语地区共同规则；住房报价不是市场信息。

[三个 A2 单元的联合目录](catalog.three-units.release.json) 使用独立 ID `a1-a2-three-units-draft-v1`，包含 A1 24 + A2 12 课、108 道题；前两份联合目录保持不变。「办理日常事务」为独立的虚构社区中心/图书馆情境，不代表真实机构流程或行政规则。日期未指定年份；“五个工作日”仅为练习设定，没有给出完整工作日历，不要求计算实际截止日期。表格采用短文说明，明确字段含义与用途，不新增交互式个人信息收集。

## 素材与角色

场景复用 [A1 原创城市场景 SVG](../a1/assets/city-morning.svg)，固定引用 `art-city-morning` revision 1；来源/哈希/尺寸记录继续使用 [原素材清单](../a1/scene-assets.bundle.json)。没有重复登记同一素材的新版本。角色复用 Camille、Luc、Léa 的 revision 1 快照：对话前独立介绍人物，接待员与售票员是本课情境身份；短文保留 narrator 和段落，不伪装成对话。

城市场景素材仍 planned、rightsConfirmed=false；头像素材与角色登记另按 [原示例清单](../../examples/asset-bundle.json) 和作者工具流程处理。引用文件已存在不等于授权和登记完成。没有添加 Web public 素材或绕过私有预览/发布门禁。

「一起生活」复用 [A1 原创室内 SVG](../a1/assets/home-morning.svg)，固定引用 `art-home-morning` revision 1，仍沿用同一场景素材清单、planned/未确认授权状态。四课均固定 Camille 和 Luc 的 revision 1 快照；短文由 Camille 叙述，正文按段落呈现。没有新增或重复登记素材。

「办理日常事务」复用 `art-city-morning` revision 1，代表城市里的日常办事场景；三段对话使用 Camille/Luc 快照与单课服务身份，填表短文由 Camille 叙述，另保留 Luc 的固定角色引用。素材登记/授权边界与原清单相同，不宣称已完成逐课插图适配或 operator 预览。

## 检查与发布边界

- 单课：`cargo run -p brioche-server -- check docs/content/a2/<lesson-id>.lesson.json`。
- 联合目录：`cargo run -p brioche-server -- check-release docs/content/a2/catalog.pilot.release.json`。
- 新联合目录：`cargo run -p brioche-server -- check-release docs/content/a2/catalog.two-units.release.json`。
- 三单元联合目录：`cargo run -p brioche-server -- check-release docs/content/a2/catalog.three-units.release.json`。
- 全包：`cargo test -p brioche-server --test curriculum`；八项检查核对两级目录及九个单元顺序、固定 revision/角色、A1/A2 共享知识一致、公开投影剥离私有字段，并使用正式 Grader 核对 108 道题的正确和合法错误答案；十二课 A2 正文检查 120–250 个空白分隔词和预期正文形式。

作者文件含 `serverOnly.grading`，不能直接发送给公共 API 或作为静态资源。正式预览需要先登记素材，再导入课程，由 operator 查看固定版本；当前不宣称完成四课的浏览器预览或教学审校。人工法语、中文译文、难度、文化范围、练习有效性、插图适配与录音仍待审校。正式发布继续受 reviewed 元数据、注册快照、授权与实际文件校验约束。

## 语言来源与待审校项

正文、译文、解释与练习为项目新写草稿，未复制第三方教材/题目/例句。参考资料只用于核对语言结构，不表示资料提供方认可或授权本课：

- [OQLF 复合过去时](https://vitrinelinguistique.oqlf.gouv.qc.ca/24218/la-grammaire/le-verbe/temps-grammaticaux/passe/generalites-sur-le-passe-compose)：核对助动词现在时 + 过去分词、已完成事件的表达，2026-10-06。
- [Larousse réserver 变位](https://www.larousse.fr/conjugaison/francais/reserver/8005)：核对现在时 réserver 的变位与复合过去时，2026-10-06。
- A1 复用知识的核对来源沿用 [A1 审校与来源记录](../a1/README.md)。
- [Larousse devoir 变位](https://www.larousse.fr/conjugaison/francais/devoir/3297)：核对 dois、devons 等现在时形式，2026-10-06。
- [Larousse falloir 变位](https://www.larousse.fr/conjugaison/francais/falloir/4605)：核对无人称结构 il faut，2026-10-06。
- [OQLF aussi 的用法](https://vitrinelinguistique.oqlf.gouv.qc.ca/21095/la-grammaire/ladverbe/emplois-de-aussi-comme-adverbe-et-comme-conjonction)：核对 aussi…que 的程度相同比较，与表示添加的 aussi 区分，2026-10-06。
- [Larousse pouvoir 变位](https://www.larousse.fr/fr/conjugaison/francais/pouvoir/6963)：核对 pouvez 及倒装 puis-je；pouvoir 没有命令式，2026-10-06。
- [OQLF parce que 与 puisque](https://vitrinelinguistique.oqlf.gouv.qc.ca/23501/la-syntaxe/les-conjonctions/emploi-de-parce-que-et-de-puisque)：核对 parce que 的原因表达与省音，2026-10-06。

「一起生活」待审校项：频率副词限定在本课简单现在时示例；ne faut pas 在规则语境中表示不应做，不能误译成“不必”。形容词比较仍保留阴阳性配合（lumineux / lumineuse），不把所有形容词比较都套用 plus。dont、celle、y、en 等正文支持表达有随文解释，仍需人工确认阅读负担和是否需要进一步拆分练习。住房比较的理解题使用明确给出的两项虚构金额，不评判学习者应选择哪套住房。

「办理日常事务」待审校项：Pouvez-vous 为礼貌请求倒装，puis-je、a-t-elle 等正式问法先随文理解；je l’ai utilisée 的阴性直接宾语配合有单句说明，尚未系统教学。pour + 原形表示目的，parce que + 句子说明原因，练习分别验证。Nom 在本表格指姓，不扩大为所有语境；adresse électronique 与住址区分。复用词条保留原知识 ID/释义，借阅 carte、表格 champ 和材料 pièces 的语境义另有说明，避免与其他课程词义混淆。

本单元不是完整过去时教程：prendre → pris、faire → fait 作为正文例形；明确不能把全部动词的助动词都设为 avoir。pas de souvenirs 的数量宾语否定和 comprend-il 的正式提问有随文解释，系统练习后续补充。待人工审校确认这些支持说明是否足够、八轮较长电话/售票正文是否适合目标学习者。
