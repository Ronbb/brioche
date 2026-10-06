# 有情绪的预生成课程语音

2026-10-07 用户要求：用 OpenAI 或 Qwen 等有情绪表现的语音模型提前生成课程语音，浏览器播放文件，不再使用设备合成。已有 Qwen 私有试听工具和真实生成的双角色样本；尚未登记正式音频，播放器当前仍含浏览器回退。后台生成/强制对齐仍为设计。

## 模型候选

| 候选 | 已核对能力 | 接入选择 |
| --- | --- | --- |
| OpenAI | TTS 文档列出法语，支持情绪、语调、速度等指令 | 先做云端双角色试听；模型与接口需结合当前退役公告选择，不把旧 speech snapshot 固定为长期方案 |
| Qwen3-TTS-Instruct-Flash API | 非实时合成支持自然语言语音指令 | 云端候选；明确区域、模型版本与音色，先试听法语 |
| Qwen3-TTS-12Hz-1.7B-CustomVoice | 官方开源模型支持法语、固定音色与 instruct | 本机候选；GPU/精度/依赖与吞吐实测后决定是否采用 |

依据：[OpenAI TTS](https://developers.openai.com/api/docs/guides/text-to-speech)、[OpenAI 退役公告](https://developers.openai.com/api/docs/deprecations)、[Qwen 非实时合成](https://help.aliyun.com/zh/model-studio/non-realtime-tts-user-guide)、[Qwen3-TTS 官方模型与示例](https://github.com/QwenLM/Qwen3-TTS)。OpenAI 于2026-10-01公告旧 tts-1/tts-1-hd/两个 gpt-4o-mini-tts snapshot 将于2027-01-06退役，推荐 Realtime 后继模型。传统 speech 文档仍展示旧接口；接入时要验证账户可用模型和后继接口的逐字生成能力，不能仅照抄旧示例。未核实 OpenAI 参数规模，不以“mini”或“HD”推断大小与法语质量。

Qwen 固定 speaker 可跨语言生成，但官方建议使用音色母语取得最佳效果；内置 speaker 不等于法语母语音色。实际法语试听是选择依据。小模型候选路线已停止，不作为正式对话声音。

用户补充准确型号为 `qwen-audio-3.1-tts-next` / `qwen-audio-3.1-tts-flash`。已核对 [Next 模型页](https://help.aliyun.com/en/model-studio/qwen-audio-3-1-tts-next) 和 [Flash 模型页](https://help.aliyun.com/en/model-studio/qwen-audio-3-1-tts-flash)：Next 输入使用 `text_prompt`，支持整段多人对话与参考音频，但官方当前语言只列中文/英文；不能沿用 Flash 的 `text/voice/instruction` 参数或声称支持法语。当前实际法语试听使用 Flash，不额外调用 Next 产生未验证法语/计费。后续若官方增加法语，或用户明确要实验其法语能力，再用独立适配器和同文对比，不静默替换模型。

## 角色和台词

角色库新增版本化声音档案：提供方、固定模型版本、voice/speaker 标识、语言 fr-FR、默认声音特征与语速。模型改变或角色音色改变创建新档案版本，不影响已发布课程。

每句增加私有配音脚本，引用正文 blockId/entryId、角色声音版本，记录场景、情绪强度、节奏和停顿；输入文本由固定课程 revision 的原文生成。正文、发音台词与情绪指令分开，模型不得朗读指令、中文注释或增加原文没有的词。文章由稳定旁白音色朗读。

例如面包店的顾客使用清晰、礼貌、略有期待的语气；店员使用温暖、轻快且自然的语气。首次见面的对话应有友好问候、介绍姓名时的自然重音和告别时的收尾变化。低等级课程保持清楚可辨的法语与自然连读，避免把全部台词统一成夸张或机械的情绪。

## 生成与发布

生成任务放在管理员后台，Rust负责授权、任务状态与审计，独立 worker 调用提供方。密钥仅由部署私有环境或秘密管理器提供，不发给浏览器、不写课程 JSON、审计或日志。先生成一段双角色试听，不自动批量生成整个目录。

任务固定课程 revision、脚本版本与声音版本。缓存键包含文本、语言、提供方、实际模型版本、音色、全部情绪/韵律参数和后处理版本。支持逐句重做、取消、失败重试和明确的数量/成本上限；超时后先核对任务结果，避免盲目重复计费。每次实际生成留来源、模型版本、参数摘要、文本哈希、音频哈希和任务审计。

试听工作区区分待生成、生成中、待试听、需重做、可登记、失败。检查实际台词、口音、音色一致性、长短音、连读、语气与角色轮换；结构验证不能证明发音正确。AI 合成来源在素材信息中明确标注。既有内容审批不冒充新增音频已经审校。

确认的分句音频按阅读顺序拼接，并从实际音频计算句子边界和停顿。单词点击仍需真实、精确的时间轴；服务不提供时间戳时另做文本强制对齐并核验，不能按字符长度平均切割或把 ASR 改写当成原文。独立词汇/表达和语法例句也需预生成；全部音频覆盖后统一移除浏览器发音与媒体失败回退。缺音频时给出 toast，不伪装播放成功。

输出复用现有 audio-check/audio-import/audioRefs/audioTracks 与固定版本媒体访问。已发布 revision 不可变，首六课补音频需要新 revision、新审校记录和新的 release 切换；保留旧会话与原目录回滚能力。尚未实现生成后台、提供方适配、强制对齐和正式替换，不将本文当作已完成能力。

## 当前需要的配置

用户已选择：不能直接使用 OpenAI 时使用 Qwen API。当前没有直接生成 OpenAI 音频的工具或已配置的 API Key，因此先接入 Qwen。

试听脚本 `scripts/qwen-tts.mjs` 使用 Qwen Audio 3.1 TTS Flash 的北京业务空间接口，固定两种官方列为支持法语的音色 `longanlingxin_v3.1` / `xunanchuan_v3.1`，8 句面包店对话逐句带情绪指令。3.0 Plus 内置音色仅标注中文/英文，不能因“Plus”就用于法语；本轮未核实云端参数量，不声称模型规模。依据：[API](https://help.aliyun.com/zh/model-studio/qwen-audio-tts-http-api)、[音色列表](https://help.aliyun.com/zh/model-studio/qwen-audio-tts-voice-list)。3.1 名称是提供方别名，未查到不可变快照；收据明确记录这一限制，不冒充可复现的固定版本。

将 `infra/tts.env.example` 复制到被忽略的 `.local/tts.env`，填写北京地域 `DASHSCOPE_API_KEY` 与 `DASHSCOPE_BASE_URL`。地址支持业务空间 HTTPS 根路径、`/api/v1` 或 `/compatible-mode/v1`，统一转为同主机的原生语音接口；不把 OpenAI 兼容路径当成音频合成接口，也不向其他域名发送密钥。仍兼容 `QWEN_WORKSPACE_ID`，配置 BASE_URL 时优先使用它。不要通过聊天、Git 或前端输入密钥。

```powershell
node scripts/qwen-tts.mjs --plan
node --env-file=.local/tts.env scripts/qwen-tts.mjs --generate
```

计划不访问网络；生成最多 8 次付费 POST，不自动重试。输出仅在 `.local/private/tts-qwen/bakery-<timestamp>`，逐句 WAV 和参数/哈希收据标为 unreviewed，开启 AI 来源标识。密钥与临时签名地址不写收据。只下载文档中的北京结果 bucket，升级 HTTPS、不跟随重定向、不转发 API 鉴权；响应大小和时限有界。重新执行会重新计费，失败前成功的文件保留，先检查提供方用量再重做。

2026-10-07 用户配置凭据后，实际完成 8 次生成，未自动重试。提供方 WAV 使用流式占位 RIFF/data 长度，正式 `audio-check` 初验拒绝；新增有界修正，仅修正这两个长度字段，不更改 PCM 或移除 AIGC 元数据，并保留 `.provider.wav` 原始文件与原始哈希。8 个修正分句全部通过 Rust 实际解码，24kHz/单声道。新增长度修正与基础地址协议回归，共8项通过。

已在私有目录用 FFmpeg 增加逐句400ms停顿并拼成13.12秒试听，实际解码通过。此拼接文件仅作试听，FFmpeg派生文件未保留原始AIGC容器块；原始/修正分句仍保留AI标识，正式导入需使用经过来源登记及审听的媒体，不能将试听混音直接冒充已批准资产。没有重新计费重生成。口音、情绪和文本忠实度仍待审听；没有正式登记/发布或课程时间轴，播放器仍保留现有浏览器回退。脚本尚未实现自动全文拼接、强制对齐、缓存复用或后台任务管理。
