<#
.SYNOPSIS
    Verify installed Windows Whisper on the native CPU and emulated Ivy Bridge and Nehalem CPUs.
.DESCRIPTION
    Run after packaged_smoke.py has installed the worker into DataDir. Requires
    PowerShell 7.2+, Python 3.11+, x64 LLVM, and a modern x64 Windows host. Retrieves
    Intel SDE from its publisher for this test under Intel's Simplified Software
    License. WorkDir caches the pinned inputs and receives whisper-windows-compat.json.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $AppDir,
    [Parameter(Mandatory)][string] $DataDir,
    [Parameter(Mandatory)][string] $WorkDir
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$work = (New-Item -ItemType Directory -Force -Path $WorkDir).FullName

function Get-PinnedFile([string] $Uri, [string] $Name, [string] $Sha256) {
    $path = Join-Path $work $Name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        $partial = "$path.partial-$PID"
        try {
            Invoke-WebRequest -Uri $Uri -OutFile $partial -MaximumRetryCount 3
            if ((Get-FileHash -LiteralPath $partial -Algorithm SHA256).Hash -ne $Sha256) {
                throw "Digest mismatch downloading $Name from $Uri"
            }
            Move-Item -LiteralPath $partial -Destination $path
        } finally {
            if (Test-Path -LiteralPath $partial) { Remove-Item -LiteralPath $partial }
        }
    }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $Sha256) {
        throw "Cached $path does not match its published SHA-256"
    }
    return $path
}

# Resolve the current first-run environment, including its content-addressed directory.
$envs = Join-Path $DataDir 'engine/envs'
$pointer = Get-Content -Raw -LiteralPath (Join-Path $envs 'asr-whispercpp.json') | ConvertFrom-Json
if ($pointer.dir -cnotmatch '^asr-whispercpp-[0-9a-f]{12}$' -or
    $pointer.digest -cnotmatch '^[0-9a-f]{64}$' -or
    $pointer.dir -cne ('asr-whispercpp-' + $pointer.digest.Substring(0, 12))) {
    throw 'Unexpected asr-whispercpp install pointer'
}
$worker = (Get-Item -LiteralPath (Join-Path $envs "$($pointer.dir)/Scripts/python.exe")).FullName
$manifests = @(Get-ChildItem -LiteralPath $AppDir -Filter engine.json -Recurse -File)
if ($manifests.Count -ne 1) { throw "Expected one installed engine manifest, found $($manifests.Count)" }
$manifest = Get-Content -Raw -LiteralPath $manifests[0].FullName | ConvertFrom-Json
$part = @($manifest.parts | Where-Object name -eq 'asr-whispercpp')
if ($part.Count -ne 1) { throw 'Missing installed Whisper engine part' }
# The daemon keys an installed environment by its Python, requirements, and ordered wheels.
$fields = @('clipmill.engine.part.v1', $manifest.python, $part[0].requirements_sha256) +
    @($part[0].wheels | ForEach-Object { $_.sha256 })
$key = [Text.Encoding]::UTF8.GetBytes(($fields -join [char]0))
$partDigest = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($key)).ToLowerInvariant()
if ($partDigest -cne $pointer.digest) { throw 'Active worker belongs to a different engine manifest' }
$entries = @($part[0].wheels | Where-Object file -clike 'wheels/pywhispercpp-*-win_amd64.whl')
if ($entries.Count -ne 1) { throw 'Expected one Windows x64 Whisper wheel in the installed manifest' }
$wheel = (Get-Item -LiteralPath (Join-Path $manifests[0].DirectoryName $entries[0].file)).FullName
if ((Get-FileHash -LiteralPath $wheel -Algorithm SHA256).Hash -ne $entries[0].sha256) {
    throw 'The shipped Whisper wheel differs from its installed manifest'
}

$kit = 'sde-external-10.13.1-2026-07-28-win'
$archive = Get-PinnedFile "https://downloadmirror.intel.com/924984/$kit.tar.xz" "$kit.tar.xz" '74e626ede09b0baa5011fc9e51b58627ea92c3fc0bae5fd7db34b490f335f651'
& tar -xf $archive -C $work
if ($LASTEXITCODE -ne 0) { throw 'Could not extract the verified Intel SDE kit' }
$sde = Join-Path $work "$kit/sde.exe"
$audio = Get-PinnedFile 'https://raw.githubusercontent.com/ggml-org/whisper.cpp/2eeeba56e9edd762b4b38467bab96c2517163158/samples/jfk.wav' 'jfk.wav' '59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e'
$registry = Join-Path $PSScriptRoot '../../models/registry/whisper-base.toml'
$specJson = & python -I -c 'import json, pathlib, sys, tomllib; print(json.dumps(tomllib.loads(pathlib.Path(sys.argv[1]).read_bytes().decode())))' $registry
if ($LASTEXITCODE -ne 0) { throw 'Could not read the Whisper model registry with Python 3.11+' }
$spec = $specJson | ConvertFrom-Json
$weights = $spec.files[0]
$modelUrl = "$($spec.source.provider)/$($spec.source.repo)/resolve/$($spec.source.revision)/$($weights.path)"
$model = Get-PinnedFile $modelUrl $weights.path $weights.sha256

$report = Join-Path $work 'whisper-windows-compat.json'
& python -I (Join-Path $PSScriptRoot 'whisper_windows_compat.py') --python $worker --wheel $wheel --model $model --audio $audio --sde $sde --report $report
if ($LASTEXITCODE -ne 0) { throw "Windows Whisper CPU regression failed; see $report" }
