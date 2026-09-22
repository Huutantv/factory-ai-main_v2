# F.Auto installer for Windows (PowerShell 5+).
#
#   irm https://raw.githubusercontent.com/Huutantv/factory-ai/main/install.ps1 | iex
#
# Downloads the latest optimized `fauto.exe` from GitHub Releases, drops it in
# %LOCALAPPDATA%\F.Auto (override with $env:AIZEN_INSTALL), and adds that folder
# to your user PATH. No admin rights, no toolchain, no Node/Python.

$ErrorActionPreference = 'Stop'

$Repo   = 'Huutantv/factory-ai'
$Suffix = 'windows-x86_64.exe'
$Dir    = if ($env:FAUTO_INSTALL) { $env:FAUTO_INSTALL } else { Join-Path $env:LOCALAPPDATA 'F.Auto' }

Write-Host "Installing F.Auto..." -ForegroundColor Cyan

$rel = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ 'User-Agent' = 'fauto-installer' }
$asset = $rel.assets | Where-Object { $_.name -like "*$Suffix" } | Select-Object -First 1
if (-not $asset) { throw "No Windows asset (*$Suffix) found in release $($rel.tag_name)." }

New-Item -ItemType Directory -Force $Dir | Out-Null
$dest = Join-Path $Dir 'fauto.exe'
Write-Host ("  {0}  {1}  ({2:N1} MB)" -f $rel.tag_name, $asset.name, ($asset.size / 1MB))

$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest $asset.browser_download_url -OutFile $dest

# Add the install dir to the USER PATH (idempotent) and to the current session.
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $Dir) {
    [Environment]::SetEnvironmentVariable('Path', ($userPath.TrimEnd(';') + ';' + $Dir), 'User')
    Write-Host "  added $Dir to your PATH"
}
if (($env:Path -split ';') -notcontains $Dir) { $env:Path = "$env:Path;$Dir" }

Write-Host ""
Write-Host "F.Auto installed -> $dest" -ForegroundColor Green
Write-Host "Open a NEW terminal, then run:  fauto config" -ForegroundColor Yellow
