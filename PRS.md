# PRs to open on ivall/xcl (from the fork's branches)

State on 2026-11-23 (upstream main = 956a476 "Retail game reaches 30 fps"):
- Merged: #127, #129. Closed as duplicates: #126, #130. Open: #128 (pr/f16-directed-rounding, fixed per Foniass'
  review and rebased – run `scripts\test.ps1` on it and post the comment).
- Obsolete, do NOT open: pr/newer-title-startup (in upstream 9add721), pr/omod-clamp, pr/vram-arenas-queue18,
  pr/frame-clock, pr/eh-pe-cpu-check (merged / closed); the chad.dll XDsp commit (ivall did the DSP himself in 66e8e97).
- All four branches below are rebased on 956a476 and cross-checked (`cargo check --tests`, Windows target).
- Never name a title, a build or a publisher in a PR or a comment.

Compare links (base: ivall/xcl main):
- https://github.com/ivall/xcl/compare/main...gbuyssens:xcl:pr/trace-tooling (to open)
- https://github.com/ivall/xcl/compare/main...gbuyssens:xcl:pr/console-insn (to open)
- https://github.com/ivall/xcl/compare/main...gbuyssens:xcl:pr/xg-resource-tiling (to open; say it supersedes the xg part of #169)
- https://github.com/ivall/xcl/compare/main...gbuyssens:xcl:pr/memory-report (to open after one run shows the line)
- https://github.com/ivall/xcl/compare/main...gbuyssens:xcl:pr/f16-directed-rounding (PR #128, open)

---
## 1. pr/trace-tooling (3 commits)
Title: Debug tracing: XCL_D3D12_TRACE shows the arguments, XCL_TRACE_IMPORTS logs every HLE import call with its result

Tooling only, off by default; no change to a title's run without the variables.

- `XCL_D3D12_TRACE=1` lines carry the arguments: the three register arguments after `this`, two stack arguments,
  and what a pointer argument points at (the first words – a descriptor, the hidden return slot of a
  struct-returning method; a string when it reads like one). The thunk passes its stack frame to the logger
  (`mov r8, rsp`); still a tail jump, so the slot splits keep classifying by the caller's return address.
- `XCL_TRACE_IMPORTS=1` (`guard::make_call_tracer`, `resolver::trace_filter`): every import of the title that
  resolves to an HLE function is wrapped in a call-through thunk → `trace: dll!Function ← module+offset: (args)
  → result`. The thunk keeps the register arguments, forwards eight stack arguments, calls the original and logs
  when it returns; it registers unwind info (`RtlAddFunctionTable`) so an exception raised inside unwinds through
  it. `all` wraps system imports too (short runs – a data export would be wrapped as well);
  `kernelx!Create,-kernelx!GetTickCount` are include / exclude prefixes of `dll!Function`. Stubs are left alone
  (they log themselves).
- Refused interfaces are logged: `QueryInterface` on an XGameRuntime object and on the DirectStorage factory /
  queue / file names the IID (how the newer DirectStorage factory IID was found).

Why: during a bring-up the question is usually "which call returned 0 just before the assert" – the D3D12 trace
showed the device calls but nothing of kernelx / XGameRuntime / GameInput.

Test: `call_tracer_forwards_arguments_and_results` (integer, stack and floating-point arguments; integer and
floating-point results) next to the existing guard tests. `cargo test -p xcl guard` on Windows.

Tested: <GPU> – `scripts\test.ps1`; a title with both variables (log lines as above), and without them (unchanged).

---
## 2. pr/console-insn (1 commit)
Title: Console CPU instructions a PC CPU lacks are emulated (clwb, clzero, rdpru, monitorx / mwaitx, rdpid)

A title compiled for the console's Zen 2 raises STATUS_ILLEGAL_INSTRUCTION (0xC000001D) on an Intel CPU before
Ice Lake / Alder Lake (`clwb`) or on any Intel CPU (`clzero`, `rdpru`, `monitorx` / `mwaitx`, AMD-only). Seen on
an i5/i7 of the Coffee Lake generation right after the first descriptor written into title memory.

- `insn::decode`: the console-only instruction at the faulting address and its length (prefixes, ModRM / SIB /
  displacement for `clwb`'s memory operand; `rdpid`'s register); `ud2` is recognised as the title's own trap.
- `eh::emulate_console_only` (the vectored handler, first chance): `clwb` / `monitorx` / `mwaitx` do nothing (the
  PC's memory is coherent), `clzero` zeroes the 64-byte line at rAX, `rdpru` answers QueryPerformanceCounter in
  edx:eax, `rdpid` the processor number; RIP is stepped over the instruction – EXCEPTION_CONTINUE_EXECUTION. Each
  kind is logged once. Any other illegal instruction is logged with its 16 code bytes, first chance and in the
  crash line, so the next missing one can be named.
- Not emulated: SHA-NI (also missing on those CPUs) – to add when a title shows `0F 38 C8..CD`.

Test: `insn::tests::decodes_the_console_only_instructions` (every form, the lookalikes `clflushopt` / `xsaveopt`,
truncated bytes).

Tested: an i5/i7 Coffee Lake + RTX 3060 – a title that died on `clwb` runs past it (both `clwb` and `clzero` were
met and emulated, it reached its first frame and ran on for minutes); `scripts\test.ps1`.

---
## 3. pr/xg-resource-tiling (1 commit)
Title: XGTextureAddressComputer #10 is GetResourceTiling (read from the console library's code)

Supersedes the xg.rs part of #169 (its DSV format remap is covered by #163 / #170).

The console library's vtable, disassembled: slot 10 takes `ID3D12Device::GetResourceTiling`'s six arguments
for the computer's resource – `UINT* pNumTilesForEntireResource, D3D12_PACKED_MIP_INFO*, D3D12_TILE_SHAPE*,
UINT* pNumSubresourceTilings (in: room, out: written), UINT FirstSubresourceTilingToGet,
D3D12_SUBRESOURCE_TILING*`. Its behaviour: E_FAIL unless the swizzle mode is a 64 KiB one (8–11, 16–19, 24–27:
"Resource does not have a 64KB swizzle mode compatible with tiled resources"), E_NOTIMPL for 1D and 3D; the standard
tile in elements goes by their size (256×256 for 1 byte, 256×128 for 2, 128×128 for 4, 128×64 for 8, 64×64 for
16; halved per MSAA doubling), reported in texels; a level's tiling is its padded size over the tile, its start tile
the level's offset in 64 KiB tiles plus the slice's; the packed levels take one tile at the first one's offset;
the tilings are indexed by D3D12 subresource (level + slice × levels) skipping the packed levels.

- `xg::get_resource_tiling` does this on the existing `Texture` model (padded sizes, mip offsets, first tail level).
- #169's guess (GetSubresourceLayout / GetSubresourceOffsetBytes, deciding by which arguments look like pointers)
  answered E_INVALIDARG by luck with the real arguments (the "plane" was the low half of a pointer) and would
  otherwise have written 0x70 bytes over a 12-byte tile shape on the caller's stack.
- Slot 11 is `GetDescriptor(UINT plane, void* descriptor)` – "pDescriptor cannot be NULL", "PlaneIndex must be
  less than 3", copies the plane's 32-byte hardware texture descriptor – not modelled, stays a logged stub; slot 12
  returns a flag byte of the resource.

A retail title's texture streaming calls #10 for every texture it places in a tiled pool (179 calls in one run,
all E_NOTIMPL before).

Test: `resource_tiling_of_a_64kb_texture` (RGBA8 256×256, full chain, 64KB_S: 6 tiles, two standard levels, seven
packed in one tile, the second level alone, a 4 KiB mode refused).

Tested: <GPU> – `scripts\test.ps1`; <a title> (the calls answer, log line of #10 gone from the stub list).

---
## 4. pr/memory-report (1 commit)
Title: A memory line at every crash and while the title runs: process and system commit, video memory, what the title asked for

A title that runs out of memory dies in an allocation of its own, far from the cause: a write to address 0 in its
thread start-up, the driver's E_OUTOFMEMORY (0x8007000e) on a pipeline. Seen with 32 GB of RAM and a 48 GB page
file 98 % full (the title's own crash reporter: `memory.load_percent 84`, `swap.free 801176` of `48801508` KB).

- `memstat.rs` logs one line – `memory: process commit … (working set …); system commit … of …, … of RAM free (… %
  load); video … of … local, … of … shared; the title asked for graphics …, Windows …, heaps …, physical pages …;
  sparse windows … tiles (…) in … pools` – at every unhandled exception, when the process ends with an error, and
  from a thread whenever the process's commit grew by a GiB (every 30 s at most).
- "The title asked for" are gross counters (`memstat::note` on kernelx's commit paths: sparse / UMA graphics memory,
  Windows commits, XMemAlloc heaps, the homes of physical pages). Video memory is `IDXGIAdapter3::QueryVideoMemoryInfo`,
  local and non-local (the system memory the driver spills into once the tile pools outgrow the card: on a 12 GB
  card every GiB of title memory beyond it costs a second GiB of commit).
- The crash line carries the code bytes at RIP for every exception, not only illegal instructions.

Tested: <GPU, RAM> – `scripts\test.ps1`; <a title> (the lines, and what the last one said before the crash).
