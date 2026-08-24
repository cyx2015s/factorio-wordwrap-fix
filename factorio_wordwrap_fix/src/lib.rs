// factorio_wordwrap_fix — an injected DLL that teaches Factorio's GUI word-wrap
// to break lines inside runs of CJK (Chinese/Japanese/Korean) text.
//
// WHY
// ===
// Factorio's GUI widget library ("agui") wraps text with
// agui::ResizableText::multiMakeLinesStrView. That function only breaks a
// line at a character for which std::isspace() is true, plus one special
// case: the ASCII ']' (0x5D). Consecutive CJK characters are none of those,
// so a whole run of Chinese is treated as a single unbreakable "word". The
// line ends up far wider than the widget, and the text overflows / is clipped
// instead of wrapping.
//
// THE FIX
// =======
// Inside multiMakeLinesStrView there is one 7-byte test that decides whether
// the character just before the current position is a legal break point:
//
//   vanilla  (original bytes)
//     cmp byte ptr [reg+disp], 0x5D   ; 0x80 0x7C 0x3B 0xFF  5D   — is it ']'?
//     je   short +0x17                ; 74  17                  — break here
//
//   patched  (our bytes — same length, in place)
//     cmp byte ptr [reg+disp], 0x80   ; 0x80 0x7C 0x3B 0xFF  80   — non-ASCII?
//     jae  short +0x17                ; 73  17                  — break here
//
// cmp ... , 0x5D ; je  (== ']') becomes  cmp ... , 0x80 ; jae  (>= 0x80).
// Every UTF-8 lead byte of a CJK character is >= 0x80, so any non-ASCII
// character becomes a legal break point and the Chinese text wraps normally.
// ASCII (0x00-0x7F) still falls through to the original isspace retreat, so
// English wrapping is untouched.
//
// This is an in-memory patch: it is applied to the running process only and
// is gone when the game exits. Nothing on disk is modified.
//
// The patch-point technique and the ORIG/PATCH bytes were found by disassembling
// factorio 2.0.77 (Windows x64) in the reference F:/rust-crates/factorio-3d-mod
// project (tools/frida_wrap_fix.py). At load time this DLL re-locates the patch
// point itself: it resolves multiMakeLinesStrView from the bundled factorio.pdb
// and scans that function for the 7-byte pattern, so a game update only needs
// the pattern to still exist (it has been stable across 2.0.77 and 2.1.15).
// See README.md for the full breakdown.

use anyhow::{Context, Result};
use pdb::FallibleIterator;
use std::path::PathBuf;
use windows::Win32::Foundation::{BOOL, TRUE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{
    VirtualProtect, VirtualQuery, MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE,
    PAGE_GUARD, PAGE_NOACCESS, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

// ---------------------------------------------------------------------------
// The wrap patch point. This DLL re-locates it at load time via find_patch_addr:
// it looks up multiMakeLinesStrView in factorio.pdb and scans the function for
// the ORIG/PATCH byte pattern. PATCH_RVA is only the fallback for builds whose
// pdb is missing on disk (verified for 2.1.15).
// ---------------------------------------------------------------------------

/// Symbol-name fragment used to find the wrap function in factorio.pdb.
const WRAP_SYMBOL: &str = "multiMakeLinesStrView";

/// How many bytes after the resolved function start to scan for the pattern.
const SCAN_WINDOW: usize = 0x1000;

/// Fallback RVA (offset from factorio.exe base) of the cmp byte[..],']' ; je
/// test inside agui::ResizableText::multiMakeLinesStrView (factorio 2.1.15).
const PATCH_RVA: usize = 0x1A826BE;

/// Original 7 bytes at the patch point (vanilla: cmp byte[..],0x5D ; je).
const ORIG: [u8; 7] = [0x80, 0x7C, 0x3B, 0xFF, 0x5D, 0x74, 0x17];

/// Patched 7 bytes (cmp byte[..],0x80 ; jae) — non-ASCII becomes a break point.
const PATCH: [u8; 7] = [0x80, 0x7C, 0x3B, 0xFF, 0x80, 0x73, 0x17];

/// The result of looking at the target bytes without having changed anything.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Vanilla, // bytes matched ORIG — patch is available
    Patched, // bytes matched PATCH — fix already applied
    Unknown, // matched neither — this factorio build moved / is unknown
}

impl State {
    fn describe(self) -> &'static str {
        match self {
            State::Vanilla => "vanilla (original isspace+']' behaviour)",
            State::Patched => "already patched (CJK break points)",
            State::Unknown => "unknown (neither original nor patched bytes)",
        }
    }
}

/// Base address of the main module of the current process — i.e. factorio.exe
/// once this DLL has been injected into it.
fn base_address() -> Result<usize> {
    let handle = unsafe { GetModuleHandleA(None) }.context("GetModuleHandleA(None) failed")?;
    Ok(handle.0 as usize)
}

// ---------------------------------------------------------------------------
// Locating the patch point in the running build
// ---------------------------------------------------------------------------

/// True if [ptr, ptr+len) is committed, readable memory.
fn is_readable(ptr: *const u8, len: usize) -> bool {
    if ptr.is_null() || len == 0 {
        return false;
    }
    const READABLE_PROT: u32 = 0xEE;
    let mut checked = 0usize;
    while checked < len {
        let p = unsafe { ptr.add(checked) };
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        let r = unsafe {
            VirtualQuery(
                Some(p as *const core::ffi::c_void),
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if r == 0 || mbi.State != MEM_COMMIT {
            return false;
        }
        let prot = mbi.Protect.0;
        if prot & (PAGE_NOACCESS.0 | PAGE_GUARD.0) != 0 || prot & READABLE_PROT == 0 {
            return false;
        }
        let region_end = mbi.BaseAddress as usize + mbi.RegionSize;
        checked = region_end.saturating_sub(ptr as usize);
    }
    true
}

/// Offset of the first occurrence of ORIG or PATCH inside [start, start+len),
/// or None if neither is present. Recognising both states keeps the fix
/// idempotent across re-injection and lets us re-locate the patch point.
fn scan_either(start: usize, len: usize) -> Option<usize> {
    if len < ORIG.len() || !is_readable(start as *const u8, len) {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(start as *const u8, len) };
    for i in 0..=(bytes.len() - ORIG.len()) {
        let w = &bytes[i..i + ORIG.len()];
        if w == ORIG || w == PATCH {
            return Some(i);
        }
    }
    None
}

/// Where factorio.pdb sits next to the running factorio.exe.
fn find_pdb() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?.to_path_buf();
    let candidates = [
        exe_dir.join("factorio.pdb"),
        exe_dir.join("bin/x64/factorio.pdb"),
        PathBuf::from("factorio.pdb"),
        PathBuf::from("bin/x64/factorio.pdb"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

/// RVA of agui::ResizableText::multiMakeLinesStrView, read from factorio.pdb.
fn resolve_wrap_rva() -> Option<usize> {
    let pdb_path = find_pdb()?;
    let file = std::fs::File::open(&pdb_path).ok()?;
    let mut pdb = pdb::PDB::open(file).ok()?;
    let symbol_table = pdb.global_symbols().ok()?;
    let address_map = pdb.address_map().ok()?;
    let mut iter = symbol_table.iter();
    while let Ok(Some(symbol)) = iter.next() {
        if let Ok(pdb::SymbolData::Public(data)) = symbol.parse() {
            let name = data.name.to_string().to_string();
            if name.contains(WRAP_SYMBOL) {
                if let Some(rva) = data.offset.to_rva(&address_map) {
                    return Some(rva.0 as usize);
                }
            }
        }
    }
    None
}

/// Absolute address of the wrap patch point for the running build.
///
/// 1. Resolve multiMakeLinesStrView from factorio.pdb and scan a window of that
///    function for the ORIG/PATCH byte pattern (survives game updates and
///    re-injection).
/// 2. Fall back to PATCH_RVA, which is verified to hold the pattern on 2.1.15.
/// 3. Neither matched → Err (the caller then refuses to touch an unknown build).
fn find_patch_addr() -> Result<usize> {
    let base = base_address()?;
    let mut addr: Option<usize> = None;

    if let Some(func_rva) = resolve_wrap_rva() {
        let func = base + func_rva;
        log::info!("resolved {WRAP_SYMBOL} in pdb @ 0x{func:X}");
        if let Some(off) = scan_either(func, SCAN_WINDOW) {
            addr = Some(func + off);
            log::info!("wrap patch point at func+0x{off:X} (rva 0x{:X})", func_rva + off);
        } else {
            log::warn!("no wrap pattern within {SCAN_WINDOW:#x} bytes of {WRAP_SYMBOL}");
        }
    } else {
        log::info!("{WRAP_SYMBOL} not found in factorio.pdb — using fallback rva");
    }

    if addr.is_none() {
        let cand = base + PATCH_RVA;
        if let Some(off) = scan_either(cand, ORIG.len()) {
            addr = Some(cand + off);
            log::info!("using fallback rva 0x{PATCH_RVA:X} (+0x{off:X})");
        } else {
            log::warn!("fallback rva 0x{PATCH_RVA:X} does not hold the wrap pattern");
        }
    }

    addr.ok_or_else(|| {
        anyhow::anyhow!(
            "could not locate the wrap patch point (unknown factorio build?) — re-extract PATCH_RVA"
        )
    })
}

/// The single write that mutates code memory: make the page writable, copy the
/// 7 goal bytes, put the old protection back. Returns the old protection for
/// logging.
unsafe fn write_code_bytes(ptr: *const u8, goal: &[u8; 7]) -> Result<PAGE_PROTECTION_FLAGS> {
    let mut old = PAGE_PROTECTION_FLAGS(0);
    // edition 2024: an 'unsafe fn' does not make its body unsafe — every call
    // to an unsafe function must live inside an explicit unsafe { } block.
    unsafe {
        VirtualProtect(
            ptr as *const core::ffi::c_void,
            goal.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old,
        )
        .context("VirtualProtect(EXECUTE_READWRITE) failed")?;
        std::ptr::copy_nonoverlapping(goal.as_ptr(), ptr as *mut u8, goal.len());
        VirtualProtect(ptr as *const core::ffi::c_void, goal.len(), old, &mut old)
            .context("VirtualProtect(restore) failed")?;
    }
    Ok(old)
}

// ---------------------------------------------------------------------------
// Core patch logic
// ---------------------------------------------------------------------------

/// Read the 7 bytes at the patch point, classify them, and (if asked) flip them
/// to the other known state. Returns a human-readable description of the outcome.
///
/// apply = true  → ensure the target holds PATCH (insert the fix).
/// apply = false → ensure the target holds ORIG (restore vanilla).
///
/// Idempotent: if the bytes already match the goal it reports success without
/// writing. It refuses to touch an unknown build (bytes match neither ORIG nor
/// PATCH) rather than silently corrupting a different version.
fn set_patch(apply: bool) -> Result<String> {
    let target = find_patch_addr()?;
    let ptr = target as *const u8;

    let mut cur = [0u8; 7];
    unsafe { std::ptr::copy_nonoverlapping(ptr, cur.as_mut_ptr(), 7) };
    let hex = |b: &[u8; 7]| b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ");

    let state = if cur == ORIG {
        State::Vanilla
    } else if cur == PATCH {
        State::Patched
    } else {
        State::Unknown
    };

    if state == State::Unknown {
        return Ok(format!(
            "REFUSED at 0x{target:X}: bytes [{0}] match neither ORIG nor PATCH. Unsupported factorio build — re-extract PATCH_RVA.",
            hex(&cur)
        ));
    }

    let goal_is_patch = apply;
    let is_goal = match state {
        State::Vanilla => !goal_is_patch,
        State::Patched => goal_is_patch,
        State::Unknown => false, // unreachable
    };

    if is_goal {
        return Ok(format!(
            "0x{target:X} already {}: bytes [{}]",
            state.describe(),
            hex(&cur)
        ));
    }

    let goal: &[u8; 7] = if goal_is_patch { &PATCH } else { &ORIG };
    let old = unsafe { write_code_bytes(ptr, goal) }
        .with_context(|| format!("writing patch at 0x{target:X} failed"))?;

    let mut after = [0u8; 7];
    unsafe { std::ptr::copy_nonoverlapping(ptr, after.as_mut_ptr(), 7) };
    Ok(format!(
        "0x{target:X}: wrote {} bytes [{}] -> [{}] (old protect {:?})",
        if goal_is_patch { "patched" } else { "vanilla" },
        hex(&cur),
        hex(&after),
        old
    ))
}

// ---------------------------------------------------------------------------
// Exported entry points (callable directly, in addition to the DLL-load flow)
// ---------------------------------------------------------------------------

/// Apply the CJK wrap fix. Returns 0 on success (or already-applied), 1 on error.
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_patch_apply() -> i32 {
    match set_patch(true) {
        Ok(msg) => {
            write_result_file(&msg);
            0
        }
        Err(e) => {
            eprintln!("wordwrap_patch_apply failed: {e:#}");
            1
        }
    }
}

/// Restore the original vanilla bytes (disable the fix). Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_patch_restore() -> i32 {
    match set_patch(false) {
        Ok(msg) => {
            write_result_file(&msg);
            0
        }
        Err(e) => {
            eprintln!("wordwrap_patch_restore failed: {e:#}");
            1
        }
    }
}

/// Windows entry point when the DLL is loaded. We do the work on a spawned
/// thread so we never hold the loader lock, and so a panic in our code cannot
/// corrupt the freshly-injected process.
#[unsafe(no_mangle)]
extern "system" fn DllMain(
    _hinst: *mut core::ffi::c_void,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(|| {
            if let Err(e) = start() {
                eprintln!("[factorio_wordwrap_fix] failed to start: {e:#}");
            }
        });
    }
    TRUE
}

// ---------------------------------------------------------------------------
// Startup
// ---------------------------------------------------------------------------

fn start() -> Result<()> {
    // Open a console window (it appears inside the injected game) plus a log file.
    let _ = unsafe { AllocConsole() };
    install_panic_hook();
    setup_logging()?;

    log::info!(
        "=== factorio_wordwrap_fix v{} loaded ===",
        env!("CARGO_PKG_VERSION")
    );

    let msg = set_patch(true)?; // apply the fix on load
    log::info!("{msg}");
    write_result_file(&msg);

    log::info!("factorio_wordwrap_fix: done.");
    Ok(())
}

/// Best-effort: also mirror the patch result to a file so it is still readable
/// after the console closes with the game.
fn write_result_file(msg: &str) {
    if let Some(dir) = log_dir() {
        let _ = std::fs::write(dir.join("factorio_wordwrap_fix.log"), msg.as_bytes());
    }
}

fn setup_logging() -> Result<()> {
    let dir = log_dir();
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    if let Some(dir) = &dir {
        if let Ok(file) = std::fs::File::create(dir.join("factorio_wordwrap_fix.log")) {
            builder.target(env_logger::Target::Pipe(Box::new(file)));
        }
    }
    builder.try_init()?;
    Ok(())
}

/// Directory for the DLL's log: %APPDATA%/Factorio when present.
fn log_dir() -> Option<std::path::PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = std::path::PathBuf::from(appdata).join("Factorio");
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

/// A panic inside a hooked / exported function must never unwind across the FFI
/// boundary (that aborts the whole game). Log the panic instead.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let msg = format!("PANIC [{:?}]: {info}\nbacktrace:\n{bt}", std::thread::current().id());
        eprintln!("{msg}");
        log::error!("{msg}");
    }));
}
