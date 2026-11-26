# Victory AMD Megafork

The first build is based on [Belobug `cd8dfc0`](https://github.com/Belobug/VIctory-Belobug-AMD/commit/cd8dfc0). The initial test targets are Windows 11 PCs with Ryzen CPUs and RX 7900 XTX or RX 9070 XT GPUs. They are validation targets, not a published compatibility list. The runtime requires your own game files.

Build with `cargo build --locked --profile diag -p victory --features diagnostics` and launch from the repository root:

```powershell
.\scripts\play-amd.ps1 -Exe 'D:\Games\Example\game.exe' -Profile Stable
```

`Stable` starts with Belobug's Safe settings and synchronized texture reads. `Performance` enables Belobug's faster preset and keeps the same texture and indirect-draw safeguards. Both use Series S. `Fsr4K` uses Series X and requires `-FidelityFxDir` pointing to the official signed DX12 upscaler. It requests 1080p input and 4K output. Compare it separately, since it changes the rendering profile and output size. All three are experimental.

For a comparable run, record the commit, GPU, CPU, RAM and driver. Use the same opening cutscene and city route. Record one cold run and a second run after a clean exit with the same cache. Record median frame time and p95 where available, peak VRAM, missing faces or walls, and any crash. Menu FPS is not comparable to 3D gameplay FPS. Keep game files, generated game assets and saves out of reports and releases. Remove local paths from logs before sharing.

Next integration: isolate the texture and GPU resource fixes from [fxceless](https://github.com/fxceless/victory-fork/commit/e4245f3), then compare each against the recorded baseline. A performance change should preserve the same visible scene and avoid new device losses.
