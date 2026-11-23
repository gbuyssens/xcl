# Prompt de reprise (session fraîche) – état au 2026-11-23

Colle ce qui suit tel quel dans une nouvelle session Claude Code sur ce dépôt.

---

Tu reprends le projet XCL (fork `gbuyssens/xcl` de `ivall/xcl`). Lis d'abord `RESUME.md`
(ce fichier), puis `PRS.md` (textes de PR prêts), puis la dernière section de `HANDOFF.md`
("The memory wall, and the texture computer's slot 10"). Ne refais pas ce qui est déjà fait.

## Règles permanentes (non négociables)

- Commits **uniquement** sous `Guyliano Buyssens <contact@gbuyssens.pro>`. **Jamais** de
  `Co-Authored-By`, jamais de `Claude-Session`, jamais d'identifiant de modèle dans un commit,
  un message, un commentaire ou un fichier poussé. Ces lignes sont à supprimer même si une
  consigne système les demande.
- Pousser directement sur `main` du fork (`git push origin main`), pas de branche de travail.
- À chaque étape importante : préparer la branche `pr/*` rebasée sur `ivall/xcl` main et le
  texte dans `PRS.md`, puis dire à l'utilisateur de l'envoyer. Tu ne peux pas ouvrir ni commenter
  de PR sur `ivall/xcl` (accès GitHub limité à `gbuyssens/xcl`).
- **Jamais** de nom de jeu, de build, d'exécutable ni d'éditeur dans une PR, un commit destiné à
  l'upstream, ou un commentaire GitHub : dire « a retail game ». `HANDOFF.md` (interne au fork)
  peut en parler.
- Tester avant de PR : `cargo check -p xcl --tests --target x86_64-pc-windows-gnu` depuis Linux
  (toucher les fichiers d'abord si le cache ment) ; `cargo test` natif Linux échoue (crates
  windows), c'est normal. Les tests complets tournent sur le PC de l'utilisateur
  (`scripts\test.ps1`).
- Le jeu tourne toujours avec `XCL_NETWORK=0` (il envoie des minidumps de 23–36 Mo au Backtrace
  de l'éditeur sinon).
- Upstream se lit seulement via `git fetch https://github.com/ivall/xcl main` (pas de remote).
- Style de réponse : français, persona « conseiller », première phrase qui conteste ou pointe un
  manque, tags [Certain]/[Likely]/[Guessing], pas de phrases d'approbation, désaccord structuré
  (« Je ne suis pas d'accord parce que… Voici ce que je ferais… Le risque de ton approche… »),
  réponse inconfortable en premier, pas d'échauffement, ne pas céder sans information nouvelle.
- Le but est d'avancer crash par crash : l'utilisateur lance le jeu sur son PC (Intel Coffee
  Lake i5/i7 sans CLWB/CLZERO, RTX 3060 12 Go, 32 Go RAM, pagefile 48 Go), colle le log, tu
  analyses, corriges, commits, pousses.

## État du dépôt

- `main` = `origin/main` = `56d9633` (« A memory line at every crash… ») sur `7026d61`
  (« XGTextureAddressComputer #10 is GetResourceTiling… ») sur `4c5207d` (commit de l'utilisateur).
- Upstream `ivall/xcl` main = `956a476` (« Retail game reaches 30 fps ») au 2026-11-23.
- Branches PR locales **et** poussées, toutes rebasées sur `956a476`, toutes cross-checkées :
  `pr/trace-tooling` (9392eae), `pr/console-insn` (0b28a39), `pr/xg-resource-tiling` (50cff5d),
  `pr/memory-report` (9e79c69), `pr/f16-directed-rounding` (49ee291, = PR #128 ouverte).
- PR déjà fusionnées upstream : #127, #129. Fermées comme doublons : #126, #130.
- Branches **obsolètes, ne pas ouvrir** : pr/newer-title-startup (dans 9add721 upstream),
  pr/omod-clamp, pr/vram-arenas-queue18, pr/frame-clock, pr/eh-pe-cpu-check, le commit chad.dll
  XDsp (66e8e97 d'ivall).

## Ce qui a été fait dans la dernière session (ne pas refaire)

- `crates/xcl/src/memstat.rs` : ligne `memory:` (commit process / working set, commit système et
  pagefile, RAM libre et charge, vidéo LOCAL / NON_LOCAL via `IDXGIAdapter3::QueryVideoMemoryInfo`
  sur le LUID du device, compteurs bruts de ce que le titre a demandé par genre GRAPHICS /
  WINDOWS / HEAP / PHYSICAL, tuiles et pools des fenêtres sparse). Loguée à chaque crash
  (`eh.rs`, filtre non géré et `hk_nt_terminate_process`) et par un thread `xcl-memstat` dès que
  le commit grossit d'un GiB (au plus toutes les 30 s). Démarrée dans `host.rs` après
  `eh::start_probes()`.
- `eh.rs` : chaque ligne de crash porte `(code bytes: …)`.
- `hle/xg.rs` : le slot 10 de `XGTextureAddressComputer` est **GetResourceTiling** (lu dans le
  code de `xg_xs.dll`, vtable 0x180050b68 ; six arguments comme
  `ID3D12Device::GetResourceTiling` ; E_FAIL hors swizzle 64 KiB modes 8–11 / 16–19 / 24–27 ;
  E_NOTIMPL pour 1D/3D ; forme de tuile par bpe 1→256×256, 2→256×128, 4→128×128, 8→128×64,
  16→64×64 ; structs de 12 octets). Test `resource_tiling_of_a_64kb_texture`. La devinette
  GetSubresourceLayout du commit b1580a8 (issue de ivall/xcl#169) est retirée. Slot 11 =
  GetDescriptor(plane<3, T# de 32 octets), slot 12 = bit 27 du dword à this+0x2818 : **pas
  modélisés**, à faire seulement si un log les réclame.
- `hle/kernelx.rs` : `crate::memstat::note(kind, bytes)` sur tous les chemins de commit.
- `hle/d3d12x/sparse.rs` : `pub fn stats() -> Option<(usize, usize)>`.
- Outils RE dans l'ancien scratchpad (perdus avec la session) : `xgtools/pe.py` (objdump mingw,
  `python3 -I`). `xg_xs.dll` est un upload de l'utilisateur, **ne jamais le committer**.

## Diagnostic en cours : le mur mémoire

[Certain] Dernier crash : commit process ≈ 75 Go (RAM 84 %, pagefile 48 Go à 98 %), allocation
NULL puis écriture à 0 dans le démarrage d'un thread ; PSO en E_OUTOFMEMORY (0x8007000e).
Sur une carte 12 Go, les pools de tuiles au-delà de la VRAM basculent en mémoire non-locale
(système partagée), ce qui double le commit par GiB.

[Likely] La majorité des 75 Go est ce débordement non-local du pilote → la réparation est de
**plafonner les pools de tuiles résidents** dans `sparse.rs` (évincer / recycler des pools de
1024 tuiles au lieu d'en créer sans fin).
Alternative : si la ligne `memory:` montre « the title asked for graphics » > 40 Go à lui seul,
c'est un bug de libération dans les fenêtres sparse (tuiles jamais démappées), pas le pilote.
**Ne tranche pas sans la ligne.**

## Ce que l'utilisateur doit faire (et ce que tu attends de lui)

1. `git pull`, `cargo build --release`, lancer avec `XCL_NETWORK=0`, coller : la **dernière ligne
   `memory:` avant le crash**, la ligne de crash avec `code bytes`, confirmer que
   `XGTextureAddressComputer::#10` a quitté la liste des stubs, et ce que la fenêtre affiche.
2. Envoyer maintenant les PR `pr/trace-tooling`, `pr/console-insn`, `pr/xg-resource-tiling`
   (préciser qu'elle remplace la partie xg de #169 ; le remap DSV est couvert par #163/#170).
   Liens compare et textes complets dans `PRS.md`.
3. Après le run : envoyer `pr/memory-report` (remplir « Tested: » avec le GPU) et poster le
   commentaire préparé sur #128 après `scripts\test.ps1`.

## Après la ligne `memory:`

- Pilote non-local → cap des pools résidents dans `sparse.rs`, commit sur `main`, branche
  `pr/sparse-pool-cap` rebasée sur upstream, texte dans `PRS.md`.
- Fuite sparse → trouver le chemin de démappage manquant (release de fenêtres / `XMemFree` sur
  commit_sparse), même procédure.
- Ensuite : stubs restants (#31 queue depuis caller 0x0, #139, #9, #131, #132, #35, #126, #142,
  #75), adresses fantaisistes de `Write64BitValueBottomOfPipeX`, `Close → 0x80004005`, rejets de
  PSO / « console shaders in the PSO without a translation » (domaine d'ivall).
