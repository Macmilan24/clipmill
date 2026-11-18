# Run on a fresh Windows user with uv, Python and the repository checkout.
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $CandidateInstaller,
    [Parameter(Mandatory)] [string] $WorkDir,
    [string] $PreviousInstaller,
    [string] $Release050Installer
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$candidate = (Resolve-Path -LiteralPath $CandidateInstaller).Path
$version = (Get-Content "$root/apps/desktop/src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version
$registry = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ClipMill'
$publisher = (Get-Content "$root/apps/desktop/src-tauri/tauri.release.conf.json" -Raw | ConvertFrom-Json).bundle.publisher
$lastLocation = "HKCU:\Software\$publisher\ClipMill"
$desktop = Join-Path ([Environment]::GetFolderPath('DesktopDirectory')) 'ClipMill.lnk'
$start = Join-Path ([Environment]::GetFolderPath('Programs')) 'ClipMill.lnk'
$data = Join-Path $env:LOCALAPPDATA 'clipmill\ClipMill\data'
$webview = Join-Path $env:LOCALAPPDATA 'dev.clipmill.shell'
$settings = Join-Path $env:APPDATA 'dev.clipmill.shell'
foreach ($path in @($registry, $lastLocation, $desktop, $start, $data, $webview, $settings, $WorkDir)) {
    if (Test-Path -LiteralPath $path) { throw "The upgrade drill needs a fresh test user and work directory. Already present: $path" }
}
$work = (New-Item -ItemType Directory -Path $WorkDir).FullName
$install = Join-Path $work 'ClipMill with spaces'
$legacy = Join-Path $install 'clipmill.exe'
$studio = Join-Path $install 'clipmill-studio.exe'
$shell = New-Object -ComObject WScript.Shell

function Get-ReleaseInstaller([string]$Version, [string]$Path, [string]$Digest) {
    if (-not $Path) {
        $name = "ClipMill_${Version}_x64-setup.exe"
        $Path = Join-Path $work $name
        $null = Invoke-WebRequest -Uri "https://github.com/Macmilan24/clipmill/releases/download/v$Version/$name" -OutFile $Path
    }
    $file = (Resolve-Path -LiteralPath $Path).Path
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $file).Hash -ne $Digest) {
        throw "Installer does not match the published $Version Windows release."
    }
    return $file
}
$previous = Get-ReleaseInstaller '0.4.0' $PreviousInstaller '0cdc0a48a0baea3a1bd19c401a6e43bc704d1f1d7aba64ed21220f72f3e40109'
$release050 = Get-ReleaseInstaller '0.5.0' $Release050Installer '4bc154e03a15dd383f2e1fbcda5950767db062d49f0f157ed72d45f4c83004fc'

function Invoke-Setup([string]$Path, [string]$Options) {
    # NSIS requires /D= and _?= last, without quotes, even for a path with spaces.
    $process = Start-Process -FilePath $Path -ArgumentList $Options -PassThru
    if (-not $process.WaitForExit(300000)) { throw "Installer timed out: $Path $Options" }
    if ($process.ExitCode -ne 0) {
        throw "Installer exited with $($process.ExitCode) ($Path $Options)"
    }
}

function Assert-Registration([string]$Version, [string]$Main) {
    $registered = Get-ItemProperty -LiteralPath $registry
    if ($registered.DisplayVersion -ne $Version -or $registered.MainBinaryName -ne $Main -or
        $registered.InstallLocation.Trim('"') -ne $install) {
        throw "Unexpected registration: $($registered.DisplayVersion), $($registered.MainBinaryName), $($registered.InstallLocation)"
    }
    Write-Host "Registered $Version, $Main in $install"
}

function Assert-Link([string]$Path, [string]$Target, [string]$Description) {
    if (-not (Test-Path -LiteralPath $Path)) { throw "Missing shortcut: $Path" }
    $link = $shell.CreateShortcut($Path)
    if ($link.TargetPath -ne $Target) { throw "Shortcut $Path points to $($link.TargetPath), expected $Target" }
    if ($Description -and $link.Description -ne $Description) { throw "Shortcut was recreated: $Path" }
}

function Use-Project([string]$Action, [string]$Version) {
    uv run --project "$root/eval/harness" python "$PSScriptRoot/windows_upgrade_project.py" --app-dir $install $Action --work-dir $work --version $Version
    if ($LASTEXITCODE -ne 0) { throw "Could not $Action the existing project with ClipMill $Version" }
}

function Test-Client {
    python "$root/tools/release/verify_cli.py" $install
    if ($LASTEXITCODE -ne 0) { throw 'The installed CLI did not complete the MCP handshake.' }
}

Invoke-Setup $previous "/S /D=$install"
Assert-Registration '0.4.0' 'clipmill.exe'
$description = 'Retained from ClipMill 0.4'
foreach ($path in @($start, $desktop)) {
    Assert-Link $path $legacy
    $link = $shell.CreateShortcut($path)
    $link.Description = $description
    $link.Save()
}
Use-Project 'create' '0.4.0'

# A real mapped Studio image must block upgrading, including on a headless runner.
uv run --project "$root/eval/harness" python "$PSScriptRoot/windows_upgrade_project.py" --app-dir $install in-use --installer $candidate
if ($LASTEXITCODE -ne 0) { throw 'The installer failed the running Studio check.' }
Assert-Registration '0.4.0' 'clipmill.exe'

# /UPDATE reuses the recorded location and updates existing shortcuts in place.
Invoke-Setup $candidate '/S /UPDATE'
Assert-Registration $version 'clipmill-studio.exe'
foreach ($path in @($start, $desktop)) { Assert-Link $path $studio $description }
Test-Client
Use-Project 'reopen' $version

# Reproduce the missing CLI from the released 0.4 to 0.5 in-place transition.
Invoke-Setup "$install/uninstall.exe" "/S _?=$install"
Invoke-Setup $previous "/S /D=$install"
Invoke-Setup $release050 '/S /UPDATE'
Assert-Registration '0.5.0' 'clipmill-studio.exe'
if (Test-Path -LiteralPath $legacy) { throw 'The released 0.5 upgrade did not reproduce the missing CLI.' }
Assert-Link $desktop $studio
Remove-Item -LiteralPath $desktop
Invoke-Setup $candidate '/S /UPDATE'
Assert-Registration $version 'clipmill-studio.exe'
if (Test-Path -LiteralPath $desktop) { throw 'A removed desktop shortcut reappeared.' }
Assert-Link $start $studio
Test-Client
Use-Project 'reopen' $version

# Also exercise the uninstall-first path using the original uninstaller.
Invoke-Setup "$install/uninstall.exe" "/S _?=$install"
Invoke-Setup $previous "/S /D=$install"
Assert-Registration '0.4.0' 'clipmill.exe'
Invoke-Setup "$install/uninstall.exe" "/S _?=$install"
Invoke-Setup $candidate "/S /D=$install"
Assert-Registration $version 'clipmill-studio.exe'
foreach ($path in @($start, $desktop)) { Assert-Link $path $studio }
Test-Client
Use-Project 'reopen' $version
Invoke-Setup "$install/uninstall.exe" "/S _?=$install"
foreach ($path in @($registry, $start, $desktop)) {
    if (Test-Path -LiteralPath $path) { throw "Uninstaller left an active registration or shortcut: $path" }
}
# The uninstaller preserves preferences and projects by default. These paths
# all belonged to this drill, as verified before starting the first installer.
foreach ($path in @($lastLocation, $data, $settings, $webview, $install)) {
    if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force -Recurse }
}
Write-Host "Verified both published Windows upgrade paths and the uninstall-first installation."
