# Contributors and credits

The repository keeps the Belobug Git history, including original commit authors, co-authors and upstream notices. See [GitHub Contributors](https://github.com/rorodev6/victory-megafork-amd/graphs/contributors) for the accounts GitHub identifies from that history. The credits below also cover work reviewed or ported indirectly.

## In the current base

- [Marvisak and VIctory contributors](https://github.com/Marvisak/VIctory) created and developed the upstream runtime.
- [Belobug and contributors](https://github.com/Belobug/VIctory-Belobug-AMD) maintain the AMD base, including the renderer, texture and launch work used here. The upstream acknowledgements remain in the README and license notices.
- [DexNom and contributors](https://github.com/DexNom/VIctory/commit/ab34a8b21477c94da2a6cf07249138e8bc84602e) supplied the PC FidelityFX integration that Belobug ported.
- [Filip722](https://github.com/Filip722) wrote the [pipeline job completion fix](https://github.com/fxceless/victory-fork/pull/1353) adapted from fxceless into the megafork.
- [domints](https://github.com/domints) contributed the [D3D12 Agility SDK setup](https://github.com/fxceless/victory-fork/pull/402), adapted from the [public Victory fork](https://github.com/rorodev6/victory/commit/bdd68c2).
- [rorodev6](https://github.com/rorodev6) owns and maintains the AMD Megafork project.

## Compared during research

[fxceless and contributors](https://github.com/fxceless/victory-fork), including tippythetaidum for [the 16-bit facial animation fix](https://github.com/fxceless/victory-fork/pull/927), codec_d and the Cool Fox contributors, [AvatarManz and contributors](https://github.com/avatarmanz/VIctory-7900XTX), [Geckomacabre](https://github.com/Geckomacabre/vx-layer), and [HrHd](https://github.com/HrHd/victory-fork) supplied code, reports or ideas reviewed in the audit. The [integration queue](AMD-TESTING.md#integration-queue) distinguishes proposed imports from what is already included in the base.

[shadowdog00 and the vi-pc-combo contributors](https://github.com/shadowdog00/vi-pc-combo) are credited in the [rorodev6/victory build](https://github.com/rorodev6/victory/commit/4ddc0c1) used for earlier comparisons.

When adapting a patch, credit the original author and link the source commit or pull request in the new commit message. Keep the upstream license and notices with any redistributed code.
