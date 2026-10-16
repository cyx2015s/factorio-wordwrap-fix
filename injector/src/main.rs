use anyhow::{Context, Result, ensure};
use dll_syringe::{Syringe, process::OwnedProcess};
use std::path::PathBuf;
fn main() -> Result<()> {
    let mut mode = "wordwrap_patch_apply";
    let mut dll = None;
    let mut pid = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--restore" => mode = "wordwrap_patch_restore",
            "--observe" => mode = "wordwrap_observe",
            "--trace-off" => mode = "wordwrap_observe_stop",
            "--pid" => {
                pid = Some(
                    args.next()
                        .context("--pid requires an integer")?
                        .parse::<u32>()?,
                )
            }
            "--help" | "-h" => {
                println!("wordwrap-inject [--pid PID] [--restore|--observe|--trace-off] [DLL]");
                return Ok(());
            }
            _ => {
                ensure!(
                    !arg.starts_with('-') && dll.is_none(),
                    "unexpected argument {arg}"
                );
                dll = Some(PathBuf::from(arg));
            }
        }
    }
    let dll = dll
        .unwrap_or(std::env::current_exe()?.with_file_name("factorio_wordwrap_fix.dll"))
        .canonicalize()
        .context("build DLL first")?;
    let process = if let Some(pid) = pid {
        OwnedProcess::from_pid(pid)?
    } else {
        let mut processes = OwnedProcess::find_all_by_name("factorio.exe");
        ensure!(
            processes.len() == 1,
            "expected one Factorio process, found {}; use --pid PID",
            processes.len()
        );
        processes.pop().context("Factorio is not running")?
    };
    let syringe = Syringe::for_process(process);
    let module = syringe.find_or_inject(&dll)?;
    let call = unsafe { syringe.get_raw_procedure::<extern "system" fn() -> u32>(module, mode)? }
        .context("DLL export missing")?;
    let status = call.call()?;
    ensure!(
        status == 0,
        "{mode} failed ({status}); see %TEMP%/factorio_wordwrap_fix.log"
    );
    println!("{mode}: success; log: %TEMP%/factorio_wordwrap_fix.log");
    Ok(())
}
