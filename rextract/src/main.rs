// rextract - locate the CJK word-wrap patch point in a given factorio.exe/pdb.
//
// 1) resolve agui::ResizableText::multiMakeLinesStrView RVA from the pdb
// 2) read the .text bytes of factorio.exe at that rva
// 3) disassemble (iced-x86) and report every cmp byte[..],0x5D instruction
//    (the ASCII ']' break-point special case) that is immediately followed by
//    a conditional jump (je/jz), together with the instruction bytes, so the
//    new PATCH_RVA can be read straight off.
//
// usage: rextract <factorio.exe> <factorio.pdb> [symbol-substring] [len-hex]
//   defaults: symbol-substring = "multiMakeLinesStrView", len = 0x700

use pdb::FallibleIterator;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let exe = PathBuf::from(args.next().ok_or("usage: rextract <exe> <pdb> [sym] [len]")?);
    let pdb_path = PathBuf::from(args.next().ok_or("missing pdb")?);
    let sym = args.next().unwrap_or_else(|| "multiMakeLinesStrView".to_string());
    let len = args.next().map(|s| usize::from_str_radix(&s, 16)).transpose()?.unwrap_or(0x700);

    // --- 1. resolve the symbol rva -------------------------------------------
    let base = resolve_rva(&pdb_path, &sym)?;
    println!("symbol '{sym}' at rva 0x{base:X}");

    // --- 2. map rva -> file offset, read the function bytes ------------------
    let exe_bytes = std::fs::read(&exe)?;
    let (section_va, section_va_size, raw_ptr, raw_size) = find_section(&exe_bytes, base)?;
    let off = raw_ptr + (base - section_va) as usize;
    let avail = raw_size.saturating_sub((base - section_va) as usize);
    let n = len.min(avail);
    let code = &exe_bytes[off..off + n];
    println!("reading {n:#x} bytes at file offset 0x{off:X}");

    // --- 3. disassemble, find cmp byte[..],0x5D followed by jcc --------------
    let mut dec = iced_x86::Decoder::with_ip(64, code, base as u64, iced_x86::DecoderOptions::NONE);
    let mut instr = iced_x86::Instruction::default();
    let mut prev_was_cmp_5d = false;
    while dec.can_decode() {
        dec.decode_out(&mut instr);
        let text = instr.to_string();
        let mnem = instr.mnemonic();
        let is_cmp_5d = mnem == iced_x86::Mnemonic::Cmp
            && instr.op_count() >= 2
            && instr.op0_kind() == iced_x86::OpKind::Memory
            && instr.op1_kind() == iced_x86::OpKind::Immediate8
            && instr.immediate8() == 0x5D;
        let is_jcc = mnem == iced_x86::Mnemonic::Je;
        if is_cmp_5d {
            prev_was_cmp_5d = true;
            let off = (instr.ip() - base) as usize;
            let patch_rva = base + off as u64;
            let raw: Vec<String> = code[off..(off + 7).min(code.len())]
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect();
            println!(
                "  CANDIDATE RVA 0x{patch_rva:X}  +0x{off:X}  {}   raw [{}]",
                text,
                raw.join(" ")
            );
        } else {
            if prev_was_cmp_5d && is_jcc {
                println!("     followed by: +0x{:X}  {}", instr.ip() - base, text);
            }
            prev_was_cmp_5d = false;
        }
    }
    Ok(())
}

fn resolve_rva(pdb_path: &std::path::Path, term: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(pdb_path)?;
    let mut pdb = pdb::PDB::open(file)?;
    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;
    let mut iter = symbol_table.iter();
    while let Some(symbol) = iter.next()? {
        if let Ok(pdb::SymbolData::Public(data)) = symbol.parse() {
            let name = data.name.to_string().to_string();
            if name.contains(term) {
                if let Some(rva) = data.offset.to_rva(&address_map) {
                    return Ok(rva.0 as u64);
                }
            }
        }
    }
    Err(format!("symbol '{term}' not found in pdb").into())
}

// minimal PE section lookup: find the section whose VA range contains rva.
fn find_section(bytes: &[u8], rva: u64) -> Result<(u64, u64, usize, usize), Box<dyn std::error::Error>> {
    let e_lfanew = u32::from_le_bytes(bytes[0x3C..0x40].try_into()?) as usize;
    let nsec = u16::from_le_bytes(bytes[e_lfanew + 6..e_lfanew + 8].try_into()?) as usize;
    let opt_size = u16::from_le_bytes(bytes[e_lfanew + 20..e_lfanew + 22].try_into()?) as usize;
    let sec_start = e_lfanew + 24 + opt_size; // after optional header
    for i in 0..nsec {
        let s = sec_start + i * 40;
        let virtual_size = u32::from_le_bytes(bytes[s + 8..s + 12].try_into()?) as u64;
        let virtual_address = u32::from_le_bytes(bytes[s + 12..s + 16].try_into()?) as u64;
        let raw_size = u32::from_le_bytes(bytes[s + 16..s + 20].try_into()?) as usize;
        let raw_ptr = u32::from_le_bytes(bytes[s + 20..s + 24].try_into()?) as usize;
        if rva >= virtual_address && rva < virtual_address + virtual_size {
            return Ok((virtual_address, virtual_size, raw_ptr, raw_size));
        }
    }
    Err("rva not in any section".into())
}
