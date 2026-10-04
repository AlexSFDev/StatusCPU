$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot

Push-Location "$root\assembler"

cargo run --release -- program.sasm program.status

Pop-Location

Push-Location "$root\emulator"

cargo test
cargo run --release -- "$root\assembler\program.status"

Pop-Location

Write-Host ""
Write-Host "StatusCPU v1 build complete."