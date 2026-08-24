# factorio-wordwrap-fix 异星工厂中文换行修复 Mod

一个通过 DLL 注入修改异星工厂（Factorio）GUI 换行函数的 mod，使连续的中文（CJK）文本能按字符正常换行，而不是被当作一个不可分割的“单词”而溢出/被裁剪。

## 背景与根因

异星工厂的 GUI 库 agui 用 `agui::ResizableText::multiMakeLinesStrView` 来对文本换行。该函数只会在一行里“空白字符（isspace）”或特殊字符 `]`（0x5D）处断行：连续的汉字字符都不是这两类，所以一整串中文会被当成一个整体单词，长度远超控件宽度，结果就是中文文本溢出被裁剪、无法换行。

这正是参考项目 https://github.com/GDani31/factorio-3d-mod 中研究工作（tools/frida_wrap_fix.py）发现的根因，并用反汇编确认了修复点。

## 修复方式（就地字节补丁）

在 `multiMakeLinesStrView` 内有一个 7 字节的判断：它决定“当前位置之前的字符”是否是合法的断行点。原逻辑是判断该字节是否为 `]`，我们把它改成判断该字节是否为非 ASCII（>= 0x80），于是任何 UTF-8 中文/日文/韩文字符都成了合法断行点。

反汇编（factorio 2.1.15, x64；该测试位于函数偏移 +0x1DE、对应 RVA 0x1A826BE）。本 mod 运行时会自行在 factorio.pdb 中解析 `multiMakeLinesStrView` 并扫描该 7 字节模式来定位补丁点（见「行为说明 - 版本自适应」）：

```
vanilla (原字节 80 7C 3B FF 5D 74 17)
    cmp byte ptr [reg+disp], 0x5D   ; 判断是否为 ']'
    je   short +0x17                 ; 是则在此断行

patched (补丁字节 80 7C 3B FF 80 73 17)
    cmp byte ptr [reg+disp], 0x80   ; 判断是否非 ASCII（UTF-8 首字节 >= 0x80）
    jae  short +0x17                 ; 是则在此断行
```

`cmp ... , 0x5D ; je`（==']'）被替换为 `cmp ... , 0x80 ; jae`（>= 0x80）。
两个版本指令长度完全一致（7 字节），因此可以就地覆盖，不改变紧随其后的控制流。
ASCII（0x00–0x7F）字符仍走原始的 isspace 回退逻辑，英文文本的换行行为不受影响。

本补丁是**内存级**补丁：只修改运行中的进程内代码，游戏退出即恢复原始状态，不修改磁盘上的任何文件。

## 目录结构

```
factorio-wordwrap-fix/
├── Cargo.toml                    工作区根
├── factorio_wordwrap_fix/        注入的 DLL（核心补丁逻辑）
│   ├── Cargo.toml
│   └── src/lib.rs
├── injector/                     注入器（把 DLL 加载进 factorio.exe）
│   ├── Cargo.toml
│   └── src/main.rs
├── rextract/                     离线偏移提取工具（独立 crate，已从工作区 exclude）
│   ├── Cargo.toml
│   └── src/main.rs
└── README.md
```

## 构建

要求：Rust（edition 2024）。注意：注入器依赖的 dll-syringe 需要 **nightly** 工具链，
因此整个工作区请用 nightly 构建（DLL 本体在 stable 下也能编译，可单独用于其它注入方式）：

```powershell
cd F:/rust-crates/factorio-wordwrap-fix
cargo +nightly build --release
```

产物：
- `target/release/factorio_wordwrap_fix.dll` —— 注入到游戏里的补丁 DLL
- `target/release/wordwrap-inject.exe` —— 注入器

> dll-syringe 需要从 Cargo 仓库拉取；若离线，可在局域网/缓存环境先 `cargo fetch`。

## 使用

1. 启动异星工厂（factorio.exe）。
2. 在游戏运行时运行注入器：

```powershell
cargo run --release -p factorio-wordwrap-injector
```

或直接运行已构建的注入器并把 DLL 路径作为参数：

```powershell
target/release/wordwrap-inject.exe target/release/factorio_wordwrap_fix.dll
```

3. 注入成功后游戏内会出现一个控制台窗口，打印补丁结果；同时结果也会写入
   `%APPDATA%/Factorio/factorio_wordwrap_fix.log`。

### 从外部切换 / 恢复

DLL 还导出了两个 C 符号，可直接调用：

- `wordwrap_patch_apply()`   应用补丁（成功返回 0）
- `wordwrap_patch_restore()` 恢复原始字节（成功返回 0）

## 行为说明

- **幂等**：若目标字节已是 `PATCH`（已应用）或已是 `ORIG`（已恢复），不会重复写入。
- **防误伤**：若目标处字节既不是 `ORIG` 也不是 `PATCH`，程序会**拒绝写入**并记录原因，避免破坏其它版本的代码。
- **版本自适应**：DLL 加载时先读 factorio.pdb，解析 `multiMakeLinesStrView` 的 RVA，再在函数起始附近一段窗口内扫描 `ORIG`/`PATCH` 的 7 字节模式来定位补丁点——即便版本更新导致函数整体移动也能自动重新定位（2.0.77 与 2.1.15 该模式都稳定出现于函数偏移 +0x1DE）。`PATCH_RVA` 仅作为 pdb 缺失时的回退值（当前为 2.1.15 的 0x1A826BE）。
- **重新提取偏移**：若未来连这个 7 字节模式都变了，用仓库里的离线工具 `rextract` 重新提取（位于 `rextract/`，依赖 `pdb` + `iced-x86`，已从工作区 exclude）：`rextract <factorio.exe> <factorio.pdb> multiMakeLinesStrView`，会打印新的补丁点 RVA 与原始 7 字节，据此更新 `factorio_wordwrap_fix/src/lib.rs` 顶部的 `PATCH_RVA`（必要时还有 `ORIG`/`PATCH`）。

## 原理与参考

- 参考项目：https://github.com/GDani31/factorio-3d-mod（其 tools/frida_wrap_fix.py 是最初的验证脚本，
  tools/frida_wrap_observe.py 用于观察换行函数的输入/输出）。
- 本 mod 的注入与补丁结构借鉴了 factorio-3d-mod（injector + cdylib DLL、DllMain 里
  开线程执行、AllocConsole 控制台、日志落盘），以及其 `windows` crate 的使用方式
  （GetModuleHandleA / VirtualProtect）。
- 本次针对 2.1.15（build 87284）重新提取偏移：先用 factorio-3d-mod 的 tools/pdb_explorer.py
  解析出 `multiMakeLinesStrView` 新 RVA（0x1A824E0），再用新增的离线工具 `rextract` 反汇编该函数、
  定位 `cmp byte[..],0x5D ; je`，读出补丁点 RVA 0x1A826BE 与原始 7 字节 `80 7C 3B FF 5D 74 17`（与原版本一致）。

## 日志位置

`%APPDATA%/Factorio/factorio_wordwrap_fix.log`（若存在 APPDATA/Factorio），否则只在控制台输出。
