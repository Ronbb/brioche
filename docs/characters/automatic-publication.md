# 所有者授权的自动录音课包

2026-10-07 所有者要求录音与课程直接发布，不再等待人工审批。自动路径保留真实媒体、固定版本、权限和发布审计；不能填写虚假的 `heard` 或 `timingsChecked`。

`speech-plan-export-direct <plan-id> <operator-email> <new-private-output.tar>` 导出最新 ready 片段及原始、修复 WAV。显式退回的片段仍不可使用。私有输入包含完整角色档案、课源及提供方参数，不能提交到公开仓库。

本机固定模型预测使用 `scripts/alignment/align.py --direct`。原始预测不得覆盖；重叠、零时长或文本不一致须由实际重新推理或新生成的录音解决，禁止平均分配时间。补充推理保留原始结果、实际媒体哈希、输入窗口、偏移、模型及依赖版本。

`speech-package-automatic <report.json> <package-request.json> <operator-email> <new-private-output.tar>` 接收 `brioche-automatic-alignment-v1` 报告及 `AdminSpeechPackageRequest`。`expectedReportHash` 使用解析后的 JSON 值规范序列化 SHA-256，不能使用缩进报告文件的字节哈希。报告须显式 `reviewRequired=false`、`humanListeningAsserted=false`，与当前固定计划、最新片段、原始输入 TAR、媒体哈希、完整文本及全部目标逐词范围一致。

组装复用实际 PCM 拼接、逐词偏移、AI 元数据保留和录音校验。新课程 revision 必须大于已导入的全部版本；交付前后重查管理员权限和固定快照。输出拒绝覆盖，是包含答案和来源的私有 TAR；课源标记所有者授权，明确没有人工试听或独立专家审校声明。组装不收费，不插入人工审核，不登记录音、不导入课源、不切换公开目录。

实际生产发布仍须登记包内录音、导入新课源，再使用 `lesson-direct-publication` 记录单独的所有者授权，最后 staging 和原子激活完整课程目录。保留旧版本及既有学习会话。后续人工退回可以使直接授权失效。

验证包括真实隔离 PostgreSQL 的完整片段生成、自动组装、旧人工流程与课包导入链；自动分支验证无人工时间决定时可组装，以及权限、未解决异常、过期片段、输入哈希和虚假人工声明被拒绝。协议 fixture 不证明真实课程已发布；生产录音覆盖应以实际登记和激活后的公开课程为准。

正文标注必须保留完整Unicode词，例如将`l’heure`放在同一segment，不能分别标注`l’`和`heure`。预览编译在生成请求前拒绝不一致的完整词/segment词边界；这不改变有效输入的合成参数或生成键。修改既有课源须追加revision，不修改旧计划。原句、角色、参数、生成键、实际PCM和源词范围完全相同时，可明确记录复用已有模型预测；重新绑定新私有导出包/课源哈希和目标映射，保留原报告/原包哈希与标注改动，不能声称进行了新的推理或审听。
