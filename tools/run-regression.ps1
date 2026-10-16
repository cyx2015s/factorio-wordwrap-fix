param(
    [string]$Factorio = 'D:\异星工厂\Factorio_2.1\bin\x64\factorio.exe',
    [int]$Port = 34210,
    [int]$GamePid = 0,
    [string]$Dll = ''
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$runtime = Join-Path $repo '.runtime'
New-Item -ItemType Directory -Force $runtime | Out-Null
# Serialize runners, and check Factorio's actual directory lock rather than
# inferring availability from a process list (which may have restricted visibility).
$runnerLock = [IO.File]::Open("$runtime/regression-runner.lock", 'OpenOrCreate', 'ReadWrite', 'None')
Push-Location $repo
try {
    if ($GamePid -eq 0) {
        try { $probe = [IO.File]::Open("$runtime/.lock", 'OpenOrCreate', 'ReadWrite', 'None'); $probe.Dispose() }
        catch { throw 'The regression directory is in use. Close that test instance, or use -GamePid PID -Dll <the exact DLL already loaded>. No game was launched.' }
        $data = Join-Path (Split-Path (Split-Path (Split-Path $Factorio -Parent) -Parent) -Parent) 'data'
        New-Item -ItemType Directory -Force "$runtime/mods", "$runtime/scenarios/wordwrap" | Out-Null
        Copy-Item "$PSScriptRoot/test-scenario/control.lua" "$runtime/scenarios/wordwrap/control.lua"
@"
[path]
read-data=$($data.Replace('\','/'))
write-data=$($runtime.Replace('\','/'))
[general]
locale=zh-CN
[graphics]
full-screen=false
[other]
show-tips-and-tricks=false
check-updates=false
"@ | Set-Content "$runtime/config.ini" -Encoding utf8
        '{"mods":[{"name":"base","enabled":true},{"name":"quality","enabled":true},{"name":"elevated-rails","enabled":true},{"name":"space-age","enabled":true}]}' | Set-Content "$runtime/mods/mod-list.json" -Encoding utf8
        cargo +nightly build --release --locked
        if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
        $Dll = Join-Path $runtime "factorio_wordwrap_fix-$([DateTime]::Now.ToString('yyyyMMdd-HHmmss')).dll"
        Copy-Item 'target/release/factorio_wordwrap_fix.dll' $Dll
        $launchArgs = @('--config', "`"$runtime/config.ini`"", '--mod-directory', "`"$runtime/mods`"", '--load-scenario','wordwrap','--enable-lua-udp', "$Port", '--disable-audio', '--window-size','1920x1200')
        $game = Start-Process -FilePath $Factorio -ArgumentList $launchArgs -PassThru -WindowStyle Hidden
        $GamePid = $game.Id
        $ready = $false
        for ($attempt=0; $attempt -lt 120; $attempt++) {
            Start-Sleep -Seconds 1
            if ($game.HasExited) { throw 'Factorio exited; inspect .runtime/factorio-current.log' }
            $tail = Get-Content "$runtime/factorio-current.log" -Tail 5 -ErrorAction SilentlyContinue
            if ($tail -match 'Checksum for script __level__/control.lua') { $ready=$true; break }
        }
        if (-not $ready) { throw 'Factorio startup timeout' }
    } else {
        if (-not $Dll) { throw '-GamePid requires -Dll pointing to the same DLL loaded by that test instance' }
        $gameProcess = Get-CimInstance Win32_Process -Filter "ProcessId = $GamePid"
        if (-not $gameProcess -or $gameProcess.Name -ne 'factorio.exe' -or -not $gameProcess.CommandLine.Contains($runtime)) {
            throw 'Refusing to reuse a process outside this isolated regression configuration'
        }
        $Dll = (Resolve-Path -LiteralPath $Dll).Path
    }
    python "$PSScriptRoot/run_matrix.py" --pid $GamePid --dll $Dll --injector "$repo/target/release/wordwrap-inject.exe" --log "$env:TEMP/factorio_wordwrap_fix.log" --output "$runtime/results" --port $Port
    if ($LASTEXITCODE -ne 0) { throw "Regression failed. Test PID $GamePid remains available for inspection; see .runtime/results and native log" }
    Write-Output "Passed. Game PID $GamePid remains open with fix enabled. DLL: $Dll. Evidence: $runtime/results"
} finally { Pop-Location; $runnerLock.Dispose() }
