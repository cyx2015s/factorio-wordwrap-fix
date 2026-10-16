//! Native wrapping stays in Factorio; only its boundary retreat is replaced.
use anyhow::{Context, Result, ensure};
use factorio_native_profile::{Function, Profile};
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

static PROFILE: OnceLock<Profile> = OnceLock::new();
static OBSERVER_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
static BOUNDARY_BYTES: OnceLock<[u8; 5]> = OnceLock::new();
fn profile() -> Result<&'static Profile> {
    if PROFILE.get().is_none() {
        let p = Profile::load(&std::env::current_exe()?)?;
        p.ensure_compatible()?;
        log::info!(
            "profile: multi RVA {:#x}, simple RVA {:#x}, normalized {}",
            p.multi.rva,
            p.simple.rva,
            p.multi.normalized_fingerprint
        );
        PROFILE
            .set(p)
            .map_err(|_| anyhow::anyhow!("profile already initialized"))?;
    }
    PROFILE.get().context("profile unavailable")
}
fn base_address() -> Result<usize> {
    Ok(unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleA(None) }?.0 as usize)
}
// Check the running image, not just the file inspected by the external injector.
// Only our own live five-byte detours may differ from the disk bytes.
fn native_address(function: &Function, is_multi: bool) -> Result<usize> {
    let p = profile()?;
    let base = base_address()?;
    let read32 = |off: usize| unsafe { ((base + off) as *const u32).read_unaligned() as usize };
    let nt = read32(0x3c);
    let opt = nt + 24;
    ensure!(
        read32(opt + 56) == p.image_size as usize,
        "running PE image size differs from inspected file"
    );
    let debug = read32(opt + 160);
    let len = read32(opt + 164);
    ensure!(
        len <= 4096 && debug + len <= p.image_size as usize,
        "invalid live debug directory"
    );
    let mut matched = false;
    for off in (debug..debug + len).step_by(28) {
        if read32(off + 12) == 2 {
            let cv = read32(off + 20);
            if cv + 24 <= p.image_size as usize {
                let bytes = unsafe { std::slice::from_raw_parts((base + cv) as *const u8, 24) };
                matched |= &bytes[..4] == b"RSDS"
                    && bytes[4..20] == p.identity.guid
                    && read32(cv + 20) == p.identity.age as usize;
            }
        }
    }
    ensure!(matched, "running PE GUID/age differs from inspected file");
    let addr = base + function.rva as usize;
    let mut live = unsafe { std::slice::from_raw_parts(addr as *const u8, function.size) }.to_vec();
    if is_multi {
        if MULTI.get().is_some_and(|h| h.is_enabled()) {
            ensure!(
                Some(&live[..5]) == OBSERVER_BYTES.get().map(|b| b.as_slice()),
                "observer entry modified by another patch"
            );
            live[..5].copy_from_slice(&function.code[..5]);
        }
        if BOUNDARY_HOOK.get().is_some_and(|h| h.is_enabled()) {
            let off = factorio_native_profile::RETREAT_OFFSET;
            ensure!(
                Some(&live[off..off + 5]) == BOUNDARY_BYTES.get().map(|b| b.as_slice()),
                "boundary entry modified by another patch"
            );
            live[off..off + 5].copy_from_slice(&function.code[off..off + 5]);
        }
    }
    ensure!(
        live == function.code,
        "running native code differs from inspected executable: {}",
        function.name
    );
    Ok(addr)
}
fn capture_five(addr: usize) -> [u8; 5] {
    unsafe { std::ptr::read_unaligned(addr as *const [u8; 5]) }
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
        let addr = native_address(&profile()?.simple, false)?;
        let _ = SIMPLE.set(unsafe { std::mem::transmute::<usize, Simple>(addr) });
    }
    if MULTI.get().is_none() {
        let addr = native_address(&profile()?.multi, true)?;
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
    let _ = OBSERVER_BYTES.set(capture_five(
        base_address()? + profile()?.multi.rva as usize,
    ));
    TRACE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn install() -> Result<()> {
    boundary_install()
}
pub fn uninstall() -> Result<()> {
    if let Some(hook) = BOUNDARY_HOOK.get() {
        native_address(&profile()?.multi, true)?;
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
    let func = native_address(&profile()?.multi, true)?;
    if BOUNDARY_HOOK.get().is_none() {
        let start = func + factorio_native_profile::RETREAT_OFFSET;
        let end = func + factorio_native_profile::CONTINUE_OFFSET;
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
    let _ = BOUNDARY_BYTES.set(capture_five(
        base_address()? + profile()?.retreat_rva as usize,
    ));
    if let Ok(mut seen) = SEEN.lock() {
        seen.clear();
    }
    log::info!("Unicode/rich-reference boundaries enabled");
    Ok(())
}
