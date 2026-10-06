# 课程配音逐词对齐

这是本机私有工具，读取 `/admin/speech-clips` 下载的已审听 TAR，将固定法语原文与真实 PCM 录音交给 Qwen3-ForcedAligner。生成的是待人工核对的预测，不会调用收费 TTS、访问应用数据库、登记录音或发布课程。输入文件仍需通过正式登记和服务端权限核对；本机清单中的审听记录不能替代服务端授权。

已验证 Windows、Python 3.12.12、CPU/float32/eager attention。GTX 1070 Ti 不作为 BF16/FlashAttention 环境使用；该模型和 Python 环境不装入 Web/API 生产镜像。模型固定到官方仓库 revision，配置、分词器及 safetensors 均核对仓库记录的 SHA-256。升级模型、编译器或关键依赖需要重新验证，不能静默接受不同版本。

## 安装与准备

以下命令在项目根目录执行，需要 `uv`。首次下载公共模型需要网络，下载不读取 Hugging Face 账户凭据或 TTS Key。Windows CPU 的全部依赖版本记录在 `requirements.windows-cpu.txt`，模型清单见 `model.json`。

```powershell
uv venv --python 3.12.12 .local/alignment-venv
uv pip install --python .local/alignment-venv/Scripts/python.exe --index-url https://download.pytorch.org/whl/cpu torch==2.10.0+cpu
uv pip install --python .local/alignment-venv/Scripts/python.exe -r scripts/alignment/requirements.windows-cpu.txt
.local/alignment-venv/Scripts/python.exe scripts/alignment/prepare.py
```

## 校验与对齐

下载包保存到 `.local/private`。先校验清单及媒体，此命令仅用 Python 标准库；正式对齐命令加载已准备且哈希正确的本机模型，设置离线模式，不发送音频。

```powershell
python scripts/alignment/align.py .local/private/speech-<plan-id>.tar --check
.local/alignment-venv/Scripts/python.exe scripts/alignment/align.py .local/private/speech-<plan-id>.tar --output .local/private/alignment-<plan-id>-v1.json
python -m unittest discover -s scripts/alignment -p test_align.py -v
```

输出必须位于 `.local/private`，已有文件不会覆盖。退出码0表示结构/模型预测范围通过；2表示已保存结果，但至少一个片段或分段需要处理；1表示输入、模型或运行失败。所有输出都保留 `reviewRequired: true`，退出码0不代表听感/时间轴人工审核通过。异常中断可能留下不完整输出，需要使用新文件名重新执行，不能把旧文件当成完成回执。

清单校验固定 compiler、Rust Serde 字段顺序的 planHash、完整请求 generationKey、所有目标覆盖、原文 Unicode scalar 范围、真实来源审听记录、原始/修复 WAV 配对和 SHA-256。只接受未压缩、128 MiB以内的 TAR；清单4 MiB以内，媒体每个16 MiB以内、单声道24 kHz/16-bit PCM且不超过180秒。只在内存读取精确名称的普通成员，不 extractall，不接受路径穿越、链接、重复或多余文件。

对齐输入按清单词单元组成：模型侧做 NFC 和法语撇号规范化，原始文本及 scalar 范围不改。保留模型返回的原始预测及运行版本；词数/文字不符、非有限数、零时长、重叠或越界会标记问题，异常片段不给出可用词时间。毫秒转换采用十进制向外取整，避免二进制浮点引入重叠，不修正模型本身的时间。

课程 segment 若把一个词切成不同文本范围，无法与整词预测精确映射时标记 `segmentWordBoundaryMismatch`。工具不平均拆分声学区间。需要人工修改分段或经过实际音频核对的时间轴；修改课程须追加新版本，再重新生成固定计划。Qwen 库自身会处理部分预测异常，输出仍是模型预测，不能宣称音素边界或人工审听已确认。

正式录音包组装、后台时间轴审查、登记和新课程/release 发布仍是后续步骤；目前不应直接把预测 JSON 导入已发布课程。

来源：[官方 Qwen3-ASR / ForcedAligner](https://github.com/QwenLM/Qwen3-ASR)、[固定模型](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/tree/c7cbfc2048c462b0d63a45797104fc9db3ad62b7)。模型许可证 Apache-2.0，模型文件不进入 Git。


`rust-plan.fixture.json` 是实际 Rust 编译器输出的测试向量，来自公开面包店示例与测试用 Léa 档案，不是生产声音选型或审听决定。用于防止 Python 对 JSONB 字段排序或请求哈希的处理与 Rust 漂移；更改编译器版本时重新生成并核对。CI 只运行标准库校验测试，不下载模型、不生成新音频。
