[CmdletBinding()]
param(
    [string]$CargoWrapper = 'cargo',
    [string]$SbomTool,
    [switch]$NoBuild,
    [switch]$Offline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$version = (Get-Content -Raw -LiteralPath (Join-Path $root 'VERSION')).Trim()
if ($version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') {
    throw "VERSION must contain a single semantic version, got '$version'."
}

# Verify version parity across VERSION, Cargo.toml, and tauri.conf.json
$cargoToml = Get-Content -Raw -LiteralPath (Join-Path $root 'Cargo.toml')
if ($cargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw 'Unable to read package version from Cargo.toml.'
}
$cargoVersion = $Matches[1]
$tauriConfig = Get-Content -Raw -LiteralPath (Join-Path $root 'tauri.conf.json') | ConvertFrom-Json
$tauriVersion = [string]$tauriConfig.version

if ($cargoVersion -ne $version -or $tauriVersion -ne $version) {
    throw "Version mismatch detected: VERSION=$version, Cargo.toml=$cargoVersion, tauri.conf.json=$tauriVersion"
}

$userCargoHome = if ($env:USERPROFILE) { Join-Path $env:USERPROFILE '.cargo' }
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { $userCargoHome }
$pathRemaps = [ordered]@{}
$pathRemaps[$root] = '/agasl/source'
if ($cargoHome) { $pathRemaps[$cargoHome] = '/agasl/cargo' }
if ($env:RUSTUP_HOME) { $pathRemaps[$env:RUSTUP_HOME] = '/agasl/rustup' }

if (-not $NoBuild) {
    $buildArgs = @('build', '--locked', '--release', '--manifest-path', (Join-Path $root 'Cargo.toml'))
    if ($Offline) {
        $buildArgs += '--offline'
    }
    $previousFlags = $env:CARGO_ENCODED_RUSTFLAGS
    $separator = [string][char]31
    $rustFlags = @()
    if ($previousFlags) {
        $rustFlags += $previousFlags -split $separator
    } elseif ($env:RUSTFLAGS) {
        $rustFlags += $env:RUSTFLAGS -split '\s+' | Where-Object { $_ }
    }
    foreach ($entry in $pathRemaps.GetEnumerator()) {
        $prefixes = @($entry.Key, $entry.Key.Replace('\', '/'))
        if ($entry.Key -match '^[a-z]:\\') { $prefixes += "\\?\$($entry.Key)" }
        foreach ($prefix in $prefixes | Select-Object -Unique) {
            $rustFlags += "--remap-path-prefix=$prefix=$($entry.Value)"
        }
    }
    $rustFlags += '-C', 'link-arg=/PDBALTPATH:%_PDB%'
    try {
        $env:CARGO_ENCODED_RUSTFLAGS = $rustFlags -join $separator
        & $CargoWrapper @buildArgs
        if ($LASTEXITCODE -ne 0) {
            throw "Release build failed ($LASTEXITCODE)"
        }
    } finally {
        $env:CARGO_ENCODED_RUSTFLAGS = $previousFlags
    }
}

$exeName = 'ag-account-switcher-lite.exe'
$builtExe = Join-Path $root "target\release\$exeName"
if (-not (Test-Path -LiteralPath $builtExe)) {
    throw "Expected compiled executable not found at $builtExe"
}

# Rust source remapping is best effort. Also check the actual distributable PE.
$binary = [IO.File]::ReadAllBytes($builtExe)
$binaryStrings = @(
    [Text.Encoding]::UTF8.GetString($binary)
    [Text.Encoding]::Unicode.GetString($binary)
    [Text.Encoding]::Unicode.GetString($binary, 1, $binary.Length - 1)
)
foreach ($directory in @($pathRemaps.Keys) + @($env:USERPROFILE)) {
    if (!$directory) { continue }
    $native = $directory.TrimEnd('\', '/') + '\'
    $portable = $native.Replace('\', '/')
    if ($binaryStrings | Where-Object {
        $_.IndexOf($native, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or
        $_.IndexOf($portable, [StringComparison]::OrdinalIgnoreCase) -ge 0
    }) {
        throw 'The release executable still contains a local build path. Rebuild with this packaging script before publishing.'
    }
}

$artifacts = Join-Path $root 'artifacts'
$runId = [guid]::NewGuid().ToString()
$stagingRoot = Join-Path $root '.local\packaging'
$runRoot = Join-Path $stagingRoot $runId
$staging = Join-Path $runRoot 'drop'
$componentScan = Join-Path $runRoot 'components'
$stem = "ag-account-switcher-lite-$version-win-x64"
$zip = Join-Path $artifacts "$stem.zip"
New-Item -ItemType Directory -Force -Path $artifacts, $staging, $componentScan | Out-Null

try {
    foreach ($file in @($builtExe, (Join-Path $root 'LICENSE'), (Join-Path $root 'NOTICE.md'), (Join-Path $root 'README.md'), (Join-Path $root 'README.en.md'))) {
        Copy-Item -LiteralPath $file -Destination $staging -Force
    }

    if ($SbomTool) {
        Copy-Item -LiteralPath (Join-Path $root 'Cargo.toml'), (Join-Path $root 'Cargo.lock') -Destination $componentScan -Force
        $scanSrc = Join-Path $componentScan 'src'
        New-Item -ItemType Directory -Force -Path $scanSrc | Out-Null
        Set-Content -Encoding ascii -LiteralPath (Join-Path $scanSrc 'main.rs') -Value 'fn main() {}'
        & $SbomTool generate -b $staging -bc $componentScan -pn 'AG Account Switcher Lite' -pv $version -ps 'AG Account Switcher Lite contributors' -mi 'SPDX:2.2'
        if ($LASTEXITCODE -ne 0) {
            throw "SBOM generation failed ($LASTEXITCODE)"
        }

        & $SbomTool validate -b $staging -o (Join-Path $artifacts "$stem.sbom-validation.json") -mi 'SPDX:2.2'
        if ($LASTEXITCODE -ne 0) {
            throw "SBOM validation failed ($LASTEXITCODE)"
        }
        Copy-Item -LiteralPath (Join-Path $staging '_manifest\spdx_2.2\manifest.spdx.json') -Destination (Join-Path $artifacts "$stem.spdx.json") -Force
    }

    if (Test-Path -LiteralPath $zip) {
        Remove-Item -LiteralPath $zip -Force
    }
    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zip -CompressionLevel Optimal -Force

    # Round-trip verify the packaged executable matches the built Release binary byte-for-byte
    # Compatible with both Windows PowerShell 5.1 (.NET Framework) and PowerShell 7+ (.NET)
    $expectedExeHash = (Get-FileHash -LiteralPath $builtExe -Algorithm SHA256).Hash.ToUpperInvariant()
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        $entry = $archive.GetEntry($exeName)
        if (-not $entry) {
            throw "Packaged archive is missing $exeName"
        }
        $stream = $entry.Open()
        $hasher = [System.Security.Cryptography.SHA256]::Create()
        try {
            $zippedExeHash = ([BitConverter]::ToString($hasher.ComputeHash($stream))).Replace('-', '').ToUpperInvariant()
            if ($zippedExeHash -ne $expectedExeHash) {
                throw "Archive verification failed: expected $expectedExeHash, got $zippedExeHash"
            }
        } finally {
            $hasher.Dispose()
            $stream.Dispose()
        }
    } finally {
        $archive.Dispose()
    }

    $sha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -Encoding ascii -LiteralPath ($zip + '.sha256') -Value "$sha256 *$stem.zip"
    Write-Output $zip
} finally {
    if (Test-Path -LiteralPath $runRoot) {
        $resolvedRun = (Resolve-Path -LiteralPath $runRoot).ProviderPath
        $resolvedRoot = (Resolve-Path -LiteralPath $stagingRoot).ProviderPath.TrimEnd('\') + '\'
        if ($resolvedRun.StartsWith($resolvedRoot, [StringComparison]::OrdinalIgnoreCase) -and
            (Split-Path -Leaf $resolvedRun) -eq $runId) {
            Remove-Item -LiteralPath $resolvedRun -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}
