//! Native wrapping stays in Factorio; only its boundary retreat is replaced.
use anyhow::{Context, Result, ensure};
use pdb::FallibleIterator;
use retour::GenericDetour;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

type Multi = unsafe extern "C" fn(*const u8, *const u8, *mut usize, i32, i32, *const u8) -> i32;
static MULTI: OnceLock<GenericDetour<Multi>> = OnceLock::new();
static TRACE: AtomicBool = AtomicBool::new(false);
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
#[repr(C)]
struct View {
    ptr: *const u8,
    len: usize,
}
type Simple =
    unsafe extern "C" fn(*mut View, *const View, usize, *const u8, *const u8, i32) -> *mut View;
static SIMPLE: OnceLock<Simple> = OnceLock::new();

pub fn resolve(term: &str) -> Result<usize> {
    let exe = std::env::current_exe()?;
    let mut pdb = pdb::PDB::open(std::fs::File::open(exe.with_extension("pdb"))?)?;
    let info = pdb.pdb_information()?;
    let base = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleA(None) }?.0 as usize;
    // Refuse a stale/replaced PDB before using any of its addresses. CodeView
    // records in the loaded PE image identify the exact PDB GUID and age.
    let read32 = |off: usize| unsafe { ((base + off) as *const u32).read_unaligned() as usize };
    let nt = read32(0x3c);
    let opt = nt + 24;
    let image_size = read32(opt + 56);
    let debug = read32(opt + 112 + 6 * 8);
    let debug_size = read32(opt + 112 + 6 * 8 + 4);
    ensure!(
        debug_size <= 4096 && debug + debug_size <= image_size,
        "invalid PE debug directory"
    );
    let mut matched = false;
    for off in (debug..debug + debug_size).step_by(28) {
        if read32(off + 12) == 2 {
            let cv = read32(off + 20);
            if cv + 24 <= image_size {
                let bytes = unsafe { std::slice::from_raw_parts((base + cv) as *const u8, 24) };
                matched |= &bytes[..4] == b"RSDS"
                    && bytes[4..20] == info.guid.to_bytes_le()
                    && read32(cv + 20) == info.age as usize;
            }
        }
    }
    ensure!(
        matched,
        "factorio.pdb GUID/age does not match the running executable"
    );
    let table = pdb.global_symbols()?;
    let map = pdb.address_map()?;
    let mut iter = table.iter();
    let mut found = Vec::new();
    while let Some(symbol) = iter.next()? {
        if let Ok(pdb::SymbolData::Public(data)) = symbol.parse()
            && data.name.to_string().contains(term)
            && let Some(rva) = data.offset.to_rva(&map)
        {
            found.push(rva.0 as usize);
        }
    }
    found.sort_unstable();
    found.dedup();
    ensure!(found.len() == 1, "expected unique symbol {term}: {found:?}");
    ensure!(
        found[0] + 0x400 <= image_size,
        "symbol outside loaded image"
    );
    Ok(base + found[0])
}

unsafe extern "C" fn observe(
    font: *const u8,
    text: *const u8,
    out: *mut usize,
    width: i32,
    height: i32,
    rich: *const u8,
) -> i32 {
    let result = unsafe {
        MULTI
            .get()
            .unwrap()
            .call(font, text, out, width, height, rich)
    };
    if TRACE.load(Ordering::Relaxed) {
        // All views are borrowed from the still-live native call's input string.
        let size = unsafe { text.add(16).cast::<usize>().read() };
        if (25..=100_000).contains(&size) {
            let cap = unsafe { text.add(24).cast::<usize>().read() };
            let p = if cap > 15 {
                unsafe { text.cast::<*const u8>().read() }
            } else {
                text
            };
            let input = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(p, size) });
            let key = format!("{font:p}:{width}:{height}:{input}");
            if let Ok(mut seen) = SEEN.lock()
                && seen.len() < 4000
                && !seen.contains(&key)
            {
                seen.push(key);
                let begin = unsafe { out.read() };
                let end = unsafe { out.add(1).read() };
                let count = (end - begin) / 16;
                let mut lines = Vec::new();
                let mut overflows = Vec::new();
                for i in 0..count.min(1000) {
                    let v = (begin + i * 16) as *const usize;
                    let ptr = unsafe { v.read() } as *const u8;
                    let len = unsafe { v.add(1).read() };
                    let line =
                        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
                            .into_owned();
                    if let Some(simple) = SIMPLE.get().filter(|_| width > 0 && width < i32::MAX) {
                        let view = View { ptr, len };
                        let mut fitted = View {
                            ptr: std::ptr::null(),
                            len: 0,
                        };
                        unsafe {
                            simple(&mut fitted, &view, line.chars().count(), rich, font, width);
                        }
                        if fitted.len < len {
                            overflows.push(i);
                        }
                    }
                    lines.push(line);
                }
                log::info!(
                    "TRACE font={font:p} width={width} height={height} rich={} result={result} overflow={overflows:?} text={input:?} lines={lines:?}",
                    !rich.is_null()
                );
            }
        }
    }
    result
}

pub fn observe_install() -> Result<()> {
    if SIMPLE.get().is_none() {
        let addr = resolve("?simpleWrapTextUnsafe@ResizableText@agui@@")?;
        let _ = SIMPLE.set(unsafe { std::mem::transmute::<usize, Simple>(addr) });
    }
    if MULTI.get().is_none() {
        let addr = resolve("?multiMakeLinesStrView@ResizableText@agui@@")?;
        log::info!("native wrap @ {addr:#x}");
        let hook =
            unsafe { GenericDetour::new(std::mem::transmute::<usize, Multi>(addr), observe)? };
        MULTI
            .set(hook)
            .map_err(|_| anyhow::anyhow!("already initialized"))?;
    }
    unsafe {
        MULTI.get().context("hook not initialized")?.enable()?;
    }
    TRACE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn install() -> Result<()> {
    boundary_install()
}
pub fn uninstall() -> Result<()> {
    if let Some(hook) = BOUNDARY_HOOK.get() {
        unsafe {
            hook.disable()?;
        }
    }
    if let Ok(mut seen) = SEEN.lock() {
        seen.clear();
    }
    log::info!("native boundaries restored");
    Ok(())
}
pub fn observe_stop() -> Result<()> {
    TRACE.store(false, Ordering::Relaxed);
    if let Some(hook) = MULTI.get() {
        unsafe {
            hook.disable()?;
        }
    }
    Ok(())
}

// Verified 2.1.19 basic block: candidate retreat only. Relative isspace call
// displacement is masked; every instruction and the continuation are checked.
const RETREAT: &[u8] = &[
    0x48, 0x85, 0xff, 0x74, 0x34, 0x0f, 0x1f, 0x84, 0, 0, 0, 0, 0, 0x0f, 0xb6, 0x4c, 0x3b, 0xff,
    0xe8, 0, 0, 0, 0, 0x85, 0xc0, 0x75, 0x1e, 0x80, 0x7c, 0x3b, 0xff, 0x5d, 0x74, 0x17, 0x48, 0x8d,
    0x47, 0xff, 0x48, 0x3b, 0xf8, 0x48, 0x0f, 0x42, 0xc7, 0x48, 0x8b, 0xf8, 0x48, 0x89, 0x45, 0xaf,
    0x48, 0x85, 0xc0, 0x75, 0xd4,
];
static BOUNDARY_HOOK: OnceLock<retour::RawDetour> = OnceLock::new();
static CONTINUE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

unsafe extern "C" fn choose_boundary(
    text: *const u8,
    fit: usize,
    len: usize,
    rich: *const u8,
) -> usize {
    let bytes = unsafe { std::slice::from_raw_parts(text, len) };
    match std::str::from_utf8(bytes) {
        Ok(text) => crate::boundary::boundary(text, fit, !rich.is_null()),
        Err(_) => fit, // do not decode or alter malformed input
    }
}

// Entered by JMP inside multiMakeLinesStrView, with an already aligned RSP.
// Preserve Windows x64 volatile registers and 32-byte shadow space. RDI and
// the local string_view length are the only outputs. No guessed Font/STL ABI.
#[unsafe(naked)]
unsafe extern "C" fn boundary_stub() {
    core::arch::naked_asm!(
        "sub rsp, 0xc0",
        "mov [rsp+0x20], rax", "mov [rsp+0x28], rcx", "mov [rsp+0x30], rdx",
        "mov [rsp+0x38], r8", "mov [rsp+0x40], r9", "mov [rsp+0x48], r10", "mov [rsp+0x50], r11",
        "movdqu [rsp+0x60], xmm0", "movdqu [rsp+0x70], xmm1", "movdqu [rsp+0x80], xmm2",
        "movdqu [rsp+0x90], xmm3", "movdqu [rsp+0xa0], xmm4", "movdqu [rsp+0xb0], xmm5",
        "mov rcx, rbx", "mov rdx, rdi", "mov r8, rsi", "mov r9, [rbp+0x7f]",
        "call {choose}", "mov rdi, rax", "mov [rbp-0x51], rax",
        "movdqu xmm0, [rsp+0x60]", "movdqu xmm1, [rsp+0x70]", "movdqu xmm2, [rsp+0x80]",
        "movdqu xmm3, [rsp+0x90]", "movdqu xmm4, [rsp+0xa0]", "movdqu xmm5, [rsp+0xb0]",
        "mov rax, [rsp+0x20]", "mov rcx, [rsp+0x28]", "mov rdx, [rsp+0x30]",
        "mov r8, [rsp+0x38]", "mov r9, [rsp+0x40]", "mov r10, [rsp+0x48]", "mov r11, [rsp+0x50]",
        "add rsp, 0xc0", "jmp qword ptr [rip+{cont}]",
        choose = sym choose_boundary, cont = sym CONTINUE,
    );
}

fn boundary_install() -> Result<()> {
    if BOUNDARY_HOOK.get().is_none() {
        let func = resolve("?multiMakeLinesStrView@ResizableText@agui@@")?;
        let code = unsafe { std::slice::from_raw_parts(func as *const u8, 0x400) };
        // Interior hooks depend on register/local-variable liveness beyond the
        // overwritten bytes. Require the complete reviewed 2.1.19 body too.
        // Skip the first 16 bytes, which the optional observer may detour.
        let fingerprint = code[16..].iter().fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
        });
        ensure!(
            fingerprint == 0x08e1f8a33b7f69a9,
            "unsupported native wrap body {fingerprint:#x}; re-disassembly required"
        );
        ensure!(
            code[0x1b..0x26] == [0x48, 0x8d, 0x68, 0xb1, 0x48, 0x81, 0xec, 0xc8, 0, 0, 0],
            "unsupported native stack frame"
        );
        let matches: Vec<_> = code
            .windows(RETREAT.len())
            .enumerate()
            .filter(|(_, b)| {
                b.iter()
                    .zip(RETREAT)
                    .enumerate()
                    .all(|(i, (a, b))| (19..23).contains(&i) || a == b)
            })
            .map(|(i, _)| i)
            .collect();
        ensure!(
            matches.len() == 1,
            "unsupported native retreat block; refusing to patch: {matches:?}"
        );
        let start = func + matches[0];
        let end = start + RETREAT.len();
        ensure!(
            unsafe { std::slice::from_raw_parts(end as *const u8, 6) }
                == [0x48, 0x8b, 0xf7, 0x48, 0x85, 0xff],
            "unsupported continuation"
        );
        CONTINUE.store(end, Ordering::Release);
        let hook =
            unsafe { retour::RawDetour::new(start as *const (), boundary_stub as *const ())? };
        BOUNDARY_HOOK
            .set(hook)
            .map_err(|_| anyhow::anyhow!("boundary already initialized"))?;
        log::info!("boundary block @ {start:#x}, continuation {end:#x}");
    }
    unsafe {
        BOUNDARY_HOOK
            .get()
            .context("boundary not initialized")?
            .enable()?;
    }
    if let Ok(mut seen) = SEEN.lock() {
        seen.clear();
    }
    log::info!("Unicode/rich-reference boundaries enabled");
    Ok(())
}
