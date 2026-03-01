# Build the tslime GUI for Windows
cargo build --release --features gui
Write-Host "Built: target\release\tslime.exe"
Write-Host "Run by double-clicking tslime.exe (no console window needed)"
