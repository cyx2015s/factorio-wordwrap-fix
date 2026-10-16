# 原生换行的逆向契约（当前实测 2.1.22 build 87719，Windows x64）

现在由 `wordwrap-inject --inspect <exe> --report <dir>` 自动读取匹配 PDB、定位符号、提取函数区间并生成带调用目标符号的反汇编。

| 版本 | multi RVA | simple RVA | 回退块入口 | 续接点 |
| --- | --- | --- | --- | --- |
| 2.1.15（历史反汇编） | 0x1A824E0 | 0x1A82120 | 0x1A826A3 | 0x1A826DC |
| 2.1.19（此前实测） | 0x1A89FE0 | 0x1A89C20 | 0x1A8A1A3 | 0x1A8A1DC |
| 2.1.22（本次实测） | 0x1AA0040 | 0x1A9FC80 | 0x1AA0203 | 0x1AA023C |

三个版本 multi 的指令布局一致；机械变化是外部调用目标位移。2.1.19 旧原始窗口指纹为 `08e1f8a33b7f69a9`，2.1.22 变为 `1a164d0d36c7e667`，因此旧 DLL 正确拒绝了更新版本。
新校验对两个完整函数作解码与归一化字节比对；multi 长 1056 字节，simple 长 960 字节。只屏蔽已解码 CALL rel32 与 RIP 相对操作数中的地址位移，其他字节必须精确一致，包括内部跳转、寄存器、栈偏移、入口和末尾错误分支。
9 个直接调用点还须命中批准的完整符号；simple 的 RIP 引用还须指向 double 1.0。不能通过把调用改向别处或更改常量来利用地址屏蔽。
动态 RVA 由 PDB 唯一符号计算；固定的 `+0x1C3/+0x1FC` 只在完整指令契约匹配后使用，非任意版本的后备偏移。

参考片段及历史重建来源见 [native-profile/fixtures/README.md](native-profile/fixtures/README.md)。2.1.15、2.1.19 的迁移回放是静态重建测试，本次实际运行的是 2.1.22。

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
.\target\release\wordwrap-inject.exe --inspect <factorio.exe> --report .runtime/inspection
.\target\release\wordwrap-inject.exe --inspect <factorio.exe> --symbol multiMakeLinesStrView --report .runtime/inspection
```

注入器先在进程外读取目标 exe 路径并检查；DLL 共用同一解析代码、缓存解析结果，再检查运行中 PE 身份与代码字节。
自己的观察/边界 detour 只允许已记录的五字节跳转差异；未知内存修改拒绝。启用和恢复时也会复核现有代码。
若新的编译器改变了指令流、布局或 ABI，仍需要人工逆向；工具输出差异与完整反汇编，不会自动批准新模板。
