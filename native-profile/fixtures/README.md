# Native profile fixtures

These are small function excerpts, not complete game binaries.

- `multi-2.1.22.bin`: 1056 bytes at RVA 0x1AA0040 from the user's installed Factorio 2.1.22 build 87719.
- `simple-2.1.22.bin`: 960 bytes at RVA 0x1A9FC80 from the same executable.
- `multi-2.1.19-reconstructed.bin`: reconstructed from the 2.1.22 excerpt and the nine external call targets recorded in the previous 2.1.19 investigation. Its `[16,0x400)` raw FNV matches the previously verified `08e1f8a33b7f69a9`. The full original 2.1.19 exe/PDB was not retained.
- `multi-2.1.15-reconstructed.bin`: reconstructed using the archived `rextract/wrap_disasm.txt` call targets. All 317 decoded instructions were compared against that archived listing; only external call addresses differ.

Historical fixtures test relocation normalization for both migrations, not a claim of rerunning those old game installations with the new DLL.
The 2.1.22 excerpts and callees were inspected before selecting them as the approved contract. Future unknown instruction streams are never enrolled automatically.
