[CmdletBinding()]
param(
    [string]$DotnetPath = 'dotnet',
    [string]$SbomTool
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$version = (Get-Content -Raw -LiteralPath (Join-Path $root 'VERSION')).Trim()
if ($version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') {
    throw 'VERSION must contain a single semantic version.'
}

$artifacts = Join-Path $root 'artifacts'
$staging = Join-Path $root ('.local\packaging\' + [guid]::NewGuid())
$stem = "ag-account-switcher-lite-$version-win-x64"
$zip = Join-Path $artifacts "$stem.zip"
New-Item -ItemType Directory -Force -Path $artifacts, $staging | Out-Null

& $DotnetPath publish (Join-Path $root 'src\AGSwitcher.App\AGSwitcher.App.csproj') -c Release --no-build --no-restore -o $staging -m:1
if ($LASTEXITCODE -ne 0) { throw "dotnet publish failed ($LASTEXITCODE)" }
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $staging

if ($SbomTool) {
    & $SbomTool generate -b $staging -bc (Join-Path $root 'src') -pn 'AG Account Switcher Lite' -pv $version -ps 'AG Account Switcher Lite contributors' -mi 'SPDX:2.2'
    if ($LASTEXITCODE -ne 0) { throw "SBOM generation failed ($LASTEXITCODE)" }

    & $SbomTool validate -b $staging -o (Join-Path $artifacts "$stem.sbom-validation.json") -mi 'SPDX:2.2'
    if ($LASTEXITCODE -ne 0) { throw "SBOM validation failed ($LASTEXITCODE)" }
    Copy-Item -LiteralPath (Join-Path $staging '_manifest\spdx_2.2\manifest.spdx.json') -Destination (Join-Path $artifacts "$stem.spdx.json") -Force
}

Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zip -Force
$sha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -Encoding ascii -LiteralPath ($zip + '.sha256') -Value "$sha256 *$stem.zip"
Write-Output $zip
