# Run full deploy-flow integration tests (temp sandboxes, not in repo).
# Usage:
#   .\scripts\run-deploy-flow-tests.ps1           # all profiles
#   .\scripts\run-deploy-flow-tests.ps1 bethesda  # one engine family
#   .\scripts\run-deploy-flow-tests.ps1 skyrimse  # single profile

param(
    [string]$Filter = "deploy_flow"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location (Join-Path $root "src-tauri")
try {
    if ($Filter -eq "deploy_flow") {
        Write-Host "Running all deploy-flow tests..."
        cargo test deploy_flow -- --nocapture
    } elseif ($Filter -match "^deploy_flow_") {
        cargo test $Filter -- --nocapture
    } elseif ($Filter -match "^(bethesda|data|kcd|cyberpunk|bg3|mods|mod_root|stardew|bepinex|subnautica|marvel|unreal|mod_path|game_root)$") {
        Write-Host "Running engine family: deploy_flow_engine_$Filter"
        cargo test "deploy_flow_engine_$Filter" -- --nocapture
    } else {
        Write-Host "Running tests matching profile: $Filter"
        cargo test deploy_flow -- $Filter --nocapture
    }
} finally {
    Pop-Location
}
