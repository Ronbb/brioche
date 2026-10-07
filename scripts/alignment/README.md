# 离线对齐入口

实现、固定模型/runtime、安装依赖和测试已迁入 [Chef](../../framework/scripts/alignment/README.md)。本目录只保留 align.py / prepare.py 兼容转发入口。

从产品根目录调用，私有模型缓存与输出仍在本产品 .local 下；从其他目录调用可设置 CHEF_WORKSPACE_ROOT 为产品绝对路径。不要将私有模型、媒体、环境或密钥迁到框架。安装依赖路径改为 framework/scripts/alignment/requirements.windows-cpu.txt，测试使用 python -m unittest discover -s framework/scripts/alignment -p test_align.py -v。

固定法语策略保持兼容，不代表粤语已支持；脚本验证或推理不等于媒体登记、发布或真实人类审听。
