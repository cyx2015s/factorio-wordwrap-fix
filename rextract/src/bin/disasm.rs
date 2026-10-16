// disasm - offline disassembly of a game function with call-target symbolization.
//
// Resolves a function (default: agui::ResizableText::multiMakeLinesStrView) from
// factorio.pdb, reads its bytes from factorio.exe, disassembles with iced-x86,
// and annotates every direct call with the name of the symbol it targets. That
// makes it easy to see which game functions the wrap routine calls (font width
// measurement, string helpers, the Result code, ...).
//
// usage: disasm <factorio.exe> <factorio.pdb> [rva-hex] [len-hex]
//   rva defaults to the resolved multiMakeLinesStrView; len defaults to the
//   distance to the next public symbol (capped at 0x1000 unless given).

use pdb::FallibleIterator;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let exe = PathBuf::from(args.next().ok_or("usage: disasm <exe> <pdb> [rva] [len]")?);
    let pdb_path = PathBuf::from(args.next().ok_or("missing pdb")?);

    let symbols = load_symbols(&pdb_path)?;
    println!("loaded {} public symbols", symbols.len());

    let rva = match args.next() {
        Some(s) => u32::from_str_radix(s.trim_start_matches("0x"), 16)?,
        None => resolve_rva(&symbols, "multiMakeLinesStrView")
            .ok_or("multiMakeLinesStrView not found in pdb")?,
    };
    let given_len = args
        .next()
        .map(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()?;

    // bound the disassembly at the next public symbol, capped at 0x1000 unless given
    let to_next = symbols
        .range((std::ops::Bound::Excluded(rva), std::ops::Bound::Unbounded))
        .next()
        .map(|(&k, _)| (k - rva) as usize)
        .unwrap_or(0x1000);
    let len = given_len.unwrap_or(to_next).min(to_next);

    let exe_bytes = std::fs::read(&exe)?;
    let (sec_va, _sec_size, raw_ptr, raw_size) = find_section(&exe_bytes, rva as u64)?;
    let off = raw_ptr + (rva as u64 - sec_va) as usize;
    let avail = raw_size.saturating_sub((rva as u64 - sec_va) as usize);
    let n = len.min(avail);
    let code = &exe_bytes[off..off + n];

    println!("disasm of 0x{rva:X} ({n} bytes):
");
    let mut dec = iced_x86::Decoder::with_ip(64, code, rva as u64, iced_x86::DecoderOptions::NONE);
    let mut instr = iced_x86::Instruction::default();
    let mut count = 0;
    while dec.can_decode() && count < 8000 {
        dec.decode_out(&mut instr);
        count += 1;
        let rip = instr.ip() as u32;
        let mnem = instr.mnemonic();
        let mut line = format!("{rip:08X}: {:16} {}", instr.len(), instr);
        if mnem == iced_x86::Mnemonic::Call {
            let kind = instr.op0_kind();
            let tgt = if kind == iced_x86::OpKind::NearBranch64
                || kind == iced_x86::OpKind::NearBranch32
            {
                Some(instr.near_branch_target() as u32)
            } else {
                None
            };
            if let Some(target) = tgt {
                if let Some(name) = symbolize(&symbols, target) {
                    line.push_str(&format!("    ; --> {name}"));
                } else {
                    line.push_str(&format!("    ; --> 0x{target:X}"));
                }
            }
        } else if mnem == iced_x86::Mnemonic::Ret {
            line.push_str("    ; (return)");
        }
        println!("{line}");
    }
    Ok(())
}

fn load_symbols(pdb_path: &std::path::Path) -> Result<BTreeMap<u32, String>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(pdb_path)?;
    let mut pdb = pdb::PDB::open(file)?;
    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;
    let mut map = BTreeMap::new();
    let mut iter = symbol_table.iter();
    while let Ok(Some(symbol)) = iter.next() {
        if let Ok(pdb::SymbolData::Public(data)) = symbol.parse() {
            if let Some(rva) = data.offset.to_rva(&address_map) {
                map.insert(rva.0, data.name.to_string().to_string());
            }
        }
    }
    Ok(map)
}

fn resolve_rva(symbols: &BTreeMap<u32, String>, term: &str) -> Option<u32> {
    symbols
        .iter()
        .find(|(_, n)| n.contains(term))
        .map(|(&rva, _)| rva)
}

fn symbolize(symbols: &BTreeMap<u32, String>, target: u32) -> Option<&str> {
    symbols
        .range(..=target)
        .next_back()
        .map(|(_, n)| n.as_str())
}

fn find_section(bytes: &[u8], rva: u64) -> Result<(u64, u64, usize, usize), Box<dyn std::error::Error>> {
    let e_lfanew = u32::from_le_bytes(bytes[0x3C..0x40].try_into()?) as usize;
    let nsec = u16::from_le_bytes(bytes[e_lfanew + 6..e_lfanew + 8].try_into()?) as usize;
    let opt_size = u16::from_le_bytes(bytes[e_lfanew + 20..e_lfanew + 22].try_into()?) as usize;
    let sec_start = e_lfanew + 24 + opt_size;
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
