#![recursion_limit = "512"]
mod wrap;
use std::sync::{Mutex, Once};
static CONTROL: Mutex<()> = Mutex::new(());
static LOGGING: Once = Once::new();
fn run(action: fn() -> anyhow::Result<()>) -> u32 {
    let Ok(_guard) = CONTROL.lock() else { return 1 };
    LOGGING.call_once(|| {
        if let Ok(f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::env::temp_dir().join("factorio_wordwrap_fix.log"))
        {
            let _ = env_logger::Builder::new()
                .filter_level(log::LevelFilter::Info)
                .target(env_logger::Target::Pipe(Box::new(f)))
                .try_init();
        }
    });
    let result = action().and_then(|()| {
        unsafe {
            windows::Win32::System::Diagnostics::Debug::FlushInstructionCache(
                windows::Win32::System::Threading::GetCurrentProcess(),
                None,
                0,
            )
        }?;
        Ok(())
    });
    match result {
        Ok(()) => 0,
        Err(e) => {
            log::error!("{e:#}");
            1
        }
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_patch_apply() -> u32 {
    run(wrap::install)
}
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_patch_restore() -> u32 {
    run(wrap::uninstall)
}
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_observe() -> u32 {
    run(wrap::observe_install)
}
#[unsafe(no_mangle)]
pub extern "system" fn wordwrap_observe_stop() -> u32 {
    run(wrap::observe_stop)
}
// No work under the loader lock. The injector invokes an export after LoadLibrary.
#[unsafe(no_mangle)]
extern "system" fn DllMain(_: *mut core::ffi::c_void, _: u32, _: *mut core::ffi::c_void) -> i32 {
    1
}
mod boundary;
