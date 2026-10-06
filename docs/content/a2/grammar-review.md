# A2 语法说明辅助审阅

2026-10-06。读取六单元 24 课的 48 条 `knowledge.grammar` 说明与例句，检查时态、否定、比较、请求、条件、顺序和跨课复用说明的范围。这里只记录有限字段的辅助检查，不表示整课法语、中文译文、文化背景、教学难度或录音通过人工审校；所有课源仍为 draft。

## 修订

`grammar-a2-sequence` 在周末计划、办事步骤和昨天经历三课共用，旧说明“本课用这些连接词组织计划”不适合后两种正文。三处同步改为可组织计划、操作步骤和过去叙述，明确连接词不决定时态；保留原近期将来例句并说明它是计划的示例。[法兰西学院：ensuite](https://www.dictionnaire-academie.fr/article/A9E1816)。

`grammar-a2-devoir-present` 在家务与后续办事两课共用，旧说明只强调约定与家务。两处同步说明这几课的义务/需要表达可用于分工与办事，同时保留其他语义的边界。没有修改变位、练习答案或知识 ID。[OQLF：devoir 的构造与不同含义](https://vitrinelinguistique.oqlf.gouv.qc.ca/22441/la-grammaire/le-verbe/construction-et-sens-de-certains-verbes/emploi-des-verbes-devoir-et-se-devoir)。

其余读取的语法说明本次没有形成确定修订项，不等于独立审校通过。正文中的医疗场景、机构流程和虚构价格等仍需连同完整课文检查，不能由本报告推出真实世界操作建议。

## 验证

四份改动课源的作者 `check` 通过。完整联合目录带三处 source 来源的 `check-release` 实际检查 48 课通过；13 项 curriculum 测试通过，覆盖跨级共享知识一致、目录与判分等工程约束。JSON 格式检查通过。第一次将 `--sources` 重复写成多个 flag 导致 CLI 把第二个 flag 当路径，修正为一个 flag 后接全部来源后通过；该调用错误不是课源缺陷。

没有导入或发布课程，也没有更改素材授权状态。课程结构与判分验证不替代语言、教学或设备验收。
