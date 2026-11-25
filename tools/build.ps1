param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$workspace = Split-Path -Parent $root
$toolchain = Join-Path $workspace ".toolchain"

# Use the portable workspace toolchain when it exists (this repo's dev setup).
if (Test-Path (Join-Path $toolchain "cargo\bin\cargo.exe")) {
    $env:CARGO_HOME = Join-Path $toolchain "cargo"
    $env:RUSTUP_HOME = Join-Path $toolchain "rustup"
    $env:PATH = (Join-Path $toolchain "cargo\bin") + ";" + (Join-Path $toolchain "mingw64\bin") + ";" + $env:PATH
}

$profileArgs = @()
if ($Release) { $profileArgs += "--release" }

Write-Host "Building Lumo ($(if ($Release) { 'release' } else { 'debug' }))..." -ForegroundColor Cyan
& cargo build --manifest-path (Join-Path $root "Cargo.toml") @profileArgs
if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }

$target = if ($Release) { "release" } else { "debug" }
$targetDir = Join-Path $root "target\$target"
$dist = Join-Path $root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$exeName = if ($Release) { "Lumo.exe" } else { "Lumo-dev.exe" }
Copy-Item (Join-Path $targetDir "lumo.exe") (Join-Path $dist $exeName) -Force
Write-Host "  -> dist\$exeName"

# GNU builds link WebView2Loader dynamically; MSVC builds don't need it.
$loader = Join-Path $targetDir "WebView2Loader.dll"
if (Test-Path $loader) {
    Copy-Item $loader (Join-Path $dist "WebView2Loader.dll") -Force
    Write-Host "  -> dist\WebView2Loader.dll"
}

$size = (Get-Item (Join-Path $dist $exeName)).Length / 1MB
Write-Host ("Done. {0:N1} MB" -f $size) -ForegroundColor Green
if ($Release) {
    Write-Host "Move dist\Lumo.exe and dist\WebView2Loader.dll to a permanent folder (e.g. C:\Apps\Lumo) and run it."
}
