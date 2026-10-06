# A2 课程草稿包

状态：2026-10-06，第一个单元「周末与出行」四课作者草稿，均 `editorial.status=draft`。未导入、未人工审校、无正式录音，未 staging 或激活正式目录；其余五个 A2 规划单元继续待制作。

| 顺序 | 作者源文件 | 正文 | 交际目标 | 正文词数（空白分词） |
| --- | --- | --- | --- | --- |
| 1 | [安排一个周末](a2-travel-plan-weekend.lesson.json) | 计划短文，6 段 | 近期将来、活动顺序、出发准备 | 140 |
| 2 | [询问两晚住宿](a2-travel-book-room.lesson.json) | 预订电话，8 轮 | 入住/离店区间、晚数、早餐与报价确认 | 164 |
| 3 | [购买城际往返票](a2-travel-buy-return-ticket.lesson.json) | 售票对话，8 轮 | 单程/往返、去回时刻、直达与座位 | 189 |
| 4 | [讲述一次周末出行](a2-travel-tell-weekend.lesson.json) | 经历消息，7 段 | avoir + 过去分词、否定、已完成和计划的区别 | 171 |

这四课是独立的虚构场景，不是一份贯穿四课的真实行程：住宿日期、周末往返时刻与一天出游经历分别供练习。每课有八个目标词汇/语块、两个语法点、解释和有范围的场景说明、单选/填空/排序各一题、可选生活任务与回顾。日期未指定年份，住宿正文星期几是虚构日历设定；价格、票程、直达、早餐与座位不是市场信息或运营方条款。A1 知识点复用原有固定条目，新增知识跨课使用同一 ID 和释义。

[联合草稿目录](catalog.pilot.release.json) 保留原 A1 六个单元 24 课，再追加 A2 第一单元 4 课，共 28 课、84 道题。独立 release ID 为 `a1-a2-travel-pilot-draft-v1`，不改变既有四份 A1 目录。完成草稿或结构检查不能宣称达到官方 A2 等级。

## 素材与角色

场景复用 [A1 原创城市场景 SVG](../a1/assets/city-morning.svg)，固定引用 `art-city-morning` revision 1；来源/哈希/尺寸记录继续使用 [原素材清单](../a1/scene-assets.bundle.json)。没有重复登记同一素材的新版本。角色复用 Camille、Luc、Léa 的 revision 1 快照：对话前独立介绍人物，接待员与售票员是本课情境身份；短文保留 narrator 和段落，不伪装成对话。

城市场景素材仍 planned、rightsConfirmed=false；头像素材与角色登记另按 [原示例清单](../../examples/asset-bundle.json) 和作者工具流程处理。引用文件已存在不等于授权和登记完成。没有添加 Web public 素材或绕过私有预览/发布门禁。

## 检查与发布边界

- 单课：`cargo run -p brioche-server -- check docs/content/a2/<lesson-id>.lesson.json`。
- 联合目录：`cargo run -p brioche-server -- check-release docs/content/a2/catalog.pilot.release.json`。
- 全包：`cargo test -p brioche-server --test curriculum`；核对两级目录及七个单元顺序、固定 revision/角色、A1/A2 共享知识一致、公开投影剥离私有字段，并使用正式 Grader 核对 84 道题的正确和合法错误答案；A2 正文检查 120–250 个空白分隔词和预期正文形式。

作者文件含 `serverOnly.grading`，不能直接发送给公共 API 或作为静态资源。正式预览需要先登记素材，再导入课程，由 operator 查看固定版本；当前不宣称完成四课的浏览器预览或教学审校。人工法语、中文译文、难度、文化范围、练习有效性、插图适配与录音仍待审校。正式发布继续受 reviewed 元数据、注册快照、授权与实际文件校验约束。

## 语言来源与待审校项

正文、译文、解释与练习为项目新写草稿，未复制第三方教材/题目/例句。参考资料只用于核对语言结构，不表示资料提供方认可或授权本课：

- [OQLF 复合过去时](https://vitrinelinguistique.oqlf.gouv.qc.ca/24218/la-grammaire/le-verbe/temps-grammaticaux/passe/generalites-sur-le-passe-compose)：核对助动词现在时 + 过去分词、已完成事件的表达，2026-10-06。
- [Larousse réserver 变位](https://www.larousse.fr/conjugaison/francais/reserver/8005)：核对现在时 réserver 的变位与复合过去时，2026-10-06。
- A1 复用知识的核对来源沿用 [A1 审校与来源记录](../a1/README.md)。

本单元不是完整过去时教程：prendre → pris、faire → fait 作为正文例形；明确不能把全部动词的助动词都设为 avoir。pas de souvenirs 的数量宾语否定和 comprend-il 的正式提问有随文解释，系统练习后续补充。待人工审校确认这些支持说明是否足够、八轮较长电话/售票正文是否适合目标学习者。
