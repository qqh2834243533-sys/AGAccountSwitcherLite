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

$privatePaths = @($pathRemaps.Keys) + @($env:USERPROFILE, $env:TEMP, $env:TMP)
$privatePathPrefixes = @($privatePaths | Where-Object { $_ } | ForEach-Object {
    $_.TrimEnd('\', '/').Replace('\', '/') + '/'
} | Select-Object -Unique)
$profilePathPattern = '(?i)(?:\b[A-Z]:/Users/|(?<![\w/])/(?:Users|home)/)[^/<>"\x00-\x1f]+/'
$secretPattern = '-----BEGIN (?:[A-Z]+ )*PRIVATE KEY-----|\b(?:ghp_|github_pat_|gho_|ghs_|ya29\.|AIza)[A-Za-z0-9_.-]{20,}|\b1//[A-Za-z0-9_-]{24,}'
$emailPattern = '(?i)\b[A-Z0-9._%+-]+@(?!example\.(?:test|com)\b|users\.noreply\.github\.com\b)[A-Z0-9.-]+\.[A-Z]{2,}\b'

function Assert-PublishableContent([string]$Name, [byte[]]$Bytes) {
    $views = @([Text.Encoding]::UTF8.GetString($Bytes))
    if ($Name -like '*.exe' -and $Bytes.Length -gt 1) {
        $views += [Text.Encoding]::Unicode.GetString($Bytes)
        $views += [Text.Encoding]::Unicode.GetString($Bytes, 1, $Bytes.Length - 1)
    }
    foreach ($view in $views) {
        $normalized = $view.Replace('\', '/') -replace '/{2,}', '/'
        foreach ($prefix in $privatePathPrefixes) {
            if ($normalized.IndexOf($prefix, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                throw "Release privacy check failed for '$Name' (local path)."
            }
        }
        if ($normalized -match $profilePathPattern) {
            throw "Release privacy check failed for '$Name' (user profile path)."
        }
        if ($view -cmatch $secretPattern -or $view -match $emailPattern) {
            throw "Release privacy check failed for '$Name' (credential or email pattern)."
        }
    }
}

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

$artifacts = Join-Path $root 'artifacts'
$runId = [guid]::NewGuid().ToString()
$stagingRoot = Join-Path $root '.local\packaging'
$runRoot = Join-Path $stagingRoot $runId
$staging = Join-Path $runRoot 'drop'
$componentScan = Join-Path $runRoot 'components'
$stem = "ag-account-switcher-lite-$version-win-x64"
$zip = Join-Path $runRoot "$stem.zip"
$shipFiles = @($exeName, 'LICENSE', 'NOTICE.md', 'README.md', 'README.en.md')
$sidecars = @("$stem.zip.sha256")
New-Item -ItemType Directory -Force -Path $artifacts, $staging, $componentScan | Out-Null

try {
    Assert-PublishableContent $exeName ([IO.File]::ReadAllBytes($builtExe))
    Assert-PublishableContent 'CHANGELOG.md' ([IO.File]::ReadAllBytes((Join-Path $root 'CHANGELOG.md')))
    foreach ($file in @($builtExe) + @($shipFiles | Where-Object { $_ -ne $exeName } | ForEach-Object { Join-Path $root $_ })) {
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

        $rawValidation = Join-Path $runRoot 'sbom-validation.json'
        & $SbomTool validate -b $staging -o $rawValidation -mi 'SPDX:2.2'
        if ($LASTEXITCODE -ne 0) {
            throw "SBOM validation failed ($LASTEXITCODE)"
        }
        $report = Get-Content -Raw -LiteralPath $rawValidation | ConvertFrom-Json
        $counts = $report.Summary.ValidationTelemetery
        $successful = [int]$counts.FilesSuccessfulCount
        $failed = [int]$counts.FilesFailedCount
        if ($successful -ne $shipFiles.Count -or $failed -ne 0 -or [int]$counts.TotalFilesInManifest -ne $successful) {
            throw 'SBOM validation did not cover every release payload file.'
        }
        [ordered]@{
            status = 'passed'
            filesValidated = $successful
            filesFailed = $failed
        } | ConvertTo-Json | Set-Content -Encoding ascii -LiteralPath (Join-Path $runRoot "$stem.sbom-summary.json")
        Copy-Item -LiteralPath (Join-Path $staging '_manifest\spdx_2.2\manifest.spdx.json') -Destination (Join-Path $runRoot "$stem.spdx.json")
        $shipFiles += '_manifest/spdx_2.2/manifest.spdx.json', '_manifest/spdx_2.2/manifest.spdx.json.sha256'
        $sidecars += "$stem.spdx.json", "$stem.sbom-summary.json"
    }

    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zip -CompressionLevel Optimal -Force

    # Verify the final archive contents before exposing any output files.
    $expectedExeHash = (Get-FileHash -LiteralPath $builtExe -Algorithm SHA256).Hash
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        $entryNames = @($archive.Entries | ForEach-Object { $_.FullName.Replace('\', '/') })
        if (@(Compare-Object -ReferenceObject $shipFiles -DifferenceObject $entryNames -CaseSensitive).Count -ne 0) {
            throw 'Release ZIP contents differ from the expected file list.'
        }
        foreach ($entry in $archive.Entries) {
            $stream = $entry.Open()
            $buffer = New-Object IO.MemoryStream
            try {
                $stream.CopyTo($buffer)
                Assert-PublishableContent ($entry.FullName.Replace('\', '/')) ($buffer.ToArray())
                if ($entry.FullName -ceq $exeName) {
                    $buffer.Position = 0
                    if ((Get-FileHash -InputStream $buffer -Algorithm SHA256).Hash -ne $expectedExeHash) {
                        throw 'The archived executable differs from the Release build.'
                    }
                }
            } finally {
                $buffer.Dispose()
                $stream.Dispose()
            }
        }
    } finally {
        $archive.Dispose()
    }

    $sha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -Encoding ascii -LiteralPath ($zip + '.sha256') -Value "$sha256 *$stem.zip"
    foreach ($name in $sidecars) {
        Assert-PublishableContent $name ([IO.File]::ReadAllBytes((Join-Path $runRoot $name)))
    }
    foreach ($name in @("$stem.zip") + $sidecars) {
        Move-Item -LiteralPath (Join-Path $runRoot $name) -Destination (Join-Path $artifacts $name) -Force
    }
    Write-Output (Join-Path $artifacts "$stem.zip")
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
