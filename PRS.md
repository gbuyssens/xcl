# PR state (2026-11-24) – upstream is `xcl-dev/xcl`

Rules: never a title, build or publisher name in a PR or comment ("a retail game"). Upstream's
CONTRIBUTING: one logical change, under ~400 changed lines, a lead paragraph with "Fixes #N",
then **What is in it**, **Tested** (GPU + driver, CPU, Rust version, `cargo test --workspace --
--test-threads=4` counts), **Regression run** (two `xcl-smoke` blocks from `scripts/smoke.ps1 -Game
<folder>`, main and the PR head, for any runtime change), **Not tested**, **AI use** (the tool and
the exact model, named by you – a PR without it, or made with a small model, is closed). A person
opens the PR; a test result not produced by a real run on the named hardware closes it.

## Done upstream (nothing to send)
- #151 console CPU instructions – merged. #190 trace tooling – merged. #129 frame clock, #127 CPU
  check – merged. #192 (XG #10) closed: upstream has it. #142, #130, #126 closed as obsolete.

## #194 "A memory line at every crash…" – close it
Upstream's `XCL_ACTIVITY` logs commit and video memory now and the fork dropped `memstat` in the
merge. Comment and close:

> Superseded by `XCL_ACTIVITY` (commit and local / non-local video memory every 2 s). Closing.

## #128 "Recompiler: f16 add / sub / fma rounded once under a directed mode" – updated, needs your run
Branch `pr/f16-directed-rounding` = 2bc03fa, rebased on upstream main 9e3831e, pushed (the PR
shows it). The fix ivall asked for is in: an infinite or NaN source of `v_fma_mix*_f16` gives
inf / NaN under every mode (lane 18 of `gpu_s_round_mode_controls_f16_rounding_across_modes`:
+inf under RTZ gave 0x7BFF). Four helper cases added (`fma_mix_f16_rm` with ±inf, clamped inf,
NaN).

Before commenting, on the PR head: `cargo test --workspace -- --test-threads=4` on your GPU
(the tests that matter: `gpu_s_round_mode_controls_f16_rounding_across_modes`,
`gpu_f16_add_and_fma_round_once_under_directed_modes`, `gpu_f16_rm_helpers_lane_by_lane` – the
helper probes). Then post, with the real numbers:

> Rebased on main (9e3831e) and the fma path fixed for infinite sources: `fma_mix_f16_rm` took an
> infinite result for a finite overflow and gave RTZ's 65504; inf / NaN sources now give inf / NaN
> under every mode (lane 18, and ±inf / clamp / NaN probes added to the helper test).
>
> **Tested:** <GPU + driver>, <CPU>, rustc <version>, release.
> `cargo test --workspace -- --test-threads=4`: <N> passed, <N> failed, <N> ignored.
> `gpu_s_round_mode_controls_f16_rounding_across_modes` and
> `gpu_f16_add_and_fma_round_once_under_directed_modes` pass on this GPU.
>
> **Not tested:** RDNA4 (the `v_sub_f16` mode 0x4 failure ivall saw on an RX 9070 – I have no
> such card; the lane's expression is unchanged by this update).
>
> **AI use:** <tool and model>, wrote the helpers and tests from the review's cases; every line
> read.

If `v_sub_f16` under mode 0x4 fails on your card too, say so in the comment instead of the
"Not tested" line and do not claim the test passes.


## NEW – `pr/dxc-strip`: "Recompiler: DXC gets the code without indentation and comment lines; the -O level counts code"
Branch on upstream 9e3831e, one commit, 2 files (~60 lines + test). Open against `xcl-dev/xcl` main
and link it in #1269. Before opening: `cargo test --workspace -- --test-threads=4` on your machine
(the new test `dxc_gets_the_code_without_indentation_and_comment_lines` is CPU-only), then a smoke
run on main and on the PR head (`scripts/smoke.ps1 -Game <folder>`), both blocks pasted. Fill every
<…>; name the AI tool and model yourself under AI use.

> Part of #1269.
>
> A translated program's HLSL is 30.7 % leading indentation and 25.0 % `// pc: instruction` comment
> lines (throny's measurement over the pack's 2 689 compute programs), and `compile_here` chose -O0
> above 8 MB and -O1 above 1 MB of that text. A 9.9 MB program with 4.4 MB of code was compiled at
> -O0: DXIL 0.95x the source instead of 0.28x, and the driver did not finish it.
>
> **What is in it**
> - `dxc::for_dxc`: DXC gets the text without the indentation and with the comment lines emptied
>   (kept, not removed: its error line numbers still match the dumped HLSL); the -O level is chosen
>   on that text, so it counts code.
> - The DXIL cache stays keyed on the full text: nothing cached is built again.
> - `XCL_DXC_STRIP=0` turns it off, for A/B measurements with the harness in #1269.
> - It does not rescue the two 30 MB programs (13 MB of code): those need #1316.
>
> **Tested:** <GPU + driver>, <CPU>, rustc <version>, release.
> `cargo test --workspace -- --test-threads=4`: <N> passed, <N> failed, <N> ignored (SKIPPED GPU tests: <…>).
>
> **Regression run**
> main: <xcl-smoke block>
> PR: <xcl-smoke block>
>
> **Not tested:** DXC time and DXIL size per program before / after on the full pack – measurable
> with throny's harness (`xcl shaders --precompile --only cs`, `XCL_DXC_STRIP=0` for the baseline);
> AMD.
>
> **AI use:** <tool and model>, wrote the function and the test from the thread's numbers; every line read.

## Nothing else to open
`pr/xg-resource-tiling`, `pr/memory-report`: superseded. Any new PR must follow the format above
and carry a smoke run; the fork's main is a merge of upstream 9e3831e plus the fork's diagnostics.
