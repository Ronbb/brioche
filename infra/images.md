# 容器镜像固定与更新

2026-10-06 已查询 Docker 官方 registry 的 manifest index，并在本机 Linux/amd64 Docker 中实际构建与启动 Compose。以下使用版本标签 + index digest；更新必须重新核对 digest、构建与运行，不能只修改标签。

| 用途 | 标签 | 固定 index SHA-256 |
| --- | --- | --- |
| API 构建 | rust:1.99.0-bookworm | 59037199c44290f2befcdd58dcc540164763fc296950255aaefeef096a1866b0 |
| API 运行 | debian:bookworm-slim | 3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251 |
| Web 构建/运行 | node:24.20.0-bookworm-slim | ba849c60be29959425b8734d57b8b4b7d56f98edd9504c9af091d5281095a71e |
| 数据库/备份测试 | postgres:18.6-bookworm | 3725f4e2499eef5134592b3b4ab79a543ed7f8e533b05b5b637af926630f6650 |
| 网关 | traefik:v3.7.13 | 24841fe2de7304c149343d877d2923b4c8800a38ba015dea9174c23b20e344a0 |

镜像更新前用 `docker buildx imagetools inspect <tag>` 获取官方 index digest，并保留实际查询结果。同步修改 Dockerfile、Compose、CI 与运维 Docker 测试中对应引用；备份 helper 直接复用现有数据库容器的实际 image ID，不自行拉取浮动标签。index 包含多平台元数据，本轮仅验证 Linux/amd64，其他架构仍需实际构建与运行验证。

基础镜像固定不代表整个构建完全离线或逐字节相同：Cargo/pnpm 仍按 lockfile 下载依赖，API runtime 会安装 Debian 包，BuildKit provenance 也会影响镜像 index。应用本身由当前源代码构建；上线时应记录 Git commit、构建参数、实际 API/Web image ID，并保存之前的应用镜像，不能把 `brioche-api:local` 当作不可变版本号。本轮 `compose-qa` 镜像只是隔离验收产物，没有推送容器 registry 或用于真实公网生产。

验收顺序：类型检查与相关测试 → `docker compose build` → 专用项目/卷的 `up --wait` → 确认一次性 migrate 退出 0、数据库/API/Web/Traefik 健康 → HTTP 入口、Origin/会话、SSR/静态资源、固定版本学习/媒体 → 备份/恢复和回滚方案。外部 HTTPS、Cloudflare 与路由器由用户处理；本栈仅 HTTP 30075。详细证据见 [设计验证记录](../docs/07-design-verification.md)。
