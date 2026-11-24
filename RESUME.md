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

## État du dépôt (2026-11-24)

- **L'upstream est `xcl-dev/xcl`** (`ivall/xcl` y redirige) : 7 300 étoiles, 811 PR ouvertes, 314 commits en un
  jour. Se mettre à jour : `git fetch https://github.com/xcl-dev/xcl main` puis fusionner dans `main` du fork.
- `main` du fork = fusion de l'upstream 9e3831e (db638f3) + les diagnostics du fork (notes de device removal,
  traçage des stubs par préfixe, `code bytes` sur chaque ligne de crash) + `HANDOFF.md` à jour (section
  « Upstream moved to xcl-dev… » : faits tirés de VIctory et AnyXboxX, deux projets GPL-3 sur le même titre –
  faits seulement, jamais de code : XCL est GPL-2).
- PR : voir `PRS.md`. #128 (f16) mise à jour et poussée (2bc03fa), attend ton run GPU et ton commentaire.
  #194 à fermer (remplacée par `XCL_ACTIVITY`). Le reste est fusionné ou obsolète.
- Branches locales `pr/*` autres que `pr/f16-directed-rounding` : obsolètes, ne rien en ouvrir.

## Ce que l'utilisateur doit faire
1. `git pull`, `cargo build --release`, et **lancer en mode Series S** : `device_type = "lockhart"` dans la
   config (le titre budgète sa mémoire d'après le type de console : 8 Go au lieu de 13,5 ; AnyXboxX mesure
   +50 % de fps en 720p interne). Toujours `XCL_NETWORK=0`. Coller le log : dernière ligne `XCL_ACTIVITY`
   avant un crash, la ligne de crash avec `code bytes`, ce que la fenêtre montre.
2. Tests GPU sur `pr/f16-directed-rounding`, puis le commentaire de `PRS.md` sur #128 ; fermer #194.

## Ensuite
- L'upstream atteint les réglages après le premier menu avec ~15 Go de commit ; VIctory charge le monde (50 s)
  mais l'image y reste noire ; AnyXboxX y a un personnage contrôlable à 2,4 fps. Les leçons transférables sont
  dans HANDOFF.md ; XCL a déjà le plafond de descripteurs (OPTIONS19), la garde de boucle (`xcl_budget` 2^16),
  `XMemGetWorkingSetStatistics` par type de console, le mode Lockhart. Manque : un garde-fou GPU des
  enregistrements d'`ExecuteIndirect` (VIctory borne sommets / instances / groupes par une passe HLSL), et un
  traçage des compteurs d'arguments par passe quand une scène reste noire.
