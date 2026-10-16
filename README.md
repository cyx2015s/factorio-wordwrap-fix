# Factorio 中英混排与富文本换行修复

Windows x64 内存补丁。当前验证版本：**Factorio 2.1.19 / build 87445 / Space Age**。
游戏退出后补丁失效，不修改游戏 exe、PDB 或已有存档；这不是放进 `mods` 目录的 Lua mod。

修复内容：

- 中文按 Unicode 换行机会断行，中英相邻时保留正常英文单词。
- 中文禁行首/禁行尾标点随相邻文字排版。
- `[item=...]`、`[entity=...]`、`[fluid=...]` 等引用作为不可拆分对象；游戏绘制的图标和名称保留在一起，对象与前后正文可以断行。
- 保留游戏原生字体、缩放、富文本测量、换行符/空行、容器分配和高度限制语义。
- 不再用固定字符宽度估计，也不再因输出 vector 容量不足而主动截断。

## 构建与使用

注入器的 `dll-syringe` 依赖 nightly Rust：

```powershell
cargo +nightly build --release --locked
```

启动游戏，在主菜单或静止的界面执行：

```powershell
.\target\release\wordwrap-inject.exe
```

DLL 默认取注入器同目录下的 `factorio_wordwrap_fix.dll`，也可以显式传入路径。
多个游戏实例同时运行时，必须用 `--pid PID` 指定目标。
注入器等待 DLL 导出函数返回，实际安装失败会返回非零退出码，而不会只报告 LoadLibrary 成功。
已有界面可能缓存排版，安装/恢复后关闭并重新打开目标窗口。

```powershell
# 恢复原版；重复执行安全
.\target\release\wordwrap-inject.exe --restore
# 再次启用；重复执行安全
.\target\release\wordwrap-inject.exe
# 可选：记录真实原生输入、输出与逐行原生宽度复核
.\target\release\wordwrap-inject.exe --observe
# 停止记录并移除观察 hook，保留换行修复
.\target\release\wordwrap-inject.exe --trace-off
```

日志位于运行游戏账户的 `%TEMP%\factorio_wordwrap_fix.log`。默认只记录安装/错误，观察模式才记录文本。
DLL 的 `DllMain` 不安装补丁；其他加载器须在 LoadLibrary 完成后调用 `wordwrap_patch_apply()`，返回 0 表示成功。
另有 `wordwrap_patch_restore()`、`wordwrap_observe()` 和 `wordwrap_observe_stop()`；均返回 0/1。
游戏运行时不要卸载 DLL 或同时注入不同副本；已加载的代码与 trampoline 需要存活到游戏退出。

## 自动游戏回归

```powershell
.\tools\run-regression.ps1 -Factorio 'D:\异星工厂\Factorio_2.1\bin\x64\factorio.exe'
```

脚本在仓库 `.runtime` 下建立独立配置、mod 列表和测试场景，启用游戏本机 UDP 调试接口（127.0.0.1:34210）。
测试不读取或加载用户的工厂存档。请先关闭同一测试配置启动的旧实例。
脚本先检查目录锁；目录被占用时不会启动第二个游戏。需要复用旧测试实例时，传入 `-GamePid PID -Dll <该实例已加载的同一个DLL路径>`，脚本不会覆盖其配置。
脚本自动执行原版、修复、恢复、重新启用四阶段，每阶段创建字体/宽度矩阵并打开真正的聚变反应堆百科、运算器窗口。
覆盖三种字体、七种宽度、七类文本、长文本、GUI 高度裁切和富文本开关。真实运算器编辑框的原生限高返回值另有实测记录。
逐行调用原生 `simpleWrapTextUnsafe` 复核宽度，并检查文字保留和引用完整性；恢复后与原版逐项比较。
逐例 JSON、原生日志与汇总在 `.runtime/results`。

已启动上述测试场景时，也可以独立调用：

```powershell
python tools/game_command.py panel
python tools/game_command.py stress
python tools/game_command.py encyclopedia
python tools/game_command.py combinator
```

Rust 测试与静态检查：

```powershell
cargo +nightly test --workspace --locked
cargo +nightly clippy --workspace --all-targets --locked -- -D warnings
```

具体本次实测证据见 [tests/VALIDATION.md](tests/VALIDATION.md)。

## 实现与边界

补丁只替换 `agui::ResizableText::multiMakeLinesStrView` 中“从已测量前缀回退至合法断点”的基本块。
其余流程继续由游戏执行。Unicode UAX #14 用于可见文本，富文本引用映射为一个对象，再映射回源字符串字节偏移；超长单词的应急断行使用字素簇。

执行前检查 exe/PDB GUID 与 age、唯一符号、已逆向函数体指纹、栈帧、基本块和跳转续接指令。
**目前仅接受已验证的 2.1.19 函数体**；未知版本拒绝安装，没有硬编码 RVA 的冒险回退。
内联 hook 与其他修改同一函数的 DLL 不保证兼容，安装/切换应在静止界面完成。

如果单个图标引用或不可拆分的标点组合比控件本身还宽，保留整体可能产生溢出；任意宽度下同时保证“不拆分”和“不溢出”无法满足。
长英文单词超过整个控件时允许按字素簇应急拆行。此补丁不扩大控件或改变游戏的高度截断策略。
观察模式每次启用阶段最多记录 4000 个不同输入、单条最多 1000 行，属于诊断上限，不限制游戏实际输出。

逆向契约与 RVA 见 [REVERSE_ENGINEERING.md](REVERSE_ENGINEERING.md)。参考了本机 `F:\rust-crates\factorio-3d-mod` 的注入结构与 `tools/frida_wrap_{fix,observe}.py`，并在本机 2.1.19 上重新反汇编和验证，未沿用旧版本偏移。
