01AA0040: mov rax,rsp
01AA0043: mov [rax+20h],r9d
01AA0047: mov [rax+18h],r8
01AA004B: mov [rax+8],rcx
01AA004F: push rbp
01AA0050: push rbx
01AA0051: push rsi
01AA0052: push rdi
01AA0053: push r12
01AA0055: push r13
01AA0057: push r14
01AA0059: push r15
01AA005B: lea rbp,[rax-4Fh]
01AA005F: sub rsp,0C8h
01AA0066: movaps [rax-58h],xmm6
01AA006A: mov r13d,r9d
01AA006D: mov r12,rcx
01AA0070: mov rax,[r8]
01AA0073: cmp rax,[r8+8]
01AA0077: je short 0000000001AA007Dh
01AA0079: mov [r8+8],rax
01AA007D: xorps xmm0,xmm0
01AA0080: movdqu [rbp-41h],xmm0
01AA0085: mov qword ptr [rbp-31h],0
01AA008D: mov rsi,rdx
01AA0090: cmp qword ptr [rdx+18h],0Fh
01AA0095: jbe short 0000000001AA009Ah
01AA0097: mov rsi,[rdx]
01AA009A: mov rdi,[rdx+10h]
01AA009E: xor ebx,ebx
01AA00A0: lea rax,[rbx+rsi]
01AA00A4: mov [rbp-79h],rax
01AA00A8: xor edx,edx
01AA00AA: mov ecx,1
01AA00AF: mov [rbp-71h],rdx
01AA00B3: mov r8,rbx
01AA00B6: cmp rbx,rdi
01AA00B9: jae short 0000000001AA010Ah
01AA00BB: inc rbx
01AA00BE: cmp rbx,rdi
01AA00C1: jae short 0000000001AA00DCh
01AA00C3: movzx eax,byte ptr [rbx+rsi]
01AA00C7: test al,al
01AA00C9: jns short 0000000001AA00DCh
01AA00CB: and al,0C0h
01AA00CD: cmp al,0C0h
01AA00CF: je short 0000000001AA00DCh
01AA00D1: inc rbx
01AA00D4: inc rcx
01AA00D7: cmp rbx,rdi
01AA00DA: jb short 0000000001AA00C3h
01AA00DC: mov r14,rcx
01AA00DF: test rcx,rcx
01AA00E2: je short 0000000001AA00F6h
01AA00E4: cmp rcx,1
01AA00E8: jne short 0000000001AA00F1h
01AA00EA: cmp byte ptr [r8+rsi],0Ah
01AA00EF: je short 0000000001AA00F6h
01AA00F1: add rdx,rcx
01AA00F4: jmp short 0000000001AA00AAh
01AA00F6: lea rdx,[rbp-79h]
01AA00FA: lea rcx,[rbp-41h]
01AA00FE: call 00000000015DB510h ; ??$_Emplace_one_at_back@AEBV?$basic_string_view@DU?$char_traits@D@std@@@std@@@?$vector@V?$basic_string_view@DU?$char_traits@D@std@@@std@@V?$allocator@V?$basic_string_view@DU?$char_traits@D@std@@@std@@@2@@std@@AEAAAEAV?$basic_string_view@DU?$char_traits@D@std@@@1@AEBV21@@Z | ??$_Emplace_one_at_back@AEBVVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@AEBV2@@Z | ??$_Emplace_one_at_back@VColor@agui@@@?$vector@VColor@agui@@V?$allocator@VColor@agui@@@std@@@std@@AEAAAEAVColor@agui@@$$QEAV23@@Z | ??$_Emplace_one_at_back@VVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@$$QEAV2@@Z
01AA0103: test r14,r14
01AA0106: je short 0000000001AA0117h
01AA0108: jmp short 0000000001AA00A0h
01AA010A: lea rdx,[rbp-79h]
01AA010E: lea rcx,[rbp-41h]
01AA0112: call 00000000015DB510h ; ??$_Emplace_one_at_back@AEBV?$basic_string_view@DU?$char_traits@D@std@@@std@@@?$vector@V?$basic_string_view@DU?$char_traits@D@std@@@std@@V?$allocator@V?$basic_string_view@DU?$char_traits@D@std@@@std@@@2@@std@@AEAAAEAV?$basic_string_view@DU?$char_traits@D@std@@@1@AEBV21@@Z | ??$_Emplace_one_at_back@AEBVVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@AEBV2@@Z | ??$_Emplace_one_at_back@VColor@agui@@@?$vector@VColor@agui@@V?$allocator@VColor@agui@@@std@@@std@@AEAAAEAVColor@agui@@$$QEAV23@@Z | ??$_Emplace_one_at_back@VVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@$$QEAV2@@Z
01AA0117: mov dword ptr [rbp+5Fh],0
01AA011E: mov rcx,[rbp-41h]
01AA0122: mov [rbp-49h],rcx
01AA0126: mov rax,[rbp-39h]
01AA012A: mov [rbp-29h],rax
01AA012E: cmp rcx,rax
01AA0131: je near ptr 0000000001AA03D1h
01AA0137: nop word ptr [rax+rax]
01AA0140: movups xmm6,[rcx]
01AA0143: movaps [rbp-79h],xmm6
01AA0147: xor r14d,r14d
01AA014A: mov [rsp+30h],r14
01AA014F: mov rax,[rcx]
01AA0152: mov rcx,[rcx+8]
01AA0156: add rcx,rax
01AA0159: cmp rax,rcx
01AA015C: je short 0000000001AA0191h
01AA015E: xchg ax,ax
01AA0160: inc r14
01AA0163: inc rax
01AA0166: cmp rax,rcx
01AA0169: je short 0000000001AA018Ch
01AA016B: nop dword ptr [rax+rax]
01AA0170: movzx edx,byte ptr [rax]
01AA0173: test dl,dl
01AA0175: jns short 0000000001AA0187h
01AA0177: and dl,0C0h
01AA017A: cmp dl,0C0h
01AA017D: je short 0000000001AA0187h
01AA017F: inc rax
01AA0182: cmp rcx,rax
01AA0185: jne short 0000000001AA0170h
01AA0187: cmp rax,rcx
01AA018A: jne short 0000000001AA0160h
01AA018C: mov [rsp+30h],r14
01AA0191: movdqa xmm0,xmm6
01AA0195: psrldq xmm0,8
01AA019A: movq rsi,xmm0
01AA019F: movq r15,xmm6
01AA01A4: mov [rbp-69h],r15
01AA01A8: mov [rsp+38h],rsi
01AA01AD: movaps [rbp-59h],xmm6
01AA01B1: cmp r13d,7FFFFFFFh
01AA01B8: je short 0000000001AA01E4h
01AA01BA: movdqa [rbp-59h],xmm6
01AA01BF: mov [rsp+28h],r13d
01AA01C4: mov [rsp+20h],r12
01AA01C9: mov r9,[rbp+7Fh]
01AA01CD: mov r8,r14
01AA01D0: lea rdx,[rbp-59h]
01AA01D4: lea rcx,[rbp-21h]
01AA01D8: call 0000000001A9FC80h ; ?simpleWrapTextUnsafe@ResizableText@agui@@SA?AV?$basic_string_view@DU?$char_traits@D@std@@@std@@V34@_KPEAVRichTextData@2@PEBVFont@2@H@Z
01AA01DD: movups xmm6,[rax]
01AA01E0: movaps [rbp-59h],xmm6
01AA01E4: xor r13d,r13d
01AA01E7: movdqa xmm0,xmm6
01AA01EB: psrldq xmm0,8
01AA01F0: movq rdi,xmm0
01AA01F5: cmp rdi,rsi
01AA01F8: je near ptr 0000000001AA030Bh
01AA01FE: movq rbx,xmm6
01AA0203: test rdi,rdi
01AA0206: je short 0000000001AA023Ch
01AA0208: nop dword ptr [rax+rax]
01AA0210: movzx ecx,byte ptr [rbx+rdi-1]
01AA0215: call 0000000001E494B0h ; isspace
01AA021A: test eax,eax
01AA021C: jne short 0000000001AA023Ch
01AA021E: cmp byte ptr [rbx+rdi-1],5Dh
01AA0223: je short 0000000001AA023Ch
01AA0225: lea rax,[rdi-1]
01AA0229: cmp rdi,rax
01AA022C: cmovb rax,rdi
01AA0230: mov rdi,rax
01AA0233: mov [rbp-51h],rax
01AA0237: test rax,rax
01AA023A: jne short 0000000001AA0210h
01AA023C: mov rsi,rdi
01AA023F: test rdi,rdi
01AA0242: je short 0000000001AA0292h
01AA0244: mov rax,rdi
01AA0247: mov r14,rdi
01AA024A: mov r15,rdi
01AA024D: mov r12,rdi
01AA0250: movzx ecx,byte ptr [rbx+rax-1]
01AA0255: call 0000000001E494B0h ; isspace
01AA025A: test eax,eax
01AA025C: je near ptr 0000000001AA02E7h
01AA0262: lea rdi,[r14-1]
01AA0266: cmp r15,rdi
01AA0269: cmovb rdi,r12
01AA026D: mov [rbp-51h],rdi
01AA0271: inc r13
01AA0274: mov rax,rdi
01AA0277: mov r14,rdi
01AA027A: mov r15,rdi
01AA027D: mov r12,rdi
01AA0280: test rdi,rdi
01AA0283: jne short 0000000001AA0250h
01AA0285: mov r14,[rsp+30h]
01AA028A: mov r15,[rbp-69h]
01AA028E: mov r12,[rbp+57h]
01AA0292: test r13,r13
01AA0295: je short 0000000001AA02CDh
01AA0297: cmp r13,rsi
01AA029A: jne short 0000000001AA02CDh
01AA029C: sub r14,r13
01AA029F: mov [rsp+30h],r14
01AA02A4: mov rsi,[rsp+38h]
01AA02A9: cmp rsi,r13
01AA02AC: jb near ptr 0000000001AA044Ch
01AA02B2: add r15,r13
01AA02B5: sub rsi,r13
01AA02B8: mov [rbp-79h],r15
01AA02BC: mov [rbp-71h],rsi
01AA02C0: movaps xmm6,[rbp-79h]
01AA02C4: mov r13d,[rbp+6Fh]
01AA02C8: jmp near ptr 0000000001AA01A4h
01AA02CD: xor r13d,r13d
01AA02D0: movaps [rbp-59h],xmm6
01AA02D4: movdqa xmm0,xmm6
01AA02D8: psrldq xmm0,8
01AA02DD: movq rdi,xmm0
01AA02E2: movq rbx,xmm6
01AA02E7: mov rsi,[rsp+38h]
01AA02EC: lea rdx,[rbp-59h]
01AA02F0: mov rcx,[rbp+67h]
01AA02F4: call 00000000015DB510h ; ??$_Emplace_one_at_back@AEBV?$basic_string_view@DU?$char_traits@D@std@@@std@@@?$vector@V?$basic_string_view@DU?$char_traits@D@std@@@std@@V?$allocator@V?$basic_string_view@DU?$char_traits@D@std@@@std@@@2@@std@@AEAAAEAV?$basic_string_view@DU?$char_traits@D@std@@@1@AEBV21@@Z | ??$_Emplace_one_at_back@AEBVVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@AEBV2@@Z | ??$_Emplace_one_at_back@VColor@agui@@@?$vector@VColor@agui@@V?$allocator@VColor@agui@@@std@@@std@@AEAAAEAVColor@agui@@$$QEAV23@@Z | ??$_Emplace_one_at_back@VVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@$$QEAV2@@Z
01AA02F9: mov rax,[rbp+7Fh]
01AA02FD: mov r12,[rbp+57h]
01AA0301: test rax,rax
01AA0304: je short 0000000001AA0311h
01AA0306: mov eax,[rax+8]
01AA0309: jmp short 0000000001AA031Bh
01AA030B: mov rbx,[rbp-59h]
01AA030F: jmp short 0000000001AA02ECh
01AA0311: mov rax,[r12]
01AA0315: mov rcx,r12
01AA0318: call qword ptr [rax+8]
01AA031B: mov edx,[rbp+5Fh]
01AA031E: add edx,eax
01AA0320: mov [rbp+5Fh],edx
01AA0323: cmp edx,[rbp+77h]
01AA0326: jg near ptr 0000000001AA0428h
01AA032C: cmp rdi,rsi
01AA032F: je near ptr 0000000001AA03AFh
01AA0335: xor edx,edx
01AA0337: lea rax,[rdi+rbx]
01AA033B: cmp rbx,rax
01AA033E: je short 0000000001AA036Ch
01AA0340: inc rdx
01AA0343: inc rbx
01AA0346: cmp rbx,rax
01AA0349: je short 0000000001AA036Ch
01AA034B: nop dword ptr [rax+rax]
01AA0350: movzx ecx,byte ptr [rbx]
01AA0353: test cl,cl
01AA0355: jns short 0000000001AA0367h
01AA0357: and cl,0C0h
01AA035A: cmp cl,0C0h
01AA035D: je short 0000000001AA0367h
01AA035F: inc rbx
01AA0362: cmp rax,rbx
01AA0365: jne short 0000000001AA0350h
01AA0367: cmp rbx,rax
01AA036A: jne short 0000000001AA0340h
01AA036C: lea rax,[rdx+r13]
01AA0370: mov r14,[rsp+30h]
01AA0375: sub r14,rax
01AA0378: mov [rsp+30h],r14
01AA037D: lea rdx,[rdi+r13]
01AA0381: cmp rsi,rdx
01AA0384: jb near ptr 0000000001AA044Ch
01AA038A: mov rcx,rsi
01AA038D: sub rcx,rdi
01AA0390: sub rcx,r13
01AA0393: sub rsi,rdx
01AA0396: mov rax,rsi
01AA0399: mov r15,[rbp-69h]
01AA039D: add r15,rdx
01AA03A0: mov rsi,rcx
01AA03A3: cmp rax,rcx
01AA03A6: cmovb rsi,rax
01AA03AA: jmp near ptr 0000000001AA02B8h
01AA03AF: mov rcx,[rbp-49h]
01AA03B3: add rcx,10h
01AA03B7: mov [rbp-49h],rcx
01AA03BB: cmp rcx,[rbp-29h]
01AA03BF: je short 0000000001AA03CDh
01AA03C1: mov [rbp+5Fh],edx
01AA03C4: mov r13d,[rbp+6Fh]
01AA03C8: jmp near ptr 0000000001AA0140h
01AA03CD: mov rcx,[rbp-41h]
01AA03D1: xor ebx,ebx
01AA03D3: test rcx,rcx
01AA03D6: je short 0000000001AA040Ah
01AA03D8: mov rdx,[rbp-31h]
01AA03DC: sub rdx,rcx
01AA03DF: and rdx,0FFFFFFFFFFFFFFF0h
01AA03E3: mov rax,rcx
01AA03E6: cmp rdx,1000h
01AA03ED: jb short 0000000001AA0404h
01AA03EF: add rdx,27h
01AA03F3: mov rcx,[rcx-8]
01AA03F7: sub rax,rcx
01AA03FA: sub rax,8
01AA03FE: cmp rax,1Fh
01AA0402: ja short 0000000001AA0433h
01AA0404: call 0000000001E2E4E4h ; ??3@YAXPEAX_K@Z | ??_V@YAXPEAX@Z
01AA0409: nop
01AA040A: mov eax,ebx
01AA040C: movaps xmm6,[rsp+0B0h]
01AA0414: add rsp,0C8h
01AA041B: pop r15
01AA041D: pop r14
01AA041F: pop r13
01AA0421: pop r12
01AA0423: pop rdi
01AA0424: pop rsi
01AA0425: pop rbx
01AA0426: pop rbp
01AA0427: ret
01AA0428: mov rcx,[rbp-41h]
01AA042C: mov ebx,1
01AA0431: jmp short 0000000001AA03D3h
01AA0433: mov qword ptr [rsp+20h],0
01AA043C: xor r9d,r9d
01AA043F: xor r8d,r8d
01AA0442: xor edx,edx
01AA0444: xor ecx,ecx
01AA0446: call 0000000001E42694h ; _invoke_watson
01AA044B: nop
01AA044C: call 000000000187B910h ; ?_Xran@?$basic_string_view@DU?$char_traits@D@std@@@std@@CAXXZ
01AA0451: int 3
01AA0452: int 3
01AA0453: int 3
01AA0454: int 3
01AA0455: int 3
01AA0456: int 3
01AA0457: int 3
01AA0458: int 3
01AA0459: int 3
01AA045A: int 3
01AA045B: int 3
01AA045C: int 3
01AA045D: int 3
01AA045E: int 3
01AA045F: int 3