$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot

$assembler = "$root\assembler"
$emulator = "$root\emulator"
$programs = "$root\test\programs"

Get-ChildItem "$programs\*.sasm" | ForEach-Object {
    $input = $_.FullName
    $output = Join-Path $_.DirectoryName ($_.BaseName + ".status")

    Push-Location $assembler
    cargo run --release -- $input $output
    Pop-Location

    Push-Location $emulator
    cargo run --release -- $output
    Pop-Location
}