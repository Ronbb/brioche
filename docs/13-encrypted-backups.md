# 加密备份副本

数据库和媒体一致快照仍由 `scripts/backup.ts` 创建与恢复；`scripts/backup-seal.ts` 为完整快照提供离线加密、校验和解密，不连接数据库，不修改生产卷，不调用 TTS，不上传外部存储。异盘目的地、每日执行、保留策略和异地恢复仍须单独落实，不能用同盘加密代替它们。

## 操作

在私有、受访问权限保护的目录准备密钥。密钥文件是随机 32 字节二进制文件，不是密码或 API Key；只通过文件路径传递，不输出密钥内容，不把密钥与加密副本一起上传。

```powershell
pnpm exec tsx scripts/backup-seal.ts keygen --key-file .local/private/backup-recovery.key
pnpm exec tsx scripts/backup-seal.ts seal --input .local/private/backups/example --output .local/private/sealed-example --key-file .local/private/backup-recovery.key
pnpm exec tsx scripts/backup-seal.ts verify --input .local/private/sealed-example --key-file .local/private/backup-recovery.key
pnpm exec tsx scripts/backup-seal.ts open --input .local/private/sealed-example --output .local/private/restored-example --key-file .local/private/backup-recovery.key
pnpm exec tsx scripts/backup.ts verify --input .local/private/restored-example
```

全部输出目录和新密钥文件必须不存在，脚本拒绝覆盖；加密目录不能包含源目录或密钥，也不能放在源目录中。`verify` 流式验证全部密文及原始 SHA-256，不写明文副本。`open` 仅写新的私有目录，全部对象认证和原摘要校验通过后才写完整 `manifest.json`，解密结果可接原有隔离 `restore` 流程。不要把结果恢复到生产数据库或现有媒体卷。

失败不自动清理文件。加密失败缺少最终 `envelope.json`，解密失败缺少完整 manifest；此类输出不得视为可恢复备份。解密中途可能留下敏感明文，即使认证最终失败也要将整个输出目录保持私有，检查后处理。目录/文件在支持 POSIX 权限的平台分别请求 0700/0600；Windows 必须依赖私有目录的 NTFS ACL，这些模式参数不能证明 Windows 访问权限。密钥丢失无法解密，应另存受保护的恢复副本；本工具不备份应用环境密钥或提供方凭据。

## 格式与验证边界

`brioche-sealed-backup-v1` 使用 AES-256-GCM，每个对象生成随机 12 字节 nonce 和 16 字节认证 tag。认证附加数据绑定格式、随机备份 UUID 和对象序号；跨备份替换、同一备份内调换文件、修改身份、截断或篡改不能通过认证。原备份 manifest 也加密，外部仅能看到格式、随机 UUID、对象数量及密文大小；媒体名称、课程信息和数据库内容不以明文写入加密副本。实现使用 [Node.js 官方 Crypto API](https://nodejs.org/api/crypto.html)，先设置 AAD，再流式加解密，完整结束后生成或验证 tag。这里是项目专用封装，不能直接交给通用 TAR/ZIP 解包工具。

加密前验证完整原备份，加密过程中再次检查每个对象的长度与摘要、manifest 是否改变。解密先认证 manifest，复用原备份路径、数量和单文件大小约束；只允许数据库固定文件名及哈希媒体文件名，不按未经校验的路径写磁盘。内存不会装入整个数据库或媒体文件。校验只证明封装和字节完整性，不能替代 `pg_restore`、数据库媒体引用检查或应用恢复演练。

自动回归覆盖往返及随机密文、错误密钥、身份篡改、密文损坏、截断、缺失对象、跨备份替换、不覆盖既有源/输出/密钥、嵌套路径及无效源摘要。生产规模验证和实际剩余门槛见实现清单。明确配置外部目的地后，只复制完整加密目录；在目的地重新下载/读取并 `verify`，再进行隔离恢复，才能声称异盘副本已验证。
