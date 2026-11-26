# Victory AMD Megafork agent guide

Use this guide to set up the runtime with the user's own Xbox Series X|S title files, diagnose a failed launch, and measure changes to the AMD build. See [README.md](README.md) for hardware requirements and options.

## Set up and launch

1. Work from the clone root. Check the Windows build and GPU driver, the pinned Rust toolchain, and the MSVC C++ build tools with a Windows SDK.
2. Use the exact path to the executable from the user's local title files. Ask for that path if it is missing. Keep the executable and its companion files together outside this clone. Keep the title files read-only for this setup.
3. Copy `victory.example.toml` to `victory.toml` only if the local file is absent. Edit `[paths].game_dir` to point at the external directory containing the title data. Keep any existing settings. `VICTORY_CONFIG` selects a different config, and `VICTORY_GAME_DIR`, `VICTORY_CACHE_DIR`, and `VICTORY_DATA_DIR` take precedence over the corresponding settings in the file.
4. On Windows 10 22H2 x64, run `cargo xtask agility` before building. On Windows 11 this is optional. The task installs the verified Microsoft D3D12 runtime under `target\diag\D3D12`. Keep it beside the executable when using an SDK-enabled build.
5. Build, inspect the resolved configuration, then launch with the baseline profile. Replace the example path with the exact file from step 2.

```powershell
cargo build --locked --profile diag -p victory --features diagnostics
.\target\diag\victory.exe config
$gameExe = 'D:\My Games\My Title\game.exe'
.\scripts\play-amd.ps1 -Exe $gameExe -Profile Stable
```

Before launching, check the path with `Test-Path -LiteralPath $gameExe -PathType Leaf`. On Windows 10, check that `config` reports `app-local SDK 620`. The launcher expects a build in the clone's `target\diag`. See the [Windows 10 setup](README.md#windows-10-setup).

Start with **Stable** (Series S, synchronized texture reads). Compare **Performance** on the same scene after one warm run. **Fsr4K** changes to Series X and requires the official AMD DX12 FidelityFX DLL. Record it as a separate configuration. See [AMD testing](AMD-TESTING.md).

## Diagnose a failed or slow run

- Record the complete command and exit code. Read the first relevant error and the end of the console log. Keep logs local and remove personal paths before sharing excerpts.
- For Shader Model 6.5 or `D3D12_ERROR_INVALID_REDIST`, verify the DLL layout, rebuild after installing the SDK, and check GPU and driver feature support.
- For a black face, missing texture, or blank 3D cutscene, reproduce at the baseline preset. Compare the same scene with cold and warm caches. Note the adapter, missing draws, pipeline compilation failures, and any texture or device errors the log reports.
- For low FPS, compare frame times with one setting changed. Compare CPU submission time and GPU time where available. Report first-run compilation stalls separately from repeatable frame times.
- Keep the cache and save locations separate. Use distinct external `VICTORY_CACHE_DIR` and `VICTORY_DATA_DIR` locations for an isolated experiment. Preserve the player's save files and existing local configuration.

## Validate changes to the megafork

- Run `cargo xtask policy`, `cargo test -p xtask --locked`, and the package tests for the code you touched. Check the [CI workflow](.github/workflows/ci.yml) for full test and lint commands. Report existing failures with the test name and environment.
- For setup changes, check the launcher in Windows PowerShell with an executable path containing spaces and brackets. Verify that a missing file fails and that the launcher's exit code matches the runtime's.
- For renderer changes, compare the same commit pair, scene, driver, profile, resolution, and cache state. Record Windows build, CPU, GPU, RAM, median and p95 frame times, peak VRAM, crashes, and visual defects. Measure the opening cutscene and a fixed gameplay route separately. A faster run with missing draws fails the comparison.
- Check AMD and NVIDIA independently before sharing a GPU default. Cite the upstream change and preserve authorship. Keep game binaries, caches, dumps, textures, shaders, and saves outside Git. See [asset policy](docs/asset-policy.md).
- State which hardware and Windows versions were tested and what still needs an in-game run.

## Where to work

| Area | Path |
| --- | --- |
| Launch profiles | `scripts/play-amd.ps1` |
| CLI and Agility SDK exports | `apps/victory` |
| Local configuration and path validation | `crates/victory-core` |
| Agility SDK installer and repository policy | `xtask` |
| D3D12 submission, textures, and pipelines | `crates/victory-gpu` |
| Shader translation | `crates/victory-shader` |
