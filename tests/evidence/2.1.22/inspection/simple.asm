01A9FC80: mov [rsp+20h],r9
01A9FC85: mov [rsp+18h],r8
01A9FC8A: mov [rsp+8],rcx
01A9FC8F: push rbx
01A9FC90: push r13
01A9FC92: push r14
01A9FC94: sub rsp,80h
01A9FC9B: mov r14,r8
01A9FC9E: mov r13,rdx
01A9FCA1: mov rbx,rcx
01A9FCA4: test r9,r9
01A9FCA7: jne near ptr 0000000001A9FD4Bh
01A9FCAD: movaps xmm0,[rdx]
01A9FCB0: lea rdx,[rsp+40h]
01A9FCB5: mov rcx,[rsp+0C0h]
01A9FCBD: movsd xmm3,qword ptr [23F0B48h]
01A9FCC5: mov r8d,[rsp+0C8h]
01A9FCCD: movdqa [rsp+40h],xmm0
01A9FCD3: mov rax,[rcx]
01A9FCD6: call qword ptr [rax+28h]
01A9FCD9: mov rdx,rax
01A9FCDC: test rax,rax
01A9FCDF: jne short 0000000001A9FD27h
01A9FCE1: cmp [r13+8],rax
01A9FCE5: je short 0000000001A9FD27h
01A9FCE7: movaps xmm0,[r13]
01A9FCEC: movaps [rsp+40h],xmm0
01A9FCF1: psrldq xmm0,8
01A9FCF6: movq rcx,xmm0
01A9FCFB: test rcx,rcx
01A9FCFE: je short 0000000001A9FD27h
01A9FD00: mov edx,1
01A9FD05: cmp rdx,rcx
01A9FD08: jae short 0000000001A9FD27h
01A9FD0A: mov r8,[rsp+40h]
01A9FD0F: nop
01A9FD10: movzx eax,byte ptr [r8+rdx]
01A9FD15: test al,al
01A9FD17: jns short 0000000001A9FD27h
01A9FD19: and al,0C0h
01A9FD1B: cmp al,0C0h
01A9FD1D: je short 0000000001A9FD27h
01A9FD1F: inc rdx
01A9FD22: cmp rdx,rcx
01A9FD25: jb short 0000000001A9FD10h
01A9FD27: mov rax,[r13]
01A9FD2B: cmp [r13+8],rdx
01A9FD2F: mov [rbx],rax
01A9FD32: mov rax,rbx
01A9FD35: cmovb rdx,[r13+8]
01A9FD3A: mov [rbx+8],rdx
01A9FD3E: add rsp,80h
01A9FD45: pop r14
01A9FD47: pop r13
01A9FD49: pop rbx
01A9FD4A: ret
01A9FD4B: movsxd rax,dword ptr [rsp+0C8h]
01A9FD53: mov [rsp+0A8h],rbp
01A9FD5B: mov [rsp+78h],rsi
01A9FD60: mov rsi,r8
01A9FD63: mov [rsp+70h],rdi
01A9FD68: xor edi,edi
01A9FD6A: shr rsi,1
01A9FD6D: mov ebp,edi
01A9FD6F: shr rax,1
01A9FD72: cmp rax,rsi
01A9FD75: mov [rsp+68h],r12
01A9FD7A: mov [rsp+60h],r15
01A9FD7F: cmovb rsi,rax
01A9FD83: movaps [rsp+50h],xmm6
01A9FD88: movsd xmm6,qword ptr [23F0B48h]
01A9FD90: cmp r8,1
01A9FD94: jbe near ptr 0000000001A9FE5Ah
01A9FD9A: mov r12,[rdx]
01A9FD9D: mov r15,[rsp+0C0h]
01A9FDA5: mov rbx,rdi
01A9FDA8: test rsi,rsi
01A9FDAB: je short 0000000001A9FDEEh
01A9FDAD: movaps xmm0,[r13]
01A9FDB2: mov r8,rsi
01A9FDB5: movdqa [rsp+30h],xmm0
01A9FDBB: mov rcx,[rsp+38h]
01A9FDC0: mov rdx,[rsp+30h]
01A9FDC5: cmp rbx,rcx
01A9FDC8: jae short 0000000001A9FDE8h
01A9FDCA: inc rbx
01A9FDCD: cmp rbx,rcx
01A9FDD0: jae short 0000000001A9FDE8h
01A9FDD2: movzx eax,byte ptr [rdx+rbx]
01A9FDD6: test al,al
01A9FDD8: jns short 0000000001A9FDE8h
01A9FDDA: and al,0C0h
01A9FDDC: cmp al,0C0h
01A9FDDE: je short 0000000001A9FDE8h
01A9FDE0: inc rbx
01A9FDE3: cmp rbx,rcx
01A9FDE6: jb short 0000000001A9FDD2h
01A9FDE8: sub r8,1
01A9FDEC: jne short 0000000001A9FDC5h
01A9FDEE: mov rax,[r15]
01A9FDF1: mov rcx,r15
01A9FDF4: call qword ptr [rax+58h]
01A9FDF7: mov r8,[rsp+0B8h]
01A9FDFF: lea rdx,[rsp+40h]
01A9FE04: mov r9,r15
01A9FE07: movsd qword ptr [rsp+20h],xmm6
01A9FE0D: mov [rsp+40h],r12
01A9FE12: mov rcx,[rax]
01A9FE15: mov [rsp+48h],rbx
01A9FE1A: mov r10,[rcx+28h]
01A9FE1E: mov rcx,rax
01A9FE21: call r10
01A9FE24: cmp eax,[rsp+0C8h]
01A9FE2B: mov rcx,rsi
01A9FE2E: cmovle rcx,r14
01A9FE32: cmovle rbp,rsi
01A9FE36: mov rax,rcx
01A9FE39: mov r14,rcx
01A9FE3C: sub rax,rbp
01A9FE3F: mov rsi,rax
01A9FE42: shr rsi,1
01A9FE45: add rsi,rbp
01A9FE48: cmp rax,1
01A9FE4C: ja near ptr 0000000001A9FDA5h
01A9FE52: mov [rsp+0B0h],rcx
01A9FE5A: test rbp,rbp
01A9FE5D: je short 0000000001A9FEA0h
01A9FE5F: movaps xmm0,[r13]
01A9FE64: mov r8,rbp
01A9FE67: movdqa [rsp+40h],xmm0
01A9FE6D: mov rcx,[rsp+48h]
01A9FE72: mov rdx,[rsp+40h]
01A9FE77: cmp rdi,rcx
01A9FE7A: jae short 0000000001A9FE9Ah
01A9FE7C: inc rdi
01A9FE7F: cmp rdi,rcx
01A9FE82: jae short 0000000001A9FE9Ah
01A9FE84: movzx eax,byte ptr [rdx+rdi]
01A9FE88: test al,al
01A9FE8A: jns short 0000000001A9FE9Ah
01A9FE8C: and al,0C0h
01A9FE8E: cmp al,0C0h
01A9FE90: je short 0000000001A9FE9Ah
01A9FE92: inc rdi
01A9FE95: cmp rdi,rcx
01A9FE98: jb short 0000000001A9FE84h
01A9FE9A: sub r8,1
01A9FE9E: jne short 0000000001A9FE77h
01A9FEA0: cmp rbp,r14
01A9FEA3: ja near ptr 0000000001A9FF49h
01A9FEA9: movaps xmm0,[r13]
01A9FEAE: mov r12,[r13]
01A9FEB2: movdqa [rsp+40h],xmm0
01A9FEB8: mov rbx,[rsp+48h]
01A9FEBD: mov rsi,[rsp+40h]
01A9FEC2: mov r15,rbp
01A9FEC5: mov r14,rdi
01A9FEC8: inc rbp
01A9FECB: cmp rdi,rbx
01A9FECE: jae short 0000000001A9FEEEh
01A9FED0: inc rdi
01A9FED3: cmp rdi,rbx
01A9FED6: jae short 0000000001A9FEEEh
01A9FED8: movzx eax,byte ptr [rsi+rdi]
01A9FEDC: test al,al
01A9FEDE: jns short 0000000001A9FEEEh
01A9FEE0: and al,0C0h
01A9FEE2: cmp al,0C0h
01A9FEE4: je short 0000000001A9FEEEh
01A9FEE6: inc rdi
01A9FEE9: cmp rdi,rbx
01A9FEEC: jb short 0000000001A9FED8h
01A9FEEE: mov rcx,[rsp+0C0h]
01A9FEF6: mov rax,[rcx]
01A9FEF9: call qword ptr [rax+58h]
01A9FEFC: mov r9,[rsp+0C0h]
01A9FF04: lea rdx,[rsp+40h]
01A9FF09: mov r8,[rsp+0B8h]
01A9FF11: movsd qword ptr [rsp+20h],xmm6
01A9FF17: mov rcx,[rax]
01A9FF1A: mov [rsp+40h],r12
01A9FF1F: mov [rsp+48h],rdi
01A9FF24: mov r10,[rcx+28h]
01A9FF28: mov rcx,rax
01A9FF2B: call r10
01A9FF2E: cmp eax,[rsp+0C8h]
01A9FF35: jg short 0000000001A9FF43h
01A9FF37: cmp rbp,[rsp+0B0h]
01A9FF3F: jbe short 0000000001A9FEC2h
01A9FF41: jmp short 0000000001A9FF49h
01A9FF43: mov rbp,r15
01A9FF46: mov rdi,r14
01A9FF49: movaps xmm6,[rsp+50h]
01A9FF4E: test rbp,rbp
01A9FF51: mov rbp,[rsp+0A8h]
01A9FF59: mov r15,[rsp+60h]
01A9FF5E: mov r12,[rsp+68h]
01A9FF63: jne near ptr 0000000001AA0005h
01A9FF69: mov rbx,[r13+8]
01A9FF6D: test rbx,rbx
01A9FF70: je near ptr 0000000001AA0005h
01A9FF76: movaps xmm0,[r13]
01A9FF7B: movaps [rsp+40h],xmm0
01A9FF80: psrldq xmm0,8
01A9FF85: movq rcx,xmm0
01A9FF8A: cmp rdi,rcx
01A9FF8D: jae short 0000000001A9FFB6h
01A9FF8F: inc rdi
01A9FF92: cmp rdi,rcx
01A9FF95: jae short 0000000001A9FFB6h
01A9FF97: mov rdx,[rsp+40h]
01A9FF9C: nop dword ptr [rax]
01A9FFA0: movzx eax,byte ptr [rdx+rdi]
01A9FFA4: test al,al
01A9FFA6: jns short 0000000001A9FFB6h
01A9FFA8: and al,0C0h
01A9FFAA: cmp al,0C0h
01A9FFAC: je short 0000000001A9FFB6h
01A9FFAE: inc rdi
01A9FFB1: cmp rdi,rcx
01A9FFB4: jb short 0000000001A9FFA0h
01A9FFB6: mov rsi,[rsp+0C0h]
01A9FFBE: mov rcx,rsi
01A9FFC1: mov rax,[rsi]
01A9FFC4: call qword ptr [rax+58h]
01A9FFC7: mov rcx,[r13]
01A9FFCB: lea r8,[rsp+40h]
01A9FFD0: mov r9,[rsp+0B8h]
01A9FFD8: lea rdx,[rsp+30h]
01A9FFDD: mov [rsp+40h],rcx
01A9FFE2: cmp rbx,rdi
01A9FFE5: mov rcx,[rax]
01A9FFE8: cmovb rdi,rbx
01A9FFEC: mov [rsp+20h],rsi
01A9FFF1: mov [rsp+48h],rdi
01A9FFF6: mov r10,[rcx+48h]
01A9FFFA: mov rcx,rax
01A9FFFD: call r10
01AA0000: mov rdi,[rsp+38h]
01AA0005: mov rdx,[rsp+0A0h]
01AA000D: cmp [r13+8],rdi
01AA0011: mov rax,[r13]
01AA0015: cmovb rdi,[r13+8]
01AA001A: mov rsi,[rsp+78h]
01AA001F: mov [rdx],rax
01AA0022: mov rax,rdx
01AA0025: mov [rdx+8],rdi
01AA0029: mov rdi,[rsp+70h]
01AA002E: add rsp,80h
01AA0035: pop r14
01AA0037: pop r13
01AA0039: pop rbx
01AA003A: ret
01AA003B: int 3
01AA003C: int 3
01AA003D: int 3
01AA003E: int 3
01AA003F: int 3