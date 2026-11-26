# Victory AMD Megafork

The first build is based on [Belobug `cd8dfc0`](https://github.com/Belobug/VIctory-Belobug-AMD/commit/cd8dfc0). The initial test targets are Windows 11 PCs with Ryzen CPUs and RX 7900 XTX or RX 9070 XT GPUs. Windows 10 22H2 uses the [Agility SDK setup](README.md#windows-10-setup) and still needs an in-game test. The runtime requires your own game files.

Build with `cargo build --locked --profile diag -p victory --features diagnostics` and launch from the repository root:

```powershell
.\scripts\play-amd.ps1 -Exe 'D:\Games\Example\game.exe' -Profile Stable
```

`Stable` starts with Belobug's Safe settings and synchronized texture reads. `Performance` enables Belobug's faster preset and keeps the same texture and indirect-draw safeguards. Both use Series S. `Fsr4K` uses Series X and requires `-FidelityFxDir` pointing to the official signed DX12 upscaler. It requests 1080p input and 4K output. Compare it separately, since it changes the rendering profile and output size. All three are experimental.

For a comparable run, record the commit, GPU, CPU, RAM and driver. Use the same opening cutscene and city route. Record one cold run and a second run after a clean exit with the same cache. Record median frame time and p95 where available, peak VRAM, missing faces or walls, and any crash. Menu FPS is not comparable to 3D gameplay FPS. Keep game files, generated game assets and saves out of reports and releases. Remove local paths from logs before sharing.

## Integration queue

The Belobug base already includes [the texture submission-order fix](https://github.com/Belobug/VIctory-Belobug-AMD/commit/5d6bd84) and [DexNom's FSR integration](https://github.com/Belobug/VIctory-Belobug-AMD/commit/cd8dfc0). The following changes are candidates, not features of this build.

| Priority | Candidate | Acceptance check |
| --- | --- | --- |
| 1 | Adapt fxceless [#1336](https://github.com/fxceless/victory-fork/pull/1336) to hold texture references during GPU copies. The upstream patch does not apply directly to this base. | No texture lifetime regression or GPU reset during the opening and city route. |
| 2 | Compare Cool Fox's targeted character texture upload synchronization with fxceless [#1031](https://github.com/fxceless/victory-fork/pull/1031) and [#1210](https://github.com/fxceless/victory-fork/pull/1210) for sampling resident mip levels. Verify whether the [16-bit facial animation fix](https://github.com/fxceless/victory-fork/pull/927) is already present before porting it. | Faces, hair and clothing render in the opening close-up with the same or better frame time. |
| 3 | Evaluate [AvatarManz #55](https://github.com/avatarmanz/VIctory-7900XTX/pull/55) for shader compilation on RDNA 4. Compare cold and warm starts separately. Consider [fxceless #1226](https://github.com/fxceless/victory-fork/pull/1226) for prebuilding pipelines in addition to pretranslating shaders. | Record pipeline build failures, pending jobs and time until the first correctly drawn 3D scene on RX 9070 XT. |
| 4 | Compare Series S and X at 1080p. Evaluate [Geckomacabre's reduced GPU waits and copy batching](https://github.com/Geckomacabre/vx-layer/commit/b9a97e9d) separately. | Report median and p95 frame time, visual defects and peak memory on the same route. Keep the FSR 4K comparison separate. |

Add the opening boat mission to the route to catch the [water and progression regression](https://github.com/Belobug/VIctory-Belobug-AMD/issues/196). Check faces and shadows together when testing shader changes. One [reported regression](https://github.com/fxceless/victory-fork/issues/1356) improved shadows while removing the protagonist's head.

The megafork includes a port of [fxceless #1353](https://github.com/fxceless/victory-fork/pull/1353). A Rust panic during a pipeline build now completes the failed job and wakes waiters. The existing native compiler guard remains responsible for native driver faults. The port has a synthetic regression test; its effect during gameplay remains to be measured.

## Cool Fox audit

The reviewed package identified itself as codec_d's Cool Fox `30e54b4`. Its release notes report around 30 FPS median over 17 minutes on an RTX 4090 with a Ryzen 7 9800X3D. That result does not establish Radeon performance. The source implements targeted synchronization for character texture uploads, including descriptor publication after the upload. Its launcher also pretranslates shaders once per build. The launcher enables several aggressive texture and draw options, so its configuration should be compared on AMD before adopting any of those defaults.
