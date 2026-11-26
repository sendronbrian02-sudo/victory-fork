# Fork VIctory (travail en cours)

Un fork en cours de développement de [VIctory](https://github.com/Marvisak/VIctory),
une couche de compatibilité qui fait tourner nativement sur Windows un titre
Xbox Series X|S, à partir des fichiers que l'utilisateur fournit. Ce fork suit
la branche `main` d'amont et y ajoute des correctifs dans un seul but : que le
monde du jeu s'affiche correctement et de façon jouable sur un PC donné.

**Ce dépôt ne contient aucun fichier de jeu, aucun code de jeu et aucune
ressource du jeu.** Il faut sa propre copie du jeu. Ce fork n'est ni affilié à
l'éditeur ou au développeur du jeu, ni approuvé par eux.

## Crédits

- **[VIctory](https://github.com/Marvisak/VIctory)** par Marvisak et ses
  contributeurs est la base de tout ce qui est ici : le chargeur, les services
  d'exécution, le moteur Direct3D 12 et le recompilateur de shaders. Sous
  licence GPL-3.0-or-later, comme ce fork. L'historique d'amont est conservé
  intact plus bas, à partir du commit qui porte les changements de ce fork.
- **[XCL](https://github.com/ivall/xcl)** par ivall et ses contributeurs
  (GPL-2.0-only) a servi de référence pour comprendre comment le même titre se
  comporte sous une autre couche. Les correctifs qui s'en inspirent ont été
  réécrits dans le code de VIctory.
- La garde posée sur chaque vue d'une page de mémoire est du code de
  **myokex** ([victory-amd](https://github.com/myokex/victory-amd), commit
  `c6dc4d8`), qui adapte la pull request #115 d'ici par Spexivus, sa correction
  par joroyt et une partie de VIctory #1122 par tightfall. Le correctif de
  lecture à côté des lectures noyau dans la mémoire du titre est de
  **Joshua Odom** (commit `10ea38e` de la même famille de forks). Les deux sont
  repris ici sans changement, hormis un conflit de fusion.
- Les changements de ce fork ont été développés avec
  [Claude Code](https://claude.com/claude-code).

## État de ce fork (25/11/2026)

**Pas encore jouable : 2,5 à 4 images par seconde, et les zones à l'ombre sont
noires.** Testé sur une seule machine : Windows 11, Intel i7-14700KF, NVIDIA
RTX 4070 12 Go, 32 Go de RAM. Le profil Series X est utilisé partout.

| Étape | État dans ce fork |
|-------|-------------------|
| Démarrage, menus, écran titre | fonctionne, 30 images/s (comme en amont) |
| Cinématique d'introduction | jouée du début à la fin, dans l'ordre, éclairée : cartons de titre, générique, plans de nuit et de crépuscule |
| Sauvegardes | le jeu enregistre un point de contrôle tout seul et le recharge |
| Première scène de jeu (le prologue) | s'affiche : personnages, textures, ciel, extérieur au soleil ; HUD, menu pause et manette fonctionnent |
| Lumière à l'ombre | manquante : l'éclairage indirect par lancer de rayons et les ombres (lampes, soleil) du jeu ne tournent pas encore, donc tout ce que le soleil ne touche pas directement est noir |
| Images par seconde dans le monde | 2-4 images/s avant le passage de vitesse ci-dessous (420-450 ms par image), 240-350 ms avec les anciens commutateurs (`VICTORY_RELAY=2`, ou `VICTORY_LATE_WAIT=3 VICTORY_WIDE_BACK=1 VICTORY_CARD_BUFFERS_MIN_MB=1 VICTORY_CARD_BUFFERS_MB=1024`) ; au départ c'était 3 à 6 secondes par image. Les commutateurs du passage de vitesse sont maintenant actifs par défaut (voir « Nouveautés » ci-dessous) ; ils ont été mesurés un par un, pas tous ensemble sur cette machine |
| Stabilité | aucune perte de périphérique ni blocage du pilote dans les exécutions normales (un blocage introduit par la mise à jour wave64 a été trouvé et corrigé : sa forme de barrière pour les programmes à 64 voies est désactivée sauf avec `VICTORY_WAVE_WIDE=1`) |

### Limite d'images : plus de plafond à 30, plafond à 120

<<<<<<< HEAD
Le titre se limite lui-même à 30 images/s : il prend l'intervalle d'image le
plus long qui tombe juste sur des multiples de 16,67 ms (60 Hz), et 30 images/s
est le plus bas de ces intervalles. Cette couche lui répond maintenant comme si
l'écran tournait à 120 Hz : l'intervalle le plus court qu'elle accepte est
**8,33 ms (120 images/s)**, quel que soit l'intervalle demandé par le titre.
Le jeu n'est donc plus bridé à 30, et il n'est pas bridé à 120 non plus : il
tourne aussi vite que la machine le permet, 120 étant le plafond.
=======
**Update, 2026-11-25 afternoon: what a frame is made of, measured, and two
changes that hold.** First rooms, about 3,850 draws a frame: 104 ms a frame
(9.6 frames a second) with the set below, from about 120 ms in the
morning. Still far from the console's 33 ms.
- How the title paces itself (new meters in `fence.rs` and
  `console_insn.rs`): it never asks for an event. Each frame begins with
  one small submission and a signal, and a title thread then waits for
  that fence value by reading it in memory with `monitorx` / `mwaitx`
  (AMD-only: about 280,000 emulated exceptions a second on an Intel
  processor, all from this wait). The value is set when the layer has
  finished the whole frame before. So a frame is the time the layer needs
  for one frame's burst of submissions plus about 20 ms of the title's
  own; putting presents behind the line, or giving the title's threads
  more processors, changes nothing (both measured).
- Two walls stand side by side and both are full: the layer's submit
  thread (about 95 ms of a 120 ms frame in the morning) and the card
  (about 102 ms busy, by a timestamp pair around every host list:
  `cardmeter.rs`). Taking time off one alone moved the frame by 0 to 7 %.
- `VICTORY_JOIN_PARTS=1` (off by default): the title's lists wait for
  memory values, which cut its ~15 submissions a frame into ~29 parts;
  parts that follow one another now go to the host as one submission
  (9 a frame), the same lists in the same order. About 6 %.
- `VICTORY_BACK_OWN_VIEW=1` (off by default): what shaders wrote goes
  back into title view memory through a view of the layer's own, so the
  copy that wrote it is not read in again. About 9 % (submit thread 93 ->
  81 ms a frame, 104 -> 80 MB a frame checked). A CORRECTION: an earlier
  run seemed to show washed-out frames with this switch; the same frames
  turn up in about 6 of 21 runs with no switch at all (cause not known),
  and `VICTORY_BACK_OWN_CHECK=1` counted 0 stores gone astray in 1.36
  million.
- Where the card's time goes now (`VICTORY_CARD_METER_PIPELINES=1`, a
  table by pipeline): no pass is large. About 82 ms of the title's lists
  are spread over ~3,400 pipeline bindings a frame; ~2,400 of the
  indirect commands still get a clamp pass of the layer's with draw
  windows on (960 are compute dispatches; about 1,000 are mesh records
  the title keeps scattered through a 100 MB argument buffer, which a
  window of records at one step cannot serve).
- Measured and found to give nothing (switches left in, off):
  `VICTORY_REFRESH_ONCE`, `VICTORY_ASYNC_PRESENT`, `VICTORY_VIEW_KNOWN`,
  `VICTORY_SPREAD`, `VICTORY_QUEUE2=3`, `VICTORY_WINDOW_JUMP`. Each has an
  `=alt` form (twenty seconds on, twenty off, each change said in the
  log) so that one run measures both in the same scene.

**Update, the same night: the first room now renders right on the test
machine** (walls, textures, people, lamps), at 190 to 450 ms a frame (2 to 5
frames a second): the picture work has caught up, the frame rate has not.
What did it, on top of the list below:
- Descriptor records the title copies into its heap as plain bytes are
  mirrored before every submission (pull request #305 by Flyander, after
  upstream VIctory 86a5ed6). Measured here: about 40,000 records a frame
  that no call had announced; their host slots were empty or stale.
- A buffer view that does not start on an element of its buffer's host
  copy gets a copy of its own bytes instead of being refused (after
  upstream 86a5ed6): 3 to 6 such views a frame.
- Room on the card is kept for the render targets and buffer copies a
  world makes in its first frame: 98 buffer copies are made (945 MB) where
  the runs before made 1, and the processor no longer compares 550 to
  710 MB a frame looking for shader writes (25 MB now).
- Levels on demand ran on the card for the first time
  (`VICTORY_LEVELS_ON_DEMAND=1`, still off by default; with the fixes of
  pull request #747 by AscilCH): 3.1 GB of the card not allocated for
  texture levels the title does not hold, no device loss.
- Guarded blocks under storage writes are opened by default; 14 more
  shader programs translate; a program is translated with first-half
  interpolation pairs only if it was refused for one (with that on for
  every program, 8,551 working pixel programs came out changed).
- Frame rate, first cuts on the processor side: the descriptor sync uses a
  cheap fingerprint and leaves draw-table slots to their draws (the pass
  over the heap went from 70-220 ms a frame to about 3), and memory the
  guards already hold is not asked of the write watch (pull request #670
  by Jonirulah). Card copies' pieces can come down inside their own
  flights (pull request #648 by Jonirulah, `VICTORY_CARD_FETCH_INFLIGHT=1`,
  off by default: the copy-back thread's wait went from 181 ms a frame to
  0 here, but the card got busier, so no gain is claimed yet). The frame is
  still 250 to 380 ms: the card is the limit now, most of it the
  per-record clamp before each of some 6,700 indirect draws a frame.
- Pull request #220 (cheaper indirect draws) is IN this tree now, its
  windows off by default (`VICTORY_INDIRECT_WINDOW=1`). History, so nobody
  repeats it: with windows on it lost the device three times here. Its own
  checker (`VICTORY_INDIRECT_WINDOW_CHECK=1`) then showed one list drawing
  90 records of 2^24 vertices from a window, where the per-draw path lets
  eight through; a record of 2^23 vertices an instance or more now runs
  empty. Since that fix: eight runs with windows on, seven without a
  device loss and one that lost the device 44 s into the world (cause not
  found; see the driver watch below). One earlier loss after the fix had
  the in-flight fetch (#648) on as well; do not combine the two. The
  in-flight fetch itself measured slower here in two runs and stays off.
- Frame rate, second night (an RTX 4070; the first room went from about
  3.5-4 to 6-7 frames a second with ALL of the switches below; the
  defaults are unchanged and give about 4.5-5):
  - `VICTORY_ASYNC_SUBMIT=1`: submissions are prepared and sent by a
    thread of the layer, not by the title's render thread, which a
    profile showed inside the submit path for 257 of 292 ms a frame.
  - `VICTORY_QUEUE2=1`: fetches of what shaders wrote go to a copy queue
    (brought back, with the fix for a piece fetched beside a later
    flight); the copy-back thread's wait fell from about 125 to 7 ms a
    frame.
  - `VICTORY_INDIRECT_WINDOW=1`: see above.
  - `VICTORY_TEXTURE_NO_WAIT=1`: a texture refresh no longer waits while
    the title remaps memory; the loading phase had 5 readings over a
    second where it had 18.
  - `VICTORY_MERGE_GAP=64 VICTORY_MERGE_RUNS=4096`: title bytes put in
    under flights' shader writes are joined over small gaps (NOT exact:
    read the comment at `merge_gap` before using it).
  - On by default: fills of upload memory and read-only buffer copies no
    longer wait for flights and are checked against what the flights then
    wrote (`VICTORY_FILL_WAIT=1` waits as before; no mismatch in any run
    so far); up to 512 idle host lists and texture staging buffers are
    reused instead of made and freed every frame.
  - `VICTORY_PROFILE=1` (diagnostics builds): a sampling profiler inside
    the layer; it is how the above was found.
- Frame rate, third session: measuring first (same RTX 4070, first room).
  - A frame log (`VICTORY_FRAME_LOG`, on in diagnostics builds): every
    frame's length, and for the usual frame and each long one what it was
    made of (the render thread's time in the present call, the submit
    thread's work, copies, copy-back, recording, texture work, faults,
    disk). A card meter (`VICTORY_CARD_METER`) adds the card's time by
    kind of list from a timestamp at each list's start and end.
  - What they showed, windows off: a usual frame of 136 ms; the title's
    render thread waits 77 ms of it in the present call for the submit
    thread, which works 118 ms (copies of title memory 41, copy-back 40);
    the card is 85-98 % busy, of about 130 ms a frame 16 ms in our lists
    that bring card copies up to date, 10 ms in our compare lists and 102
    ms in the title's own lists with the per-record clamp. Frame time is
    linear in draws (about 32 microseconds a draw all-in): the title draws
    about twice as much on some frames, which are the long ones.
  - The regular dips of earlier test runs were the test harness: a saved
    frame (`VICTORY_CAPTURE_WORLD`, every 8th world frame) is read back
    and written as a PNG inside the present, 235 ms each.
    `VICTORY_CAPTURE_WORLD_EVERY=64` for runs that measure speed.
  - `VICTORY_INDIRECT_WINDOW=1` measured again with those instruments. A
    CORRECTION: the first figures given here (6.0 against 8.9 frames a
    second) compared two runs that stood in different scenes during the
    measured window (about 4,300 against 1,550 draws a frame) and are
    withdrawn. Compared at equal draws, by a line fitted to each run's
    frame times against its draws per frame: windows off 145 ms at 4,500
    draws and 271 ms at 10,000; windows on with `VICTORY_COPY_PLAIN=1`
    124 ms and 211 ms. On screen: 7-8 frames a second in the first rooms,
    10-12 in their lighter views.
  - A driver watch (on by default; `VICTORY_DRIVER_WATCH=0` off): the
    layer subscribes to the system's event log and logs every entry of a
    display driver as it is recorded; the first one turns draw windows
    off for the rest of the run. Every device loss here had such entries
    2.7 to 55 seconds before the device was removed, and no run without
    them lost its device.
  - A copy between two title textures that share one host texture is no
    longer recorded (the host does not allow a texture copied onto
    itself): 402 of them in a five-minute run.
  - The list of shader-visible heaps is a reader-writer lock (recording
    threads waited 48 ms a frame, summed, behind the heap sync); the heap
    sync looks at a changed page's records under one lock, fingerprint
    first; guarded view memory is left out of a submission's write-watch
    askings ahead (after triippywhale).
  - `VICTORY_COPY_PLAIN=1`: copies of title memory made in place by this
    process, with a handler that falls back to the kernel's copy on a
    missing page. Its first form also took the layer's one-byte "is this
    page there" reads, 1,000 to 1,300 faults a frame, and won nothing;
    reads under 256 bytes now stay with the kernel and a source that
    faulted is left alone for two seconds (4 to 9 faults a frame). The
    submit thread's kernel time fell from 44 to 20 ms a frame; the
    picture is the same.
  - The layer's submit and copy-back threads ask for the fast cores of a
    hybrid processor (`VICTORY_FAST_CORES=0`: not): the submit thread was
    found on a slow core in 6 % of its looks, now in none of 21,059.
    `VICTORY_DISK_CORES=slow` (off, not measured) asks the slow cores for
    the DirectStorage workers, which are as many as the logical
    processors and free to run anywhere.
  - Behind switches, built and unit-tested, not yet shown to pay:
    `VICTORY_REFRESH_ONCE=1`, `VICTORY_VIEW_KNOWN=1`,
    `VICTORY_BACK_OWN_VIEW=1` (picture to be checked),
    `VICTORY_ASYNC_PRESENT=1` (the present goes behind the queue's line;
    no gain while the card is the limit).
  - Tried and refused: `VICTORY_BVH_HOLD=1` holds back the 64 programs
    that walk ray-tracing structures nothing has built; the picture loses
    its indirect light with it, so it stays off.
  - The process opts out of the system's efficiency mode
    (`VICTORY_EFFICIENCY_MODE=1` leaves it); on a hybrid processor the
    frame log found the title's render and recording threads on fast
    cores in every look of a run, and the layer's submit thread on a slow
    core in 6 % of its looks.
  Still open: the per-record clamp is most of the card's time, and the
  copies between the PC's memory and the card's are most of the submit
  thread's; the cause of the one device loss with windows is not known.
  A first play session past the first rooms reached the open world and
  lost the device there with no driver entry (reason "device removed"):
  the card had no room for new textures from the opening cutscene on
  (9.2 of a 10.3 GB budget on a 12 GB card; 2,073 passes made no texture
  for lack of room; 44,613 textures let go and made again in 13 minutes).
  Cause of that removal not known either.
- Known and counted, not fixed yet: an indirect compute pass asks for
  about 77,500 thread groups in x where one dispatch runs 65,535 (the
  rest is cut off).
>>>>>>> origin/main

- `VICTORY_MIN_INTERVAL_US=16667` remet le plafond à 60 images/s,
  `=33333` à 30 (comme avant ce fork), `=0` laisse le titre faire ce qu'il
  demande.
- Le plafond ne fait pas gagner d'images : il enlève une attente. Tant que la
  carte est le facteur limitant (voir plus bas), l'image dure ce que dure le
  travail ; c'est quand le jeu deviendra plus rapide que 30 images/s que ce
  changement se verra.

### Nouveautés : passage de vitesse de cette branche (commutateurs par défaut)

Tous les commutateurs ci-dessous étaient éteints, et chacun a été mesuré comme
payant sur la machine de test. Ils sont maintenant **actifs sauf si une
variable d'environnement dit le contraire**, donc une exécution normale en
profite. Chacun se remet à l'ancien comportement avec la variable indiquée dans
son commentaire, là où il est lu.

| Variable | Maintenant | Ce que ça fait |
|----------|-----------|----------------|
| `VICTORY_LATE_WAIT` | `3` | les soumissions ne prennent plus leur tour avec la carte : une soumission part, et ses clôtures et valeurs de bas de pipe sont données au titre quand la carte l'a exécutée (avant : préparation 134 ms, puis attente de la carte 194 ms, puis copie retour 78 ms, l'un après l'autre). `=0` rétablit l'ancien tour de rôle, et c'est la première chose à essayer si l'image montre un problème introduit par cette branche |
| `VICTORY_RELAY` | `1` | la copie retour de ce que les shaders ont écrit tourne sur son propre thread, hors du verrou de soumission |
| `VICTORY_ASYNC_SUBMIT` | actif | les soumissions sont préparées et envoyées par un thread de la couche, plus par le thread de rendu du titre (un profil montrait le titre dans le chemin de soumission pendant 257 des 292 ms d'une image) |
| `VICTORY_QUEUE2` | `1` | les récupérations de ce que les shaders ont écrit passent par une file de copie, à côté du relais |
| `VICTORY_INDIRECT_WINDOW` | actif | les arguments de draw indirect sont bornés une fenêtre à la fois au lieu d'un enregistrement à la fois |
| `VICTORY_INDIRECT_WINDOW_CHECK` | actif | chaque draw re-borne son enregistrement sur la carte, donc un argument périmé est repéré |
| `VICTORY_COPY_PLAIN` | actif | les copies de la mémoire du titre sont faites sur place, avec la copie du noyau derrière pour une page absente (le temps noyau du thread de soumission est passé de 44 à 20 ms par image) |
| `VICTORY_VIEW_KNOWN` | actif | un bloc déjà gardé reste connu comme mémoire de vue : 338 à 450 interrogations de write-watch ratées et 6,5 à 7,8 ms de `VirtualQuery` par image disparaissent |
| `VICTORY_WIDE_THREADS` | 8 (un quart des processeurs, de 2 à 8) | threads gardés pour les rafraîchissements d'une soumission, les instantanés et comparaisons des plages déclarées, et la fusion de ce que les shaders ont écrit ; le plafond était 6 et il était le facteur limitant de l'image |
| `VICTORY_FULL_CARD_PASS` | actif | quand la carte n'a plus de place pour de nouvelles textures, la passe de textures est sautée au lieu de parcourir ses 18 000 emplacements pour apprendre la même chose |
| `VICTORY_LEVELS_ON_DEMAND` | actif | les copies hôtes des textures du titre prennent la mémoire de la carte niveau par niveau, donc les niveaux que le titre ne garde pas n'en prennent aucune (3,1 Go de la carte non alloués sur la machine de test). `=0` désactive, et c'est la première chose à essayer si une texture affiche de mauvais octets |

**Mesuré sur une deuxième machine** (Windows 11, NVIDIA RTX 4070, dossier
`victory-data` de test) : sans, 3 images/s et des images de 3 s dont 2 130 ms
de préparation de textures ; avec les niveaux à la demande, **27 à 34 images/s
dans les menus du titre** (30 images/s = la limite d'origine du titre, levée
plus bas), préparation ramenée à 5-12 ms par image.

Tout ce que ces commutateurs supposent est écrit là où chacun est lu ; quand
l'un change ce que le titre observe, son commentaire le dit. Si une exécution
montre un problème d'image, désactiver d'abord `VICTORY_LATE_WAIT`
(`VICTORY_LATE_WAIT=0`), puis `VICTORY_RELAY=0`, `VICTORY_ASYNC_SUBMIT=0`.

### Ce que j'ai appris en regardant les autres forks (25/11/2026, soir)

Trois projets font la même chose en public, et j'ai lu les deux plus avancés
pour voir ce qu'ils ont que cette branche n'a pas :

- **ZSlayerHQ/victory-world-runtime**, branche `next` (la ligne à suivre,
  dernier commit `0808b0c`) : le runtime de *victory-world*. C'est la base la
  plus à jour, et elle rassemble les forks publics. Le README y annonce
  « environ 24 Go de RAM et 9 Go de mémoire vidéo dans le monde », et des
  réglages de mémoire Windows (erreur 1455 dans `crashes.log` = pas assez de
  mémoire de commit).
- **DexNom/VIctory**, un fork de cette même base, testé sur Radeon RX 7900
  XTX : il ajoute le **lancer de rayons** (constructeur de structures
  d'accélération, traversée et éclairage) derrière un mode expérimental, des
  correctifs de shaders (échantillonnage 2×2 aux bords de triangles, cube
  maps, wave64, samplers statiques par branche) et la reprise des textures
  éparses. Ses propres mesures donnent **969 ms par image en ville avec le RT
  allumé** (3 469 ms avant son correctif d'écriture), soit environ 1 image/s :
  le RT n'est pas encore une solution de vitesse, même chez lui.

**Ce que ça change pour toi (NVIDIA, pas AMD)** : rien dans ce que fait
DexNom n'est propre à Radeon. Ses correctifs de shaders et de textures sont
du D3D12 ordinaire ; seul son test et ses chiffres le sont. Ce que sa branche
apporte vraiment, et que celle-ci n'a pas encore, c'est le **lancer de rayons**
(chantier 3 ci-dessous, les zones noires) et la **reprise des textures
éparses** (une texture dont seules les pages du milieu sont là s'envoie quand
même). Ces deux-là sont le prochain vrai travail ; recopier le reste ne
donnerait presque rien, puisque la base est la même.

Une mesure que j'ai faite ce soir sur ta machine va dans ce sens : sur un même
dossier de test, cette branche tient **0,8 à 1,0 image/s** avec 260 à 970 ms
de « heap buffers » par image, contre **0,3 image/s et 3 000 ms** pour la base
d'origine avant le passage de vitesse — et les deux tournent sur la même
machine, donc le reste de l'écart avec DexNom est dans la scène (la ville) et
dans le RT, pas dans le pilote.

### Ce qui manque encore pour être jouable (chantiers en cours)

1. **Le bornage par enregistrement dans les propres listes du jeu est
   l'essentiel du temps de la carte** (102 ms sur 130). Les fenêtres en
   enlèvent une grande part, mais le bornage par draw qui reste est toujours
   sur la carte. Il faut une autre façon d'émuler la lecture d'arguments
   enregistrement par enregistrement de la console, pas un commutateur.
2. **Les copies entre la mémoire du PC et celle de la carte restent le
   principal du temps de soumission** même avec le relais et la file de copie :
   le titre réécrit chaque image de grandes parties de sa mémoire, et chaque
   octet lu par les shaders est copié. `VICTORY_INPLACE` (mémoire du titre
   ouverte comme tas graphique, donc les tampons sont utilisés là où ils sont)
   est la sortie ; seul son premier étage existe
   (`VICTORY_INPLACE=1` pour les fenêtres hors tampons, `=2` pour les tampons
   du titre, `VICTORY_INPLACE_PROBE=1` pour la sonde de diagnostic). À
   noter : `=2` est incompatible avec `VICTORY_LATE_WAIT`, donc l'essayer
   demande aussi `VICTORY_LATE_WAIT=0`.
3. **Les zones à l'ombre sont noires** : l'éclairage indirect par lancer de
   rayons et les ombres (lampes, soleil) ne tournent pas encore. C'est de
   l'image, pas de la vitesse, mais c'est ce qui empêche de dire que le fork
   est jouable.

### Nouveautés (25/11/2026, plus tard le même jour)

**Mise à jour, la même nuit : la première pièce s'affiche maintenant
correctement sur la machine de test** (murs, textures, personnages, lampes), à
190-450 ms par image (2 à 5 images/s) : le travail d'image a rattrapé son
retard, pas la vitesse. Ce qui l'a fait, en plus de la liste ci-dessous :
- Les enregistrements de descripteurs que le titre copie dans son tas comme de
  simples octets sont recopiés avant chaque soumission (pull request #305 de
  Flyander, après l'amont VIctory 86a5ed6). Mesuré ici : environ 40 000
  enregistrements par image qu'aucun appel n'annonçait ; leurs emplacements
  hôtes étaient vides ou périmés.
- Une vue de tampon qui ne commence pas sur un élément de la copie hôte de son
  tampon reçoit une copie de ses propres octets au lieu d'être refusée (après
  l'amont 86a5ed6) : 3 à 6 telles vues par image.
- De la place sur la carte est réservée pour les cibles de rendu et les copies
  de tampons qu'un monde fait à sa première image : 98 copies de tampons sont
  faites (945 Mo) là où les exécutions précédentes en faisaient 1, et le
  processeur ne compare plus 550 à 710 Mo par image pour trouver les écritures
  des shaders (25 Mo maintenant).
- Les niveaux à la demande ont tourné sur la carte pour la première fois
  (`VICTORY_LEVELS_ON_DEMAND=1`, toujours éteint par défaut ; avec les
  correctifs de la pull request #747 d'AscilCH) : 3,1 Go de la carte non
  alloués pour des niveaux de textures que le titre ne garde pas, aucune perte
  de périphérique.
- Les blocs gardés sous des écritures de stockage sont ouverts par défaut ;
  14 programmes de shader de plus sont traduits ; un programme n'est traduit
  avec des paires d'interpolation de première moitié que s'il a été refusé
  pour cela (avec ça pour tous les programmes, 8 551 programmes de pixels
  fonctionnels ressortaient modifiés).
- Vitesse, premières coupes côté processeur : la synchronisation des
  descripteurs utilise une empreinte bon marché et laisse les emplacements des
  tables de draw à leurs draws (la passe sur le tas est passée de 70-220 ms par
  image à environ 3), et la mémoire que les gardes tiennent déjà n'est plus
  demandée au write watch (pull request #670 de Jonirulah). Les morceaux des
  copies carte peuvent descendre à l'intérieur de leurs propres vols (pull
  request #648 de Jonirulah, `VICTORY_CARD_FETCH_INFLIGHT=1`, éteint par
  défaut : l'attente du thread de copie retour est passée de 181 ms par image
  à 0 ici, mais la carte est devenue plus occupée, donc aucun gain n'est
  revendiqué encore). L'image est toujours de 250 à 380 ms : la carte est le
  facteur limitant maintenant, surtout le bornage par enregistrement avant
  chacun des quelque 6 700 draws indirects par image.
- La pull request #220 (draws indirects moins chers) est DANS cet arbre
  maintenant, ses fenêtres éteintes par défaut (`VICTORY_INDIRECT_WINDOW=1`).
  Historique, pour que personne ne le refasse : avec les fenêtres allumées,
  elle a perdu le périphérique trois fois ici. Son propre vérificateur
  (`VICTORY_INDIRECT_WINDOW_CHECK=1`) a ensuite montré une liste tirant
  90 enregistrements de 2^24 sommets depuis une fenêtre, là où le chemin par
  draw en laisse passer huit ; un enregistrement de 2^23 sommets par instance
  ou plus tourne maintenant à vide. Depuis ce correctif : huit exécutions avec
  les fenêtres allumées, sept sans perte de périphérique et une qui a perdu le
  périphérique 44 s après l'entrée dans le monde (cause non trouvée ; voir la
  surveillance du pilote ci-dessous).

*(Le reste de cette section historique est repris plus bas, en anglais, tel
qu'il a été écrit pendant la mise au point.)*

### Ce que ce fork a changé (résumé)

Rendu :
- Le culling des meshlets lisait les tampons de scène comme le mauvais type de vue, donc le monde ne dessinait presque rien ; corrigé.
- Les tables de descripteurs par draw pouvaient s'épuiser, après quoi les draws utilisaient des emplacements jamais faits pour eux et la carte se bloquait ; corrigé.
- La lumière empaquetée par les compute shaders du jeu était relue par des copies vides (ciel, sondes, brouillard) ; corrigé pour chaque niveau de mip et chaque couche.
- La reconstruction du brouillard, le masque d'ombre, les cheveux, le skinning par blend-shape, le feu et plusieurs programmes de matériaux ne se traduisaient pas ; corrigé.
- Les rassemblements à un niveau de mip nommé (avec et sans décalage) lisaient le niveau 0 ; corrigé.
- Les échantillonnages à niveau implicite hors des pixel shaders produisaient du DXIL invalide dans 730 programmes compute ; corrigé.
- Les listes de rectangles, les jumeaux profondeur et stencil pour les textures de profondeur écrites par compute, les relectures de textures.
- Le lancer de rayons, en cours et éteint par défaut : les programmes de construction de structure du jeu se traduisent, la mémoire du titre peut être lue par adresse brute depuis les shaders, le test matériel rayon/boîte et rayon/triangle est émulé en DXIL (vérifié contre une référence sur la carte), et 59 des 64 programmes de trace se traduisent. Rien n'est encore éclairé par ça. Voir [docs/fork-ray-tracing.md](docs/fork-ray-tracing.md).

Vitesse et mémoire :
- Les attentes de valeurs en mémoire, les requêtes de write-watch et les requêtes de pages ne parcourent plus les tables de pages à chaque appel.
- Les gros tampons du titre reçoivent une copie dans la mémoire de la carte, avec les écritures des shaders trouvées par hachage sur la carte.
- Les soumissions peuvent partir avant l'attente de la carte (`VICTORY_LATE_WAIT`), avec un thread relais qui dépose les résultats au fur et à mesure (`VICTORY_RELAY`) : dans une première exécution de cinq minutes, le temps d'inactivité de la carte derrière le thread de soumission est passé d'environ 136 à 30 ms par image.
- La mémoire du titre peut être ouverte comme tas graphique pour que les tampons soient utilisés sur place au lieu d'être copiés (`VICTORY_INPLACE`, premier étage seulement ; une sonde a confirmé l'approche sur la carte de test).
- La file de textures bindless n'envoie plus de vues périmées ; la table de vues ne grossit plus sans fin ; la bibliothèque de pipelines est mappée au lieu d'être lue en mémoire.
- Une perte de périphérique intermittente venait du fichier de bibliothèque de pipelines remplacé pendant qu'il était mappé ; corrigé.

- Les shaders traduits reçoivent des passes de nettoyage sur le module fini (code mort, pliage de constantes, réutilisation de valeurs, faits de branche) ; le nombre d'instructions hôte du pack baisse d'environ 43 %, ce qui vaut environ 20 ms de temps carte par image sur la machine de test (`VICTORY_SHADER_OPT=0` les désactive).

La plupart des changements de vitesse plus anciens restent derrière des
commutateurs éteints par défaut (`VICTORY_WIDE_BACK`,
`VICTORY_CARD_BUFFERS_MIN_MB`, `VICTORY_INPLACE`, `VICTORY_BVH_BUILDER`, ...).
Chacun est décrit dans un commentaire là où il est lu.

### Problèmes connus

- Tout ce qui est à l'ombre est noir (voir plus haut).
- L'intérieur de la première pièce de jeu n'est parfois pas dessiné après le passage de la cinématique au jeu.
- Environ 24 Go de mémoire sont utilisés pendant le jeu.
- Quelques lints de style sont désactivés pour ce fork dans le `Cargo.toml` du workspace jusqu'à ce que le code plus récent soit rangé.
- Beaucoup de changements ici sont spécifiques à ce titre et à cette carte graphique, et certains sont marqués `shortcut:` dans le code là où ils approchent ce que fait la console.

---

Tout ce qui suit est le README d'amont de VIctory, inchangé (en anglais) ; sa
section d'état décrit l'amont, pas ce fork.

---

# VIctory

Runs the title (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; VIctory supplies
the platform underneath it and converts the title's data, ahead of time where
it can and on first use where it must (shaders). See
[docs/architecture.md](docs/architecture.md).

## Status (2026-11-23)

**Not playable yet.** The title boots, runs its menus at its own 30 fps and
loads the world, where rendering fails.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | reached and runs with indirect draws; its title card renders, the scene after it is still dark; 0.5-1.8 s per frame |

Tested on one machine: Windows 11, AMD RX 6700 XT.

> **Warning.** Entering the world on a real GPU can currently hang it, and
> Windows then resets the display driver (every monitor goes black for a
> moment). To look further, use a diagnostics build with
> `VICTORY_D3D12_WARP=1`, which renders on the software adapter.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Most of the scene is still missing (untranslated shaders drop draws); frames take seconds; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

## Getting started

Pour compiler sur Windows, exporter les ressources ou mesurer les 30 FPS
dans le monde : [guide Windows en français](docs/guide-windows-fr.md).

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title:

```powershell
cargo build --release -p victory
target\release\victory.exe run <game dir>\<the title's exe> --execute
```

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
                  [--export-dxil <directory>]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_DUMP_TARGETS=1` | with the two above, saves every render target of those frames |
| `VICTORY_TEXTURE_DUMP=<dir>` | saves up to 256 DDS snapshots of 2D texture mips as the title uploads them; `VICTORY_TEXTURE_DUMP_AFTER_FRAME=<n>` delays export |
| `VICTORY_D3D12_DEBUG=1` | host debug layer |
| `VICTORY_D3D12_GBV=1` | with the above, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_PIPELINE_WAIT_MS=24` | reduce the per-frame wait for background pipeline creation from the default of 500 ms. A draw can be omitted until its pipeline is ready |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
