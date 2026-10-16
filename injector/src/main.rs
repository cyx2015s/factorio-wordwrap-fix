use anyhow::{Context, Result, ensure};
use dll_syringe::{
    Syringe,
    process::{OwnedProcess, Process},
};
use factorio_native_profile::Profile;
use std::path::PathBuf;
fn main() -> Result<()> {
    let mut mode = "wordwrap_patch_apply";
    let mut dll = None;
    let mut pid = None;
    let mut inspect = None;
    let mut report = None;
    let mut symbol = None;
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
            "--inspect" => {
                inspect = Some(PathBuf::from(
                    args.next().context("--inspect requires factorio.exe")?,
                ))
            }
            "--report" => {
                report = Some(PathBuf::from(
                    args.next().context("--report requires a directory")?,
                ))
            }
            "--symbol" => symbol = Some(args.next().context("--symbol requires a name fragment")?),
            "--help" | "-h" => {
                println!(
                    "wordwrap-inject [--pid PID] [--restore|--observe|--trace-off] [--report DIR] [DLL]\nwordwrap-inject --inspect FACTORIO.EXE [--symbol QUERY] [--report DIR]"
                );
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
    if let Some(exe) = inspect {
        let profile = Profile::load(&exe)?;
        describe(&profile);
        if let Some(query) = symbol {
            for (rva, name) in profile.search(&query) {
                println!("RVA {rva:#010x} {name}");
            }
        }
        let dir = report.unwrap_or_else(|| PathBuf::from("wordwrap-diagnostics"));
        profile.write_report(&dir)?;
        println!("Report: {}", dir.display());
        return profile.ensure_compatible();
    }
    ensure!(symbol.is_none(), "--symbol requires --inspect");
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
    // Read-only preflight before LoadLibrary; the DLL independently checks the
    // mapped image too. Restore/off remain usable without a PDB on disk.
    if matches!(mode, "wordwrap_patch_apply" | "wordwrap_observe") || report.is_some() {
        let profile = Profile::load(&process.path()?)?;
        describe(&profile);
        if let Some(dir) = report {
            profile.write_report(&dir)?;
        } else if !profile.compatible {
            let dir = std::env::temp_dir().join("factorio-wordwrap-diagnostics");
            profile.write_report(&dir)?;
            eprintln!("Diagnostics: {}", dir.display());
        }
        profile.ensure_compatible()?;
    }
    let dll = dll
        .unwrap_or(std::env::current_exe()?.with_file_name("factorio_wordwrap_fix.dll"))
        .canonicalize()
        .context("build DLL first")?;
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
fn describe(p: &Profile) {
    println!(
        "PE/PDB matched: {} (age {})",
        p.exe.display(),
        p.identity.age
    );
    println!(
        "multi RVA {:#x} ({} bytes), simple RVA {:#x} ({} bytes), retreat {:#x} -> {:#x}",
        p.multi.rva, p.multi.size, p.simple.rva, p.simple.size, p.retreat_rva, p.continuation_rva
    );
    println!(
        "Native contract: {}",
        if p.compatible {
            "compatible; instruction bodies and callees verified"
        } else {
            "INCOMPATIBLE; inspect report"
        }
    );
}
