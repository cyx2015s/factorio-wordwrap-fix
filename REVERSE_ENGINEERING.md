# 原生换行的逆向契约（2.1.19 build 87445，Windows x64）

本次通过同目录 `factorio.pdb` 与实际 exe 重新提取；不是根据旧 2.1.15 文本推断。

- `agui::ResizableText::multiMakeLinesStrView`：RVA `0x1A89FE0`。
- 原生宽度前缀计算 `simpleWrapTextUnsafe`：RVA `0x1A89C20`。
- 断点回退基本块：`multi+0x1C3 .. +0x1FC`，即 `0x1A8A1A3 .. 0x1A8A1DC`。
- 函数 `[+16,+0x400)` 的 FNV-1a/64 指纹：`0x08E1F8A33B7F69A9`。首 16 字节允许诊断 hook 使用，剩余函数体必须与复核版本一致。

`multi` 的机器级参数是 `(Font*, const MSVC_string*, vector<string_view>*, int width, int max_height_pixels, RichTextData*) -> int`。
`simple` 返回 16 字节 string_view，有 MSVC 的隐藏返回地址参数：`(View* result, const View* input, size_t character_count, RichTextData*, Font*, int width) -> View*`。
这些 ABI 通过调用点反汇编和实际运行验证。`max_height` 是像素高度，不是行数。

原生 `multi` 会分离显式换行，调用 `simple` 找出真实字体能容纳的前缀，再向前寻找空白或 `]`。
找不到断点时存在使用测量前缀的回退，**纯中文并非始终无法换行**；实际问题是混合单词、富文本边界和标点回退不正确。
旧的 `>=0x80` 七字节补丁既无法识别中文标点/非 CJK 字符，也移除了 `]` 的规则，不作为后备方案保留。

断点基本块入口的寄存器：

| 数据 | 来源 |
| --- | --- |
| 当前待处理段首地址 | RBX |
| 原生测量的前缀字节数 | RDI |
| 整个待处理段的字节数 | RSI |
| rich data | `[RBP+0x7F]` |
| 本地候选 string_view 的长度 | `[RBP-0x51]` |

stub 保存 volatile 通用寄存器和 XMM0–5，留出 32 字节 shadow space，保持调用前 RSP 的 16 字节对齐。
调用 Rust 边界选择器后，只改 RDI 和本地 view 长度，跳至原生 `mov rsi,rdi` 继续执行空白处理、vector 扩容与高度检查。
原有回退基本块及其循环内指令保留；入口通过 retour 的五字节跳转进入 stub。

引用 `[entity=fusion-generator]` 的图标和本地化名称由原生富文本 renderer 测量和绘制，不由 Rust 展开/估宽。
style 标签不占宽度，关闭标签尽量随前一段保留；对象两侧允许断行，但服从标点约束。
rich data 为空时，方括号文本按普通文本处理。

加载器在远程 LoadLibrary 完成后显式调用导出函数，安装失败向 CLI 传播。
不在 loader lock 下初始化 hook，不分配/释放游戏 STL 容器，不调用猜测的 FontStashFont 实例方法；写码后刷新 instruction cache。
控制调用用 mutex 串行，hook/relay 生命周期持续至进程退出。此处使用普通内联 detour，不宣称对其他 DLL 的并发改码或任意时刻热切换提供事务保证。

复核命令：

```powershell
cargo run --release --manifest-path rextract/Cargo.toml --bin disasm -- <factorio.exe> <factorio.pdb>
```

更新版本时需要重新检查完整函数、参数/返回 ABI、寄存器与局部变量存活区，再更新指纹；只看到短字节模式相同不够。
