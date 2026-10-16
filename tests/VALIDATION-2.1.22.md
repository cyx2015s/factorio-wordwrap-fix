# 2.1.22 迁移与实测（2026-10-16）

实际游戏：Factorio **2.1.22 build 87719 / Windows x64 / Space Age**。
复用独立测试实例 PID 40720；测试没有加载用户工厂存档，也没有重复启动相同 write-data 的实例。

## 旧版本真实行为

先用原有注入器/DLL运行测试：加载和观察成功，但 apply 返回 1，日志为：

```
unsupported native wrap body 0x1a164d0d36c7e667; re-disassembly required
```

因此旧包不能直接在 2.1.22 应用修复。不是注入 API 失效，而是旧原始字节指纹不接受新的外部调用位移。

## 内置迁移工具

现由注入器内置 `--inspect <exe>`、`--symbol <fragment>`、`--report <dir>`，不再调用旧 Python / rextract 工具完成符号定位、函数提取、反汇编或指纹检查。
注入前自动读取目标进程 exe 路径，校验 PE/PDB GUID+age，读取全部公共符号一次并定位两个唯一函数；解码完整函数并验证调用符号、常量和归一化指令。
DLL 通过共享 crate 使用同一规则，并检查真正映射到游戏进程的身份与代码，缓存定位结果。

| 项目 | 2.1.22 |
| --- | --- |
| multi RVA / 长度 | 0x1AA0040 / 1056 |
| simple RVA / 长度 | 0x1A9FC80 / 960 |
| 边界块入口 / 续接点 | 0x1AA0203 / 0x1AA023C |
| 直接调用验证 | 9 处，完整符号身份 |
| simple 的 RIP 常量验证 | double 1.0 |

内置报告见 [evidence/2.1.22/inspection/profile.json](evidence/2.1.22/inspection/profile.json)，同目录包含带调用符号的两个 ASM 文件和两个原始 BIN 文件。BIN 与批准的参考片段逐字节一致。
历史 2.1.15→2.1.19→2.1.22 的 multi 归一化迁移均有静态回放测试；历史二进制是根据当时的调用目标/反汇编重建，并非本次重新运行旧版游戏。来源记录在 `native-profile/fixtures/README.md`。

## 最终发布构建的游戏回归

原版 → 修复 → 恢复 → 重新启用：每阶段 182 条原生输入/输出，其中 155 条有限宽度，共 728 条。
三种字体、七种宽度、七类混排/图标/标点文本，另有长文本、文本框、GUI 高度裁切、实际聚变反应堆百科与运算器窗口。
每条输出行再交给游戏的原生 simple wrapper 复查是否放得下。

- 修复/重新启用：0 文字丢失、0 富文本引用拆分、0 非预期溢出。
- 修复/重新启用各有 17 条包含不可拆分标点组合的极窄宽度溢出记录；没有隐去这一限制。
- 恢复后的 182/182 条输出与原版完全一致。
- 重新启用的 182/182 条输出与首次修复完全一致。
- 重复 apply/restore 成功；最终关闭观察 hook，修复保持启用，游戏停留在聚变反应堆百科。

详见 [evidence/2.1.22/summary.json](evidence/2.1.22/summary.json) 及四阶段 JSON/日志。
自动测试里的 GUI 高度裁切不等同于原生 TooLong 分支，本次不声称重新覆盖了真实编辑弹窗的有限高度返回分支；上次 2.1.19 的单独实测记录保留。

发布构建首轮遇到一次本机 UDP 应答超时，游戏仍正常响应；该轮未算作通过。完整重跑后通过，原失败目录 `.runtime/results-2.1.22-release` 保留。测试结果与通信失败分开统计。

## 拒绝路径和工程检查

- 真实 CLI 离线检查：复制 exe/PDB 到测试目录，只把 multi+0x1E2 的 `0x5D` 改为 `0x5E`，返回 1，报告精确定位该字节；原始游戏文件未修改。
- 真实 CLI 离线检查：给 exe 配上另一个模块的 PDB，返回 1，提示 GUID/age 不匹配。
- 单元测试：错误 CALL 目标即使归一化字节相同也拒绝；RIP 常量变更、指令改变、重复符号、非法 PE、代码越出映射范围均拒绝。
- `cargo test --offline --workspace`：21 项通过（9 项原生迁移/校验，12 项换行）。
- `cargo clippy --offline --workspace --all-targets -- -D warnings`、release build、`git diff --check` 均通过。

最终 DLL SHA256：`6D24E5648F14E41EC4BD456DCD6CCF2E7CE632B27E3D943B5D37033D27DC209E`。
最终注入器 SHA256：`33AA4EAE6E8C75718C9AE4E4BF22D62F4F2E6744C120BB8EB8F7812D0EAD6287`。

未来只有函数位置/外部地址移动时可自动适配；指令契约、栈偏移或 ABI 改变仍需人工审查，工具不会自动批准新模板。
