# Victory AMD Megafork

Victory AMD Megafork is an experimental Windows compatibility layer for running a user-supplied Xbox Series X|S title. We combine Belobug's AMD renderer with selected fixes from other VIctory projects, then test them on the same scenes before changing the default profile. The current targets are Windows 11 PCs with a Ryzen CPU and a Radeon RX 7900 XTX or RX 9070 XT. Those cards are test targets, and compatibility across Radeon generations is still being measured.

[Build and test guide](AMD-TESTING.md) · [Contributors and credits](CONTRIBUTORS.md) · [Original Belobug documentation](README.belobug.md)

## What has been combined

| Project | What it contributes to this build |
| --- | --- |
| [Marvisak / VIctory](https://github.com/Marvisak/VIctory) | The Xbox runtime, Direct3D 12 rendering layer, and shader translator. |
| [Belobug / VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD) | The AMD base at [`cd8dfc0`](https://github.com/Belobug/VIctory-Belobug-AMD/commit/cd8dfc0), including the texture submission-order fix. |
| [DexNom / VIctory](https://github.com/DexNom/VIctory) | The experimental PC FidelityFX integration carried through Belobug. |
| [fxceless / victory-fork](https://github.com/fxceless/victory-fork) | An adapted fix by [Filip722](https://github.com/Filip722) that marks a failed pipeline job complete after a Rust panic. The [port](https://github.com/rorodev6/victory-amd-megafork-private/commit/af875f2) passed eight targeted tests. |
| [rorodev6](https://github.com/rorodev6) | The megafork, AMD launch profiles, and shared benchmark guide. |

**Also reviewed:** Cool Fox (codec_d), [AvatarManz](https://github.com/avatarmanz/VIctory-7900XTX), [Geckomacabre](https://github.com/Geckomacabre/vx-layer), [HrHd](https://github.com/HrHd/victory-fork), and [shadowdog00](https://github.com/shadowdog00). Proposed ports are listed in the [integration queue](AMD-TESTING.md#integration-queue). The [GitHub contributor history](https://github.com/rorodev6/victory-amd-megafork-private/graphs/contributors) is generated from Git history. This table provides visible attribution on the project homepage.

## Build and run

Use the Rust toolchain pinned in this repository and the MSVC C++ build tools with a Windows SDK. From the project root:

```powershell
cargo build --locked --profile diag -p victory --features diagnostics
.\scripts\play-amd.ps1 -Exe 'D:\Games\Example\game.exe' -Profile Stable
```

| Profile | Use it for |
| --- | --- |
| **Stable** | First launch and troubleshooting. Series S with synchronized texture reads and Belobug's conservative preset. |
| **Performance** | Comparing Belobug's faster settings in Series S. Compare against Stable on the same route. |
| **Fsr4K** | Experimental Series X mode with 1080p input and 4K output. Supply `-FidelityFxDir` with the official AMD DX12 upscaler. |

## Help validate Radeon support

Repeat the opening close-up, a fixed route through the city, and the boat sequence. Record your CPU, Radeon model, driver, RAM, commit and launch profile. Report median and p95 frame times, peak VRAM, missing surfaces and any GPU resets. Compare cold and warm caches separately. The goal is stable **30 FPS in 3D with correct rendering**. Menu FPS and isolated clips are excluded from that target.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE), the [preserved upstream credits](README.belobug.md#credits), and the [asset policy](docs/asset-policy.md). This repository contains the runtime and scripts. Supply your own game files locally.
