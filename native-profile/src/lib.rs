//! Shared, read-only PE/PDB inspection used by the injector AND injected DLL.
//! Compatibility ignores relocation operands only, while checking their targets.
use anyhow::{Context, Result, bail, ensure};
use iced_x86::{Decoder, DecoderOptions, Mnemonic, OpKind};
use pdb::FallibleIterator;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub const MULTI: &str = "?multiMakeLinesStrView@ResizableText@agui@@";
pub const SIMPLE: &str = "?simpleWrapTextUnsafe@ResizableText@agui@@";
pub const RETREAT_OFFSET: usize = 0x1c3;
pub const CONTINUE_OFFSET: usize = 0x1fc;
const MULTI_REFERENCE: &[u8] = include_bytes!("../fixtures/multi-2.1.22.bin");
const SIMPLE_REFERENCE: &[u8] = include_bytes!("../fixtures/simple-2.1.22.bin");

#[derive(Clone, Debug, Serialize)]
pub struct Identity {
    pub guid: [u8; 16],
    pub age: u32,
}
#[derive(Clone, Debug)]
struct Section {
    va: u32,
    raw: usize,
    size: usize,
    executable: bool,
}
pub struct Image {
    bytes: Vec<u8>,
    sections: Vec<Section>,
    pub size: u32,
    pub identity: Identity,
}
fn u16_at(b: &[u8], n: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(n..n + 2).context("truncated PE u16")?.try_into()?,
    ))
}
fn u32_at(b: &[u8], n: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(n..n + 4).context("truncated PE u32")?.try_into()?,
    ))
}
impl Image {
    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        ensure!(bytes.get(..2) == Some(b"MZ"), "not a PE image");
        let nt = u32_at(&bytes, 0x3c)? as usize;
        ensure!(
            bytes.get(nt..nt + 4) == Some(b"PE\0\0") && u16_at(&bytes, nt + 4)? == 0x8664,
            "expected x64 PE"
        );
        let opt = nt + 24;
        ensure!(u16_at(&bytes, opt)? == 0x20b, "expected PE32+");
        ensure!(
            u16_at(&bytes, nt + 20)? >= 168,
            "optional header has no debug directory"
        );
        let size = u32_at(&bytes, opt + 56)?;
        let table = opt + u16_at(&bytes, nt + 20)? as usize;
        let mut sections = Vec::new();
        for i in 0..usize::from(u16_at(&bytes, nt + 6)?) {
            let off = table + i * 40;
            sections.push(Section {
                va: u32_at(&bytes, off + 12)?,
                raw: u32_at(&bytes, off + 20)? as usize,
                size: u32_at(&bytes, off + 16)? as usize,
                executable: u32_at(&bytes, off + 36)? & 0x20000000 != 0,
            });
        }
        let mut image = Self {
            bytes,
            sections,
            size,
            identity: Identity {
                guid: [0; 16],
                age: 0,
            },
        };
        let debug = u32_at(&image.bytes, opt + 160)?;
        let len = u32_at(&image.bytes, opt + 164)? as usize;
        ensure!(
            len > 0 && len <= 4096 && len.is_multiple_of(28),
            "invalid debug directory size"
        );
        let entries = image.read(debug, len)?.to_vec();
        let mut identities = Vec::new();
        for entry in entries.as_chunks::<28>().0 {
            if u32_at(entry, 12)? == 2 {
                let cv = image.read(u32_at(entry, 20)?, 24)?;
                if &cv[..4] == b"RSDS" {
                    identities.push(Identity {
                        guid: cv[4..20].try_into()?,
                        age: u32_at(cv, 20)?,
                    });
                }
            }
        }
        ensure!(identities.len() == 1, "expected one RSDS identity");
        image.identity = identities.remove(0);
        Ok(image)
    }
    pub fn read(&self, rva: u32, len: usize) -> Result<&[u8]> {
        let section = self
            .sections
            .iter()
            .find(|s| rva >= s.va && ((rva - s.va) as usize) < s.size)
            .context("RVA outside file-backed sections")?;
        let delta = (rva - section.va) as usize;
        ensure!(len <= section.size - delta, "RVA range crosses section");
        self.bytes
            .get(section.raw + delta..section.raw + delta + len)
            .context("truncated section data")
    }
    fn executable(&self, rva: u32, len: usize) -> bool {
        u64::from(rva) + len as u64 <= u64::from(self.size)
            && self
                .sections
                .iter()
                .any(|s| s.executable && rva >= s.va && (rva - s.va) as usize + len <= s.size)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Call {
    pub offset: usize,
    pub target_rva: u32,
    pub symbols: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Function {
    pub name: String,
    pub rva: u32,
    pub size: usize,
    pub raw_fingerprint: String,
    pub normalized_fingerprint: String,
    pub calls: Vec<Call>,
    pub differences: Vec<String>,
    pub assembly: String,
    #[serde(skip)]
    pub code: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct Profile {
    pub exe: PathBuf,
    pub pdb: PathBuf,
    pub identity: Identity,
    pub image_size: u32,
    pub compatible: bool,
    pub diagnostics: Vec<String>,
    pub multi: Function,
    pub simple: Function,
    pub retreat_rva: u32,
    pub continuation_rva: u32,
    #[serde(skip)]
    symbols: BTreeMap<u32, Vec<String>>,
}
fn hash(b: &[u8]) -> String {
    format!(
        "{:016x}",
        b.iter()
            .fold(0xcbf29ce484222325u64, |h, b| (h ^ u64::from(*b))
                .wrapping_mul(0x100000001b3))
    )
}
struct Decoded {
    normalized: Vec<u8>,
    calls: Vec<(usize, u32)>,
    rip: Vec<(usize, u32)>,
    assembly: Vec<(usize, String)>,
}
fn decode(code: &[u8], rva: u32) -> Result<Decoded> {
    let mut decoder = Decoder::with_ip(64, code, u64::from(rva), DecoderOptions::NONE);
    let mut normalized = code.to_vec();
    let mut calls = Vec::new();
    let mut rip = Vec::new();
    let mut assembly = Vec::new();
    while decoder.can_decode() {
        let offset = decoder.position();
        let ins = decoder.decode();
        ensure!(!ins.is_invalid(), "invalid instruction at +{offset:#x}");
        let constants = decoder.get_constant_offsets(&ins);
        if ins.mnemonic() == Mnemonic::Call && ins.op0_kind() == OpKind::NearBranch64 {
            ensure!(constants.immediate_size() == 4, "unexpected call encoding");
            normalized
                [offset + constants.immediate_offset()..offset + constants.immediate_offset() + 4]
                .fill(0);
            calls.push((offset, u32::try_from(ins.near_branch_target())?));
        }
        if ins.is_ip_rel_memory_operand() {
            ensure!(constants.displacement_size() == 4, "unexpected RIP operand");
            normalized[offset + constants.displacement_offset()
                ..offset + constants.displacement_offset() + 4]
                .fill(0);
            rip.push((offset, u32::try_from(ins.ip_rel_memory_address())?));
        }
        assembly.push((offset, format!("{:08X}: {ins}", ins.ip())));
    }
    Ok(Decoded {
        normalized,
        calls,
        rip,
        assembly,
    })
}
// Full callee identity is checked separately: zeroing addresses must not accept
// the same instruction stream retargeted to a different function.
fn expected_call(offset: usize) -> Option<&'static str> {
    match offset {
        0xbe | 0xd2 | 0x2b4 => Some(
            "??$_Emplace_one_at_back@AEBVVector@@@?$vector@VVector@@V?$allocator@VVector@@@std@@@std@@AEAAAEAVVector@@AEBV2@@Z",
        ),
        0x198 => Some(
            "?simpleWrapTextUnsafe@ResizableText@agui@@SA?AV?$basic_string_view@DU?$char_traits@D@std@@@std@@V34@_KPEAVRichTextData@2@PEBVFont@2@H@Z",
        ),
        0x1d5 | 0x215 => Some("isspace"),
        0x3c4 => Some("??3@YAXPEAX_K@Z"),
        0x406 => Some("_invoke_watson"),
        0x40c => Some("?_Xran@?$basic_string_view@DU?$char_traits@D@std@@@std@@CAXXZ"),
        _ => None,
    }
}
fn inspect_function(
    image: &Image,
    symbols: &BTreeMap<u32, Vec<String>>,
    term: &str,
    reference: &[u8],
    reference_rva: u32,
) -> Result<Function> {
    let found: Vec<_> = symbols
        .iter()
        .filter(|(_, names)| names.iter().any(|n| n.starts_with(term)))
        .collect();
    ensure!(
        found.len() == 1,
        "expected unique symbol {term}, found {}",
        found.len()
    );
    let (&rva, names) = found[0];
    let next = symbols
        .range((std::ops::Bound::Excluded(rva), std::ops::Bound::Unbounded))
        .next()
        .context("no function end symbol")?
        .0;
    let size = (next - rva) as usize;
    ensure!(
        (16..=16384).contains(&size),
        "implausible function size {size}"
    );
    ensure!(
        image.executable(rva, size),
        "symbol outside executable section"
    );
    let code = image.read(rva, size)?.to_vec();
    let d = decode(&code, rva)?;
    let approved = decode(reference, reference_rva)?;
    let mut differences = Vec::new();
    if d.normalized != approved.normalized {
        differences.push(format!(
            "instruction body differs (size {} versus {})",
            size,
            reference.len()
        ));
        for (off, (a, b)) in d
            .normalized
            .iter()
            .zip(&approved.normalized)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .take(16)
        {
            differences.push(format!("+{off:#x}: {b:02x} -> {a:02x}"));
        }
    }
    let calls: Vec<_> = d
        .calls
        .iter()
        .map(|&(offset, target_rva)| Call {
            offset,
            target_rva,
            symbols: symbols.get(&target_rva).cloned().unwrap_or_default(),
        })
        .collect();
    for call in &calls {
        if term != MULTI
            || expected_call(call.offset)
                .is_none_or(|expected| !call.symbols.iter().any(|n| n == expected))
        {
            differences.push(format!(
                "unapproved call +{:#x} -> {:#x}: {:?}",
                call.offset, call.target_rva, call.symbols
            ));
        }
    }
    if d.calls.len() != approved.calls.len() {
        differences.push("direct call count changed".into());
    }
    // The reviewed simple wrapper only references double 1.0 through RIP. Its
    // address may move; changing the value must still fail compatibility checks.
    for &(offset, target) in &d.rip {
        if term != SIMPLE || image.read(target, 8)? != 1.0f64.to_le_bytes() {
            differences.push(format!("unapproved RIP constant at +{offset:#x}"));
        }
    }
    if d.rip.len() != approved.rip.len() {
        differences.push("RIP reference count changed".into());
    }
    let assembly = d
        .assembly
        .into_iter()
        .map(|(offset, mut line)| {
            if let Some(call) = calls.iter().find(|c| c.offset == offset) {
                line.push_str(&format!(" ; {}", call.symbols.join(" | ")));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Function {
        name: names
            .iter()
            .find(|n| n.starts_with(term))
            .context("symbol name")?
            .clone(),
        rva,
        size,
        raw_fingerprint: hash(&code),
        normalized_fingerprint: hash(&d.normalized),
        calls,
        differences,
        assembly,
        code,
    })
}
impl Profile {
    pub fn load(exe: &Path) -> Result<Self> {
        let exe = exe.canonicalize()?;
        let pdb_path = exe.with_extension("pdb");
        let image = Image::parse(std::fs::read(&exe)?)?;
        let mut pdb =
            pdb::PDB::open(std::fs::File::open(&pdb_path).context("open matching factorio.pdb")?)?;
        let info = pdb.pdb_information()?;
        ensure!(
            info.guid.to_bytes_le() == image.identity.guid && info.age == image.identity.age,
            "exe/PDB GUID or age mismatch"
        );
        let table = pdb.global_symbols()?;
        let map = pdb.address_map()?;
        let mut it = table.iter();
        let mut symbols: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        while let Some(s) = it.next()? {
            if let Ok(pdb::SymbolData::Public(data)) = s.parse()
                && let Some(rva) = data.offset.to_rva(&map)
            {
                symbols
                    .entry(rva.0)
                    .or_default()
                    .push(data.name.to_string().into_owned());
            }
        }
        for names in symbols.values_mut() {
            names.sort();
            names.dedup();
        }
        let multi = inspect_function(&image, &symbols, MULTI, MULTI_REFERENCE, 0x1aa0040)?;
        let simple = inspect_function(&image, &symbols, SIMPLE, SIMPLE_REFERENCE, 0x1a9fc80)?;
        let diagnostics = multi
            .differences
            .iter()
            .map(|s| format!("multi: {s}"))
            .chain(simple.differences.iter().map(|s| format!("simple: {s}")))
            .collect::<Vec<_>>();
        let retreat_rva = multi.rva + RETREAT_OFFSET as u32;
        let continuation_rva = multi.rva + CONTINUE_OFFSET as u32;
        Ok(Self {
            exe,
            pdb: pdb_path,
            identity: image.identity,
            image_size: image.size,
            compatible: diagnostics.is_empty(),
            diagnostics,
            multi,
            simple,
            retreat_rva,
            continuation_rva,
            symbols,
        })
    }
    pub fn ensure_compatible(&self) -> Result<()> {
        if !self.compatible {
            bail!(
                "unsupported native ABI/instructions: {}",
                self.diagnostics.join("; ")
            );
        }
        Ok(())
    }
    pub fn search(&self, query: &str) -> Vec<(u32, &str)> {
        self.symbols
            .iter()
            .flat_map(|(&rva, n)| {
                n.iter()
                    .filter(move |n| n.contains(query))
                    .map(move |n| (rva, n.as_str()))
            })
            .collect()
    }
    pub fn write_report(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::write(dir.join("profile.json"), serde_json::to_vec_pretty(self)?)?;
        std::fs::write(dir.join("multi.asm"), &self.multi.assembly)?;
        std::fs::write(dir.join("simple.asm"), &self.simple.assembly)?;
        std::fs::write(dir.join("multi.bin"), &self.multi.code)?;
        std::fs::write(dir.join("simple.bin"), &self.simple.code)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic(code: &[u8], rva: u32, term: &str) -> (Image, BTreeMap<u32, Vec<String>>) {
        let decoded = decode(code, rva).unwrap();
        let mut bytes = code.to_vec();
        let mut sections = vec![Section {
            va: rva,
            raw: 0,
            size: code.len(),
            executable: true,
        }];
        let mut symbols = BTreeMap::from([
            (rva, vec![term.to_owned()]),
            (rva + code.len() as u32, vec!["end".into()]),
        ]);
        for (offset, target) in decoded.calls {
            symbols.insert(target, vec![expected_call(offset).unwrap().into()]);
        }
        for (_, target) in decoded.rip {
            let raw = bytes.len();
            bytes.extend_from_slice(&1.0f64.to_le_bytes());
            sections.push(Section {
                va: target,
                raw,
                size: 8,
                executable: false,
            });
        }
        (
            Image {
                bytes,
                sections,
                size: u32::MAX,
                identity: Identity {
                    guid: [0; 16],
                    age: 1,
                },
            },
            symbols,
        )
    }
    #[test]
    fn approved_bodies_pass_full_validation() {
        for (code, rva, term) in [
            (MULTI_REFERENCE, 0x1aa0040, MULTI),
            (SIMPLE_REFERENCE, 0x1a9fc80, SIMPLE),
        ] {
            let (image, symbols) = synthetic(code, rva, term);
            assert!(
                inspect_function(&image, &symbols, term, code, rva)
                    .unwrap()
                    .differences
                    .is_empty()
            );
        }
    }
    #[test]
    fn wrong_call_target_is_rejected_despite_identical_normalized_bytes() {
        let (mut image, symbols) = synthetic(MULTI_REFERENCE, 0x1aa0040, MULTI);
        image.bytes[0x1d6] ^= 1;
        let f = inspect_function(&image, &symbols, MULTI, MULTI_REFERENCE, 0x1aa0040).unwrap();
        assert_eq!(
            decode(MULTI_REFERENCE, 0x1aa0040).unwrap().normalized,
            decode(&f.code, 0x1aa0040).unwrap().normalized
        );
        assert!(f.differences.iter().any(|s| s.contains("unapproved call")));
    }
    #[test]
    fn relocation_with_same_callees_passes() {
        let (mut image, mut symbols) = synthetic(MULTI_REFERENCE, 0x1aa0040, MULTI);
        for (offset, target) in decode(MULTI_REFERENCE, 0x1aa0040).unwrap().calls {
            let new_target = target + 0x100000;
            let displacement =
                (i64::from(new_target) - i64::from(0x1aa0040 + offset as u32 + 5)) as i32;
            image.bytes[offset + 1..offset + 5].copy_from_slice(&displacement.to_le_bytes());
            symbols.insert(new_target, symbols[&target].clone());
        }
        assert!(
            inspect_function(&image, &symbols, MULTI, MULTI_REFERENCE, 0x1aa0040)
                .unwrap()
                .differences
                .is_empty()
        );
    }
    #[test]
    fn changed_rip_constant_is_rejected() {
        let (mut image, symbols) = synthetic(SIMPLE_REFERENCE, 0x1a9fc80, SIMPLE);
        image.bytes[SIMPLE_REFERENCE.len()..SIMPLE_REFERENCE.len() + 8].fill(0);
        assert!(
            inspect_function(&image, &symbols, SIMPLE, SIMPLE_REFERENCE, 0x1a9fc80)
                .unwrap()
                .differences
                .iter()
                .any(|s| s.contains("RIP constant"))
        );
    }
    #[test]
    fn duplicate_function_symbol_is_rejected() {
        let (image, mut symbols) = synthetic(MULTI_REFERENCE, 0x1aa0040, MULTI);
        symbols.insert(0x2000000, vec![MULTI.into()]);
        assert!(inspect_function(&image, &symbols, MULTI, MULTI_REFERENCE, 0x1aa0040).is_err());
    }
    #[test]
    fn code_outside_mapped_image_is_rejected() {
        let (mut image, symbols) = synthetic(MULTI_REFERENCE, 0x1aa0040, MULTI);
        image.size = 0x1000;
        assert!(inspect_function(&image, &symbols, MULTI, MULTI_REFERENCE, 0x1aa0040).is_err());
    }
    #[test]
    fn historical_relocation_migration() {
        let old = include_bytes!("../fixtures/multi-2.1.19-reconstructed.bin");
        assert_eq!(hash(&old[16..0x400]), "08e1f8a33b7f69a9");
        assert_ne!(old.as_slice(), MULTI_REFERENCE);
        assert_eq!(
            decode(old, 0x1a89fe0).unwrap().normalized,
            decode(MULTI_REFERENCE, 0x1aa0040).unwrap().normalized
        );
        let older = include_bytes!("../fixtures/multi-2.1.15-reconstructed.bin");
        assert_eq!(
            decode(older, 0x1a824e0).unwrap().normalized,
            decode(old, 0x1a89fe0).unwrap().normalized
        );
    }
    #[test]
    fn semantic_change_is_not_masked() {
        let mut b = MULTI_REFERENCE.to_vec();
        b[0x1e2] = 0x73;
        assert_ne!(
            decode(&b, 0x1aa0040).unwrap().normalized,
            decode(MULTI_REFERENCE, 0x1aa0040).unwrap().normalized
        );
    }
    #[test]
    fn malformed_images_fail() {
        for b in [vec![], b"MZ".to_vec(), vec![0; 512]] {
            assert!(Image::parse(b).is_err());
        }
    }
}
