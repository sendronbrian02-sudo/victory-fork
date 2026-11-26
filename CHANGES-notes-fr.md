# Victory-fork — notes de travail (français)

## 0. Le dossier a été remplacé par flamingo, renommé Tirsos (25/11)

Ton `C:/Users/sendr/Desktop/victory-fork` est maintenant une copie de
[`realblastex/flamingo`](https://github.com/realblastex/flamingo) — le fork
le plus à jour — et non plus le mien. L'ancien dossier est gardé à côté dans
`C:/Users/sendr/Desktop/victory-fork-backup-2228`. Branche poussée sur ton
dépôt : `tirsos` (le `main` distant est resté l'ancien historique).

Les sections 1 à 10 plus bas décrivent l'ancien travail (fork de victory-fork)
et ne s'appliquent plus au code ; elles restent pour mémoire.

## 11. Ce que j'ai corrigé dans tirsos (mesuré sur ta RTX 4070)

| # | Changement | Fichier | Effet mesuré |
|---|---|---|---|
| 1 | La copie en place ne continuait que sur une page `PAGE_READWRITE` stricte : toute page en `WRITECOPY` / `EXECUTE_READWRITE` renvoyait la copie entière au noyau, page par page | `crates/victory-gpu/src/watch.rs` (`writable`) | une partie des copies de 120-190 Mo par image |
| 2 | La passe de tas redemandait ~4 000 copies (12 000 slots) par image, la plupart couvrant exactement les mêmes octets que l'image d'avant | `crates/victory-gpu/src/draw.rs` | passe de tas **195-873 ms → 84 ms** (slots relus 33 321 → 13 362, textures 398 → 18 ms) |
| 3 | Les écritures de shaders étaient cherchées sur **un seul fil** au-delà de 16 Mo par fil : 340 plages de 22,9 Mo à la suite par image | `crates/victory-gpu/src/shadow.rs` (`differing`) | **4,4 → 8,2-9,6 fps** ; la frame lente type tombe de 532 à 217 ms |

Réglage important trouvé en route : `VICTORY_EARLY_TEXTURES_MS=4` (au lieu de
12). Au-delà, la passe textures prend 250-750 ms sur les images où la carte
reçoit beaucoup de géométrie neuve ; à 0 c'est bien pire. C'est ce qui fait
passer de « 1 à 4 fps » à « 4,5 à 9,6 fps ».

Les trois correctifs sont des commits sur la branche `tirsos`.

## 12. Ce que valent les deux conseils qu'on t'a donnés

**« map shared buffers to a ReBAR WRITE_COMBINE custom heap »** — tirsos
fait déjà le mapping « une seule mémoire » (`inplace.rs` : `OpenExistingHeap
FromAddress` sur la mémoire du titre) et ses copies de lecture passent par
`watch::try_copy` (copie directe, faute rattrapée, aucun appel noyau), pas par
une boucle `VirtualProtect` + `memcpy`. Essayé : `VICTORY_INPLACE=1` place
87 fenêtres (408 Mo) et 918 refus (mémoire de vue ou non privée) → **3,8-4,4
puis 1,7 fps, aucun gain**, exactement comme le journal de tirsos l'avait
mesuré (5,6-6,9 fps, plus des rectangles noirs). La limite n'est pas le PCIe :
le compteur dit 84 Mo envoyés en 480 plages, et le coût est CPU (comparaison
et copie), pas le bus.

**« hook FlushCpuCacheRange »** — pas un problème chez nous : rien de tel
dans ce code, et le conseil parle de **RAGE** (GTA/RDR), pas de notre titre.

**« GPUVA shadow table for bindless / ExecuteIndirect »** — c'est déjà le
principe de `indirect.rs` + `heap_pass.rs` ; le reste à faire est là
(« indirect » restait à 55-2 158 draws jetés par image dans les anciens runs,
à 0 maintenant).

## 13. Ce qui reste à changer

1. `VICTORY_EARLY_TEXTURES_MS` : trouver le bon chiffre entre 4 et 12, et
   faire en sorte que la passe textures prenne le temps restant de l'image
   plutôt qu'un budget fixe.
2. `VICTORY_INPLACE_POOL` (victory-fork #772, absent de tirsos) : il vise
   justement les 918 refus dus au pool de pages `SEC_RESERVE`.
3. Les zones sombres : pas de ray tracing, et `VICTORY_AMBIENT` (constant)
   ne se déclenche pas ici — le programme de résolution n'a pas de nom dans
   ce build (`VICTORY_AMBIENT_PROGRAM=<nom>` pour le forcer).
4. Manette PS5 : lecture HID faite côté victory-fork, à reporter sur tirsos
   si le mode DualSense de tirsos ne suffit pas.
5. Le plafond d'images : à vérifier (120 visés) une fois qu'on tient 30+ fps.

# Ce qui a changé, et ce qu'il reste à changer

Note tenue au fil du travail, en français. Elle dit ce qui a été fait (avec
les chiffres mesurés) puis, à la fin, ce qu'il faut encore changer, du plus
utile au moins utile. La version anglaise d'avant ce soir reste à côté, dans
`CHANGES-passage-de-vitesse.en.md`.

---

## 1. Fait : le passage de vitesse (commutateurs par défaut)

Le fork portait déjà les bons commutateurs, mais tous éteints : une exécution
normale prenait le chemin lent. Ils sont maintenant actifs par défaut, chacun
remis à l'ancien comportement par sa variable.

| Variable | Défaut maintenant | Ce qu'elle évite |
|---|---|---|
| `VICTORY_LATE_WAIT` | 3 | le titre et nous attendons chacun notre tour |
| `VICTORY_RELAY` | actif | une copie de relais en trop par envoi |
| `VICTORY_ASYNC_SUBMIT` | actif | l'envoi qui bloque le fil du titre |
| `VICTORY_QUEUE2` | 1 | la file de copie non découpée |
| `VICTORY_INDIRECT_WINDOW` (+ `_CHECK`) | actif | le bornage par enregistrement partout |
| `VICTORY_COPY_PLAIN` | actif | la copie page par page |
| `VICTORY_VIEW_KNOWN` | actif | les interrogations `VirtualQuery` ratées |
| `VICTORY_WIDE_THREADS` | 8 (avant : 6) | le nombre de threads qui bridait l'image |
| `VICTORY_FULL_CARD_PASS` | actif | la passe de textures quand la carte est pleine |
| `VICTORY_LEVELS_ON_DEMAND` | actif | les niveaux de texture que le titre ne garde pas |

## 2. Fait : le plafond d'images, de 30 à 120

Le titre se limite à 30 images/s en prenant l'intervalle le plus long qui
tombe juste sur des multiples de 16,67 ms. La couche répond maintenant comme
un écran à 120 Hz : l'intervalle le plus court gardé est 8,33 ms.
`VICTORY_MIN_INTERVAL_US` en règle un autre (16667 = 60, 33333 = 30 comme
avant, 0 = ce que le titre demande).

À savoir : cela enlève une attente, cela n'ajoute pas d'images quand c'est la
carte qui limite.

## 3. Fait : les niveaux à la demande (le gros gain)

Le titre crée ses textures avec tous leurs niveaux, mais n'en remplit qu'une
partie. La couche prenait la mémoire de la carte pour tous les niveaux :
**2 130 ms de préparation de textures par image** et 3 images/s sur le dossier
de test. Avec les niveaux à la demande, seuls les niveaux que le titre
remplit prennent de la mémoire.

| Mesure (même machine, même dossier) | avant | après |
|---|---|---|
| images/s | 3 | **27 à 34** (menus) |
| ms par image | 3 034 | 29 à 37 |
| préparation des textures | 2 130 ms | **5 à 12 ms** |
| mémoire carte non allouée | — | 3,1 Go en moins |

Son test sur la vraie carte passe (`cargo test -p victory-gpu --release tiles
-- --ignored` avec `VICTORY_TILES_TEST_CARD=1` : « all steps passed ON THE
CARD »).

## 4. Fait : la manette PlayStation

Le titre ne lit que XInput, et une manette DualSense ou DualShock 4 n'est pas
un périphérique XInput : elle n'était donc pas là du tout. Le premier
périphérique HID que Windows présente et qui se déclare « gamepad » est
maintenant ouvert et lu en entier ; chaque usage est pris par son numéro, donc
la disposition du rapport est celle de Windows et pas une table à nous. Ça
couvre DualSense, DualShock 4 et Switch Pro. `VICTORY_HID_PAD=0` désactive.

Non fait : les moteurs d'une manette HID ne sont pas encore écrits ; l'ancien
chemin XInput les fait pour une manette Xbox.

## 5. Fait : des sous-objets de pipeline en plus

Le flux de pipeline du titre nomme deux choses que la couche ne savait pas
lire (le nombre d'échantillons forcé du rasteriseur, et les formats de cible
un par un). Elles sont lues ; un flux qui les nomme n'est plus refusé.

## 6. Fusion de l'amont et méthode 0x218 (25 nov.)

- **L'amont `fxceless/victory-fork` est fusionné** (`3973179`, 16 fichiers :
  ce qui fait une image, les parties jointes, les copies de retour par notre
  propre vue, un compteur d'images dans le coin, les enregistrements indirects
  ramassés une fois par liste). Un seul conflit, sur `watch.rs` : notre
  `VICTORY_VIEW_KNOWN` allumé par défaut est gardé, et le mode `=alt` de
  l'amont (vingt secondes avec, vingt sans) est ajouté à côté. 176 tests
  passent. Mesure après fusion au dossier de test : **1,1 à 1,5 s par image**
  (contre 4 à 5 s avant), soit environ 0,6 à 0,9 images/s au chargement.
- **La méthode de création de pipeline au slot 0x218 est écrite.** C'est
  `CreatePipelineState` sur un flux de sous-objets, la même forme que la
  méthode 0x1a8 que la couche savait déjà lire : elle est maintenant lue par
  le même code. Le titre fait 13 appels au lancement et les 13 réussissent
  (`VICTORY_PIPELINE_STREAM=0` revient à l'ancien refus).
  **Ce que ça ne fait pas** : ces 13 flux commencent tous par un sous-objet
  de type console `0x1c`, dont on ne connaît pas la forme ; les shaders d'un
  état d'objet (lancer de rayons) n'y sont pas. Ce qui est rendu est donc un
  pipeline sans étage : le titre ne reçoit plus un échec, mais ce n'est pas
  un pipeline de lancer de rayons. Les 969 ms par image de DexNom viennent
  d'un travail qui reste à faire.

## 7. Repris de DexNom (25 nov.)

Trois correctifs de [DexNom/VIctory](https://github.com/DexNom/VIctory), portés
et adaptés à notre branche :

- **Une image qui change sans que le garde l'ait vue** (le film de l'intro :
  trois images qui changent une fois sur huit) était montrée jusqu'à huit
  images en retard, d'où les changements d'image brutaux au lancement. Elle
  est maintenant hachée à chaque vérification pendant 16 images après une
  première découverte (120 si c'était déjà le cas), et les images linéaires
  d'un seul niveau (les plans du film) sont toujours hachées.
  `crate::texture::Staging::unseen_write`.
- **Les copies de retour de la carte** (ce que les shaders écrivent dans la
  mémoire du titre) étaient une par morceau de 64 Kio. Celles qui se suivent
  dans la même copie sont jointes en une seule copie ; un compteur les compte
  (`fetches N pieces whole, M joined` dans la ligne de profil).
  `VICTORY_FETCH_JOIN=0` revient à l'ancien comportement.
- **La fenêtre d'envois indirects** était plafonnée à 64 enregistrements par
  passe : 54 passes par image pour 3 460 envois. Le plafond est passé à 256
  (réglable de 4 à 4096 par `VICTORY_INDIRECT_WINDOW_MOST`).

## 8. Essayé et abandonné (mesuré, pas supposé)

Deux pistes ont été écrites, compilées, mesurées, puis retirées parce
qu'elles n'aidaient pas :

- **Un budget de 8 ms par image sur la passe des tampons du tas**
  (`heap_buffers`, 260 à 970 ms par image). Quand la passe est coupée, les
  copies de l'image suivante sont plus grosses : le total ne bouge pas.
- **Lire une texture partiellement en mémoire niveau par niveau** (une
  texture dont seuls les niveaux du milieu sont arrivés). Le détilage refuse
  chaque niveau (`TooSmall`) et le total des textures passe de 310 à
  10 400 ms par image. La bonne façon est celle de DexNom (reprendre les
  textures éparses à l'envoi), pas de découper la lecture ici.

## 9. Niveaux de texture lus un par un, bornages hissés (ce passage)

**Les niveaux d'une texture sont lus un par un** (`texture.rs`, nouveau
`Layout::span` dans `victory-formats`). Avant, une texture n'était envoyée
à la carte que si *toute* sa mémoire était arrivée. Or le titre fait
arriver les pages du milieu d'abord : une texture restait donc hors de la
carte pendant des frames entières, et s'affichait en noir. Maintenant
chaque niveau dont la mémoire est là est détilé, et un niveau qui n'est
pas encore arrivé est mis à zéro (noir) jusqu'à ce qu'il le soit : la
texture s'affiche tout de suite, en basse résolution d'abord.
`VICTORY_SPARSE_LEVELS=0` revient à l'ancien comportement.

Deux corrections de la même famille :
- une texture dont la **première page** n'est pas encore mappée mais dont
  les pages du milieu le sont n'est plus prise pour une texture absente
  (`any_mapped`, repris de DexNom) ;
- `Layout::span` traite le mode linéaire à part (ses motifs de ligne et de
  colonne sont vides : c'est ce qui a fait planter le premier essai).

**Les bornages d'une suite de draws indirects sont hissés** (`indirect.rs`,
`host.rs`, `draw.rs`) : au lieu d'un bornage (un dispatch de compute) entre
chaque paire de draws, ils vont dans une liste de commandes à part qui
s'exécute juste avant celle qui dessine. Une suite de draws ne fait plus
qu'un aller-retour dessin → compute au lieu d'un par draw. C'est la
modification de Belobug (fork AMD), où 225 ms de temps carte par frame
sont passées à environ 80 ms dans une scène de 9 000 draws.
`VICTORY_HOIST_CLAMPS=0` revient à un bornage devant chaque draw.

Mesures sur cette machine : 176 tests passent ; le lancement va jusqu'au
monde, 30 fps constants dans les menus (le plafond), 0 refus de detile,
0 plantage sur 4 minutes.

## 10. Ce qui reste à changer, du plus utile au moins utile

1. **Les zones à l'ombre sont noires** : le lancer de rayons (l'éclairage
   indirect, les ombres) ne tourne pas. C'est ce que le fork de DexNom a en
   plus, et ce n'est pas une question de pilote : c'est du D3D12 ordinaire,
   donc ça marchera aussi sur NVIDIA. Son propre chiffre, en ville avec le RT
   allumé, est **969 ms par image** : c'est de l'image avant d'être de la
   vitesse. Gros chantier.
2. ~~**Les textures éparses**~~ : **fait au passage 9**. Les niveaux sont lus
   un par un ; ce qui reste possible est d'aller plus fin, bloc par bloc à
   l'intérieur d'un niveau, pour les très grandes textures.
3. **Le bornage par enregistrement** dans les listes du jeu reste l'essentiel
   du temps de la carte (102 ms sur 130). Les fenêtres en enlèvent une part ;
   le reste demande une autre façon d'émuler la lecture d'arguments de la
   console. Pas un commutateur.
4. **Les copies mémoire PC ↔ carte** restent le principal du temps de
   soumission. La sortie est `VICTORY_INPLACE` (les tampons du titre utilisés
   là où ils sont) ; seul son premier étage existe. `=2` est incompatible avec
   `VICTORY_LATE_WAIT`, donc l'essayer demande aussi `VICTORY_LATE_WAIT=0`.
5. **La passe sur les tampons du tas** (« heap buffers ») coûte 260 à
   970 ms par image sur le dossier de test. Un budget par image a été essayé
   et **n'aide pas** : quand la passe est coupée, les copies de l'image
   suivante sont plus grosses, et le total ne bouge pas. Le vrai travail est de
   ne pas copier du tout (point 4).
6. **Les micro-freezes au lancement** viennent de deux choses mesurées : la
   création des pipelines (`host pipeline refused`, et deux méthodes de
   création que le titre appelle et que la couche refuse) et les textures qui
   arrivent en retard. Le point 2 règle une partie des secondes ; les
   pipelines demandent d'écrire les deux méthodes manquantes.
7. ~~**La base est en retard sur l'amont**~~ : fusionné au passage 6.

8. **Ce que le fork de Belobug a et que nous n'avons pas encore** : son
   `VICTORY_STRETCH` (la mémoire du titre vue un bloc de 64 Kio à la fois est
   utilisée là où elle est : copies de retour de 36 à 12 ms) et son
   `VICTORY_AMBIENT` (la lumière indirecte remplacée par une constante, sinon
   les zones que le soleil n'atteint pas sont noires). Le second est un
   pansement d'image, mais il rend le monde regardable tout de suite.

## 14. Fence par ressource, et le retour carte (25/11, après-midi)

### 14.1 Ce qui a été fait : une fence par ressource

Le niveau 2 de l'inplace (les tampons du jeu placés directement sur leur
propre mémoire, sans copie) était documenté comme **incompatible** avec
`VICTORY_LATE_WAIT` : les soumissions ne sont pas attendues, donc un tampon
placé pouvait être lâché alors qu'une liste que la carte n'avait pas encore
exécutée le nommait encore — et son tas partait avec elle.

C'est réparé (`crates/victory-gpu/src/inplace.rs`, `struct Fenced`) :

- chaque tas placé retient **les numéros de soumission** qui nomment son
  tampon (`touch`), rempli au moment de l'envoi à la carte
  (`host.rs`, `submit`, à côté de `note_upload_fence`) ;
- quand le tas est lâché (`let_go`, appelé par le `Drop` du tampon), on
  attend la **fence de la dernière soumission de ce tas seulement**
  (`host::fence_flight`), pas la queue entière comme avant ;
- une liste réinitialisée sans être soumise reprend sa déclaration
  (`drop_list`), sinon le tas attendrait une soumission qui n'existe pas ;
- `defer_refresh` n'enfile plus de `Step::Refresh` pour un tampon placé
  (il n'y a rien à copier) : il note la plage pour la fence.

**Mesure** : `VICTORY_INPLACE=2 VICTORY_LATE_WAIT=3` tourne désormais sans
perte de device, avec **3 262 tampons titre (29 Mo) placés et utilisés en
place** (87 fenêtres, 408 Mo, en plus). C'est la première fois que le
niveau 2 est utilisable avec `LATE_WAIT`.

**Mais** le niveau 2 reste plus lent que le niveau 1 : 29 Mo de petits
tampons contre 408 Mo de fenêtres. 1,0 fps contre 1,9 fps sur les dix
premières secondes de la scène, et les images calmes sont les mêmes.
La machinerie est là ; elle ne se paie pas encore. La suite serait de
placer les tampons de plus de 64 Ko qui sont aujourd'hui refusés, pas les
petits.

### 14.2 Le retour carte : ce que j'ai vérifié, et ce que ça a donné

Le conseil relayé (« est-ce que le readback de 24 ms bloque le
pipeline ? ») : oui, il bloque, mais il n'est pas 24 ms. Sur les runs de
référence il fait **40 à 113 ms par image** (« copying back ») et il est
**synchrone** : `card_back` copie ce que les shaders ont écrit, signale une
fence, attend l'événement, puis fusionne. Rien ne peut avancer pendant ce
temps.

Le relais (`VICTORY_RELAY`) le fait déjà en différé. Deux essais :

| configuration | images calmes | verdict |
|---|---|---|
| `VICTORY_INPLACE=1` (référence) | 30,0 fps (plafond) | — |
| `+ VICTORY_CARD_FETCH_INFLIGHT=1` | 30,0 fps, puis 1,0-2,7 fps | **plus lent** : un compte à rebours d'une image |
| `+ VICTORY_RELAY=2` | 30,0 fps, puis 0,5-2,7 fps | **plus lent** : les fusions se font sur le thread qui soumet |
| `+ VICTORY_LATE_WAIT=3 + RELAY=2 + INFLIGHT=1` | 30,0 → 3,0 fps | **plus lent** |

Le retour carte ne coûte donc rien quand les images calmes sont à 30 fps :
il n'y a presque rien à ramener (0,0 Mo). Les 40-113 ms n'apparaissent que
sur les images lourdes, et le recouvrir coûte plus qu'il ne rapporte ici.
Je n'ai rien activé par défaut : `card_back` reste synchrone.

### 14.3 Découverte au passage : le plafond réel est 30, pas 120

Sur toutes les courses d'aujourd'hui, les images calmes sortent à
**30,0 fps / 33,3 ms**, y compris sans aucun de mes changements. Le présent
se fait avec `Present(0, DXGI_PRESENT(0))` (vsync, pas de tearing) et un
swap chain à **2 buffers** : c'est le compositeur qui tient le rythme, pas
nous. Quand on aura de la marge (l'image préparée en moins de 33 ms), le
prochain gain sera de demander un swap chain à 3 buffers, ou
`ALLOW_TEARING`, pour dépasser 30 — la limite des 120 fps visée n'est pas
atteignable avec le présent actuel.

### 14.4 Ce qu'il reste à faire

1. `VICTORY_INPLACE_POOL` (#772) : placer les tampons de plus de 64 Ko que
   le noyau refuse aujourd'hui (918 refus dans le journal de tirsos).
2. Le plafond 30 fps (swap chain à 3 buffers / tearing).
3. Les images lourdes : 250 ms de textures quand la carte reçoit beaucoup
   de géométrie neuve.
4. Vérifier à la main la manette PS5 et le retour de force.

## 15. Le plafond 30 fps cassé, la mémoire mesurée (26/11)

### 15.1 Ce n'est pas la cadence d'affichage, c'est le titre

3 buffers seuls ne changent rien (30,0 fps mesuré). La limite venait du
**titre qui demande au device un intervalle de 33,333 ms** (`frame.rs`,
`set_interval`, journal « frame interval interval=33.333ms »). Le plafond
est maintenant réglé par `VICTORY_FRAME_US`, **par défaut 16 667 µs
(60 fps)** ; à 8 333 (120 fps) quatre images d'affilée sont sorties à
120,0 / 120,1 / 119,8 / 120,0 fps. Le présent bloquait aussi : il faut
`VICTORY_TEARING=1` (désormais par défaut) pour que le présent ne
attende pas le retour du balayage.

Mesure du jour, avec les défauts : 56,4 puis 60,0 / 60,0 fps sur des
images légères, 30,0 quand une image coûte plus de 16,7 ms, 0 panique,
0 device perdu.

### 15.2 La mémoire du processus, et où elle va

Nouvelle ligne, toutes les quatre secondes, dans la ligne de cadence
(`copystat::log_memory_now`) : ensemble de travail, privé, libre sur le PC.

Une session de 3 min 20 : **3 667 MB d'ensemble de travail et 7 972 MB de
privé au début, 6 908 MB et 12 551 MB à la fin**, soit environ 1,4 GB de
privé gagné par minute. C'est cette croissance qui termine une longue
session, pas un plantage.

Avec `VICTORY_MEMORY_MAP_ALWAYS=1`, la carte mémoire du processus donne les
postes (fin de session, « in memory » = réellement en RAM) :

| poste | réservé | en RAM |
|---|---|---|
| tampons partagés avec la carte (write-combined) | 11 428 MB | 2 194 MB |
| vues des pages du titre | 6 055 MB | 3 077 MB |
| réserve de pages du titre | 6 236 MB | 0 MB |
| tas du titre | 4 629 MB | 2 070 MB |
| segments du tas du processus | 3 037 MB | 3 019 MB |
| autres allocations privées ≥ 1 MB | 2 656 MB | 2 624 MB |
| fenêtres de mémoire du titre et blocs de 4 MB | 1 607 MB | 1 537 MB |

Les plus grosses allocations privées : **346 blocs de 32 MB write-combined
(7 854 MB)**, 39 de 32 MB (1 238 MB), puis des blocs de 5 à 30 MB.

### 15.3 Qui tient la mémoire, et qui ne la lâche pas

- **Copies des cibles de rendu** : 84 vivantes, 596 MB, « jamais lâchées »
  par le balayage (`resource::let_go_idle`), dont 652 des 1 295 modifiées
  dans les 30 dernières secondes (2 340 MB) et **643 nommées par une autre
  de nos références (2 516 MB)** ; 0 tenues par la carte. Même problème de
  durée de vie qu'une copie de tampon : quelque chose garde son `Arc` (un
  slot, une liste) et rien ne le relâche. 6,4 GB en 3 min, ~120 nouvelles
  par seconde.
- **Fenêtres hors tampons** : 4 161 vivantes, 3 100 MB ; 2 328 MB de
  fenêtres et **1 148 MB dont le titre a l'adresse GPU**. Le balayage ne
  peut lâcher que ce que rien ne nomme, et il le fait (2198 MB lâchés en
  tout sur la session) : le reste est tenu par le titre ou par un slot.
- **Copies en mémoire carte** : plafonnées à `VICTORY_CARD_BUFFERS_MB`
  (1024) alors que le budget de la carte laisse 7 733 MB libres. Un
  message dit maintenant pourquoi une copie n'y va pas.

### 15.4 Ce qu'il reste à faire, dans l'ordre

1. **Cibles de rendu** : trouver qui garde l'`Arc` de leur copie et le
   relâcher quand la texture n'est plus décrite ; c'est 90 % de la
   croissance (2 516 MB tenus par une référence de plus sur 84 textures).
2. **Copies en mémoire carte** : monter `VICTORY_CARD_BUFFERS_MB` (1024 →
   4096) pour que les tampons de plus de 64 Ko sortent de la RAM partagée ;
   c'est l'item 4 demandé, et le plafond en est la seule raison.
3. **Images lourdes** : la carte (60 ms médiane) et le retour carte
   (47 ms) restent les deux premiers postes ; la ligne « heap buffers »
   dit combien de temps passe dans la passe des textures choisies par index.
4. Manette PS5 et retour de force : à vérifier à la main.

## 16. Où va la mémoire, mesuré sur une course de 4 min (26/11)

Course avec `VICTORY_CARD_BUFFERS_MB=4096`, défauts par ailleurs, 3 min 58,
0 panique, 0 device perdu.

| | début | fin |
|---|---|---|
| mémoire privée du processus | 8 744 MB | 14 824 MB |
| RAM libre sur le PC | 16 035 MB | 10 862 MB |
| textures hôtes | 610 (848 MB) | **9 238 (5 766 MB)** |
| dont côté carte, jamais envoyé au titre | 433 (2 029 MB) | **~8 500 (3 729 MB)** |
| copies de tampons du titre | 1 (174 MB) | 4 (186 MB) |
| tampons en mémoire carte | 56 (416 MB) | 376 (1 206 MB) |
| place sur la carte | in use 50 MB | in use 7 490 MB / budget 15 211 |

Le vrai poste n'est donc pas là où le premier regard portait : les copies
de tampons restent à 186 MB. **Ce sont les textures que l'hôte fabrique
pour le compte du moteur, et garde en mémoire du PC alors qu'elles ne sont
jamais relues par le titre, qui font les 4,9 GB de croissance.**

Détail : 8 803 de ces copies vivantes (3 729 MB) sont dites « placées »,
c'est-à-dire lues ou écrites par le titre ; 57 seulement ne le sont pas,
et le balayage n'en a fait revenir que 274 (291 MB) depuis le début. Le
titre écrit dans 2 116 de leurs pages, et la même adresse physique en
change : à chaque fois une nouvelle copie d'hôte est faite et l'ancienne
reste jusqu'à ce que le balayage la voie idle — mais elle n'est idle
jamais que des fenêtres, pas des textures placées.

### 16.1 Ce que ça implique, dans l'ordre

1. **Textures hôtes des cibles rendues par le GPU (2 029 MB) que le titre
   ne relit jamais** : si le titre ne les relit pas, leur copie ne sert
   qu'à la carte ; une fois le rendu fini et lu par ses passes, elle peut
   être lâchée (ou déplacée en mémoire carte, 7,7 GB libres). C'est le
   premier gain, et il ne touche pas au rendu.
2. **Textures hôtes que le titre écrit (3 729 MB « placées »)** : la même
   adresse réécrite fait une copie neuve ; il faut réutiliser celle qui
   existe au lieu d'en rendre une autre.
3. `VICTORY_INPLACE_POOL` (#772) : 420 refus de placement, 918 plus tôt ;
   les tampons de plus de 64 Ko restent à faire (item 4).
4. Le plafond des copies carte (`VICTORY_CARD_BUFFERS_MB`) n'était pas la
   contrainte : à 4 096 MB, 376 tampons seulement y vont (1 206 MB), le
   reste des copies restant dans le PC.

## 17. Trois minutes plus tard, sans interruption (26/11)

Course complète avec les défauts (60 fps, tearing, `VICTORY_INPLACE=1`),
3 min 09 d'affilée, **aucune panique, aucun device perdu, le jeu ne
s'arrête pas**. Les défauts sont donc sains sur la durée ; c'est la
mémoire qui est le limiteur.

- images : 4 au-dessus de 55 fps, 45 entre 29 et 55, 40 en dessous de 29 ;
  **médiane 29,9 fps, meilleure 60,0** (le plafond demandé).
- mémoire privée : **8 749 MB au début, 24 923 MB à la fin**, RAM libre du
  PC de 15 375 à ~10 400 MB. Le poste est celui du §16 (les textures
  hôtes), pas les copies de tampons.
- le budget de la carte baisse (14 145 → 13 878 MB) quand Windows la
  réclame, ce qui confirme que la carte n'est pas la ressource à pousser
  tant que les textures restent dans le PC.

### 17.1 Ce qu'il faut faire ensuite, dans l'ordre

1. **Textures hôtes que le titre ne relit jamais** (2 029 MB au milieu du
   chargement, ~3,7 GB en jeu) : les lâcher une fois la passe qui les lit
   finie, ou les mettre en mémoire carte (7,7 GB libres).
2. **Textures hôtes que le titre écrit** : réutiliser la copie existante
   quand la même adresse est réécrite, au lieu d'en refaire une.
3. `VICTORY_INPLACE_POOL` (#772) : 886 tas refusés par le noyau (« view
   memory or not private memory committed whole »), 80 fenêtres placées
   sur 966 — c'est l'item 4 demandé.
4. Les images lourdes : 30 ms de passe sur le tas, 20 ms de rafraîchissement
   de textures, 83 ms d'attente de la carte par image.

## 18. Les shaders du jeu, tous, passé au traducteur (26/11)

Commande (celle que l'utilisateur proposait, avec le bon nom) :

```
.\target\release\victory.exe translate --list --jobs 8 "C:\victory-data\game\TEST\shaders.rpf"
```

Archive brute du jeu : 411 695 600 octets, lue directement (les conteneurs
sont décompressés dans `shaders.rpf`). Il n'y a pas de sous-commande
`shaders` : c'est `translate`, et `--warm` transforme en plus le résultat en
cache pour le run.

**Résultat :**

| | |
|---|---|
| conteneurs trouvés | 26 503 (614 doublons passés) |
| programmes traduits | **26 084 sur 26 084** (40 597 693 instructions IR) |
| structures | 26 084 (32,5 M instructions vectorielles, 791 463 gardées) |
| ressources suivies | 25 889 sur 26 084 |
| E/S élevées | 14 247 sur 14 247 (vertex et pixel) |
| DXIL émis | 14 247/14 247 vertex+pixel, **2 679/2 682 compute**, 8 765/8 765 mesh |
| géométrie | 195 programmes, 195 E/S élevées, 0 arrêté |

**Ce qui ne passe pas encore** (c'est là qu'est le travail) :

- 195 × `ES record: an LDS access of the ES half other than a record write`
  (les `VS_VoxelizeGeneric_*`, le voxelizer) ;
- 195 × `resources: an address with an untracked high half` (même famille,
  `GS_VoxelizeGeneric_*`) ;
- 3 × `emit: a lane mask holding lanes elsewhere read as data`
  (`CS_TransmissionVolumetrics*`, l'eau).

Les shaders du contenu sont donc tous lus : le jeu peut être pré-cuit en
entier (`translate --warm`) avant la première image, ce qui supprime les
traductions pendant le jeu.

### 18.1 Les textures ne sont pas dans `shaders.rpf`

Contrairement au principe proposé, `translate` ne lit **que des shaders**
(conteneurs DXBC/Xbox, fichiers SGD5). Les textures du jeu sont dans
`textures.rpf` (192 585 024 octets), `data.rpf`, `common.rpf`,
`levels.rpf`, `pack/` et `data/` : c'est le layer qui les copie à la
demande, pas cet outil. Passer `textures.rpf` à `translate` ne donnerait
rien. Le gain côté textures viendra du §16 (lâcher les copies de textures
que le titre ne relit jamais) et non d'une pré-cuisson.

## 19. Les textures : ce que la mesure a écarté (26/11, suite)

- Les copies ne naissent pas hors des slots (compteur `in_slot_make` :
  **0** sur une course entière) : chaque copie de texture existe parce
  qu'un slot la regarde.
- Les slots qui gardent une ancienne vue pendant que le titre en fait une
  neuve : 4 000 fois en trois minutes, 3 044 MB, presque tout en
  **placé** (l'ancienne vue ne tient donc pas de copie).
- Les envois sautés (25 370) sont tous de textures **placées** : le
  raccourci « le titre a écrit sa mémoire après l'hôte » ne concerne pas
  les copies, il concerne les textures que le titre remplit lui-même.

Conclusion : **toutes les copies non placées sont lues au moins une fois**
par la carte. Elles ne sont donc pas du poids mort, et le « lâcher quand
le titre ne relit pas » du §16 est faux sur cette course : elles restent
parce que leur texture reste.

### 19.1 Ce qui reste à faire, et qui est mesuré

La base d'une course de 4 min (tout par défaut) : les copies du titre
passent de 407 (245 MB) après le chargement à **9 173 (4 253 MB)** en jeu,
et la mémoire privée de 8 079 à 21 827 MB. Le vrai levier n'est pas de
lâcher (rien n'est orphelin), c'est de **ne pas garder 4,2 GB en mémoire du
PC pour des copies que la carte lit** : les mettre en mémoire de la carte
(7,7 GB libres) au-delà d'un seuil, et faire baisser ce que la copie coûte
en place.

## 20. Ce que le fork AMD (VIctory-7900XTX) a, et que Tirsos n'a pas (26/11)

Source : <https://github.com/avatarmanz/VIctory-7900XTX>, commit `9576c45`
(26/11), README + `docs/findings.md` lus, et l'historique comparé au nôtre.
Ce n'est **pas** une autre couche : c'est le même arbre (Marvisak →
fxceless → lui), donc le code se reprend par morceaux, pas à pas. Il n'y a
pas de base commune exacte : leur `main` a réécrit l'histoire (un commit
« Initial commit »), donc un `merge` est impossible ; on reprend des
**patches**, un par un.

Ce qui manque chez nous, par ordre d'importance pour une RTX 4070 :

| Levier | Chez eux | Effet annoncé |
|---|---|---|
| `VICTORY_SHOW_NEW_LATER` (par défaut ON) | `descriptor/host.rs`, 128 lignes | une texture faite pendant que des listes déjà envoyées tournent n'est montrée qu'après : **fin des pertes de device par faute de page** (~toutes les 15-16 min) |
| `VICTORY_UP_QUEUE` + `VICTORY_SEND_OWN` | `host/upq.rs` + `host/rec.rs`, 393 lignes | les envois montent sur une **file de copie** au lieu d'attendre la file graphique |
| `VICTORY_CARD_STEPS` | `host/card_steps.rs`, 1 343 lignes | les remplissages qui ont besoin de la carte se font **sur la carte** (7 ms/frame chez eux) |
| `VICTORY_THIN` | `list/thin.rs`, 913 lignes | les appels de liste sont **écrits et traduits par 8 threads** : 230 ms/frame de travail réparti |
| `VICTORY_TEXTURE_STORE=<dossier>` | `texstore.rs`, 294 lignes | les grands niveaux d'une texture sont gardés sur disque et réinjectés : tue le **pop-in** (chez eux, 8,7 s d'attente moyenne) |
| `VICTORY_CARD_BUFFERS_MIN_KB=64` | `shadow.rs` | les petits buffers (≥64 KiB) en mémoire carte — **notre essai en MB avait échoué, celui-là n'a pas été mesuré** |
| `VICTORY_FRAME_INTERVAL_MS` | `apps/victory` | notre cadence **et** les trois lectures des 30 fps du titre corrigées octet par octet (nous n'avons que `VICTORY_FRAME_US`, qui ne règle que l'intervalle demandé) |
| `VICTORY_WRITE_SITES`, `VICTORY_FILL_TITLE`, `VICTORY_SPANS_QUICK`, `VICTORY_SYNC_FINE`, `VICTORY_LANE_CORE`, `VICTORY_JOIN_BARRIERS`, `VICTORY_TEXTURE_LOOK_MS`, `VICTORY_CACHE_CORES` | divers | tous mesurés chez eux, tous absents chez nous |
| 33 pull requests fusionnées | historique | correctifs shaders **wave32 / 32 lanes** (PR #4, #66, #82, #142) — c'est **notre cas NVIDIA**, à regarder en premier |

### 20.1 Ce que la mesure dit, à nous

- Nous n'avons **aucune** perte de device dans nos courses (3 min et 4 min,
  0 `DEVICE_REMOVED`), mais nos courses sont courtes : le défaut de 15 min
  du fork AMD ne se serait pas encore montré. `SHOW_NEW_LATER` coûte peu et
  ferme la porte : **à prendre**.
- Leur `findings.md` mesure le même mur que nous : **le thread qui prépare
  une soumission** (48 ms chez eux), devant la carte (41 ms). Chez nous,
  la frame médiane dit GPU 66 ms et préparation 58 ms : même ordre, même
  remède — répartir la préparation (`THIN`, `ASYNC_SUBMIT`) et monter les
  envois sur une file de copie.
- Leur mémoire est à 35-46 GB privés, la nôtre à 21-24 GB : nous sommes
  mieux, leur `TEXTURE_STORE` n'est donc pas une urgence pour la mémoire,
  mais pour le **pop-in**.

### 20.2 Ordre proposé

1. `VICTORY_SHOW_NEW_LATER` (petit, sûr, corrige une classe entière de
   plantages).
2. `VICTORY_UP_QUEUE` + `VICTORY_SEND_OWN` (attaque le mur n°1 mesuré chez
   nous : le temps d'attente de la carte).
3. `VICTORY_CARD_BUFFERS_MIN_KB` (notre échec en MB venait de l'unité :
   1 MB de seuil ne prenait presque rien).
4. Les correctifs shaders des PR #4/#66/#82/#142 (wave32/32 lanes) : c'est
   notre GPU.
5. `THIN` et `CARD_STEPS` : gros morceaux, à faire quand 1-4 sont mesurés.

## 21. Le troisième fork : carlm125/cool_fox (le plus avancé des trois)

Source : <https://github.com/carlm125/cool_fox>, commit `f4b48ad` (26/11).
627 commits, un journal de bugs daté, et **testé sur une RTX 5090 32 GB** :
c'est le seul des trois forks qui parle de notre GPU.

### 21.1 Ce qu'il apporte que nous n'avons pas

| Levier | Où | Effet mesuré (journal du 25/11) |
|---|---|---|
| **Release `pareto-residency`** | `f4b48ad`, 1 670 lignes | 64,2 → **54,8 ms** de médiane sur la même sauvegarde (RTX 5090), 57,7 ms sur 23 intervalles lourds |
| **Baisse de la résidence** (plafonds de commit, réserves, refus) | `residency.rs`, `pool.rs`, `texture/ahead.rs` | plus de `E_OUTOFMEMORY` : 0 refus sur 120 intervalles, médiane **54,3 ms** |
| **Wave32 NVIDIA** | `victory-shader/src/resource.rs`, `emit/lower.rs` | garde `lanes.guards[b][i]` en élaguant les arêtes mortes : sans ça, un buffer de transmission reste à zéro → **carrés noirs** et blending cassé (nos 195+195 refus de shaders sont du même genre) |
| **Tables de strides** (`SLOT_TABLE_STRIDES=252`, `heap_strides`) | `descriptor/host.rs` | déformations de visage et maillages corrects |
| **`VICTORY_INDIRECT_REST=0` par défaut** | `indirect.rs` | un enregistrement indirect trop gros ne met plus à zéro **tous les dessins suivants** → le scintillement des arbres et des bâtiments au loin |
| **Libération des slots dynamiques** (`Detached::release_tables`) | `host.rs`, `draw.rs` | ne sature plus les 262 144 slots dynamiques NVIDIA (dessins perdus) |
| **`victory warm`** des pipelines | `pipeline/make.rs` | les saccades de la première minute |
| **d16** (`bufferLoad/Store d16`) | `emit/lower.rs` | ombres et visages |
| Audio XMA | `victory-hle/src/modules/audio.rs` | son |
| **256 MiB DirectStorage**, `WaitOnAddress` sur les fences | `host.rs` | chargement et attentes |

### 21.2 Ce que nous avons déjà mieux

- Notre `VICTORY_FRAME_US` = 16 667 (60 fps) marche et est mesuré (60,0 fps
  sur frames légères) ; eux n'ont que la correction du titre à 30 fps.
- Notre `VICTORY_SHOW_NEW_LATER` (porté aujourd'hui, 405 reports mesurés,
  0 perte de device) est **plus complet** que leur `SHOW_AFTER` seul.
- Notre mémoire privée : 21-24 GB, contre **34-46 GB** chez eux : notre sweep
  et notre `let_go_idle` font mieux, et c'est pour ça qu'ils ont besoin d'un
  « residency » que nous n'avons pas encore.
- Notre traducteur passe **26 084 shaders sur 26 084** ; eux en sont à
  « 25 790 sur 25 985, les autres sont des geometry shaders inutilisés ».

### 21.3 Ordre d'attaque révisé (30 fps constant, textures propres)

1. **`VICTORY_INDIRECT_REST=0`** (petit, corrige le scintillement et des
   dessins perdus).
2. **La garde `lanes.guards[b][i]`** en élaguant les arêtes mortes
   (`victory-shader/src/resource.rs`) : c'est le correctif des carrés noirs,
   et il touche nos 390 refus de shaders.
3. **`Detached::release_tables`** (slots dynamiques rendus à la fin de la
   soumission) : dessins perdus et saccades.
4. **La baisse de résidence** (`residency.rs`) : c'est ce qui tient la
   mémoire et évite les `E_OUTOFMEMORY` en session longue.
5. **`victory warm`** : pipelines d'avance.
6. `CARD_BUFFERS_MIN_KB=64`, `UP_QUEUE`, `THIN`, `CARD_STEPS` du fork AMD.

Rien n'est pris sans mesure : chaque point entre avec son interrupteur
(`VICTORY_...`) pour pouvoir comparer dans la même scène.

## 22. Ce qui est porté de cool_fox, et ce qui est mesuré (26/11, suite)

| Correctif | Commit | Mesure |
|---|---|---|
| `SHOW_NEW_LATER` (d'avatarmanz, complété) | `6db84fa` | 405 reports sur une course, **0 perte de device**, médiane 30,0 inchangée |
| `INDIRECT_REST=0` par défaut | `20864b2` | clamp compilé sans erreur, médiane 30,0 inchangée, scintillement à vérifier à l'œil |
| Garde des lanes mortes (`lanes.guards[b][i]` au site de l'accès) | `4b0daf0` | **310 tests victory-shader passent**, passage complet identique (26 084 programmes, 23 012 DXIL), course sans erreur de shader |

### 22.1 Mesuré et écarté : rendre les slots dynamiques plus tôt

Porté (`Detached::release_tables` appelé à l'atterrissage du vol), **mesuré,
puis annulé** : médiane **4,3 fps contre 26,6 sans**, les fins de course à
3,8 fps contre 4,7. Chez nous les tables sont déjà rendues par
`queue.rs`/`list::tables_done` (4 239 tables demandées, **0 refusées**), donc
le correctif de cool_fox n'apporte rien et casse des listes réutilisées.
Le code est revenu à l'état d'avant : rien n'est poussé pour ce point.

### 22.2 Ce qui reste, dans l'ordre

1. **Résidence** (`residency.rs` de cool_fox, 200 lignes, autonome) : plafond
   de commit, refus des envois sous 1,5 GB, prélecture limitée à 16-64 MB
   sous pression, et surtout **ne plus envoyer des dizaines de milliers de
   textures sur `neutral()`** quand la mémoire manque. C'est ce qui leur a
   fait gagner 64,2 → 54,3 ms sur une 5090.
2. **`victory warm`** (pipelines d'avance).
3. Les leviers du fork AMD : `CARD_BUFFERS_MIN_KB=64`, `UP_QUEUE`,
   `THIN`, `CARD_STEPS`.

## 23. Le vrai bilan : la RAM qui monte, à partir du moment où le jeu tourne (26/11)

Course de 4 min, tout par défaut, `VICTORY_INPLACE=1`, après l'intro
(l'utilisateur a raison : avant, c'est le lancement, ça ne compte pas).

| Temps depuis le début du jeu | Mémoire privée |
|---|---|
| 0 s | 7 589 MB |
| 60 s | 10 720 MB |
| 120 s | 12 107 MB |
| 180 s | 13 145 MB |
| 240 s | 14 902 MB |
| fin (300 s) | **14 307 MB** (+6 718 MB en 5 min) |

Soit **~1,3 GB par minute en jeu**, avec un ralentissement visible après
2 min (la courbe s'aplatit : le sweep commence à rattraper). C'est ce chiffre
qu'il faut faire baisser, et c'est exactement ce que la **résidence** de
cool_fox traite (leur journal : la même pression mémoire sur 5090 32 GB,
avec des `E_OUTOFMEMORY` avant, zéro après).

## 24. « Recopier cool_fox » : ce que ça veut dire exactement

Comparaison mesurée entre notre arbre et `carlm125/cool_fox` :

- nos fichiers `.rs` font **147 876 lignes**, les leurs **187 099** ;
- mais la différence **entre notre base commune et leur dernier commit**
  n'est que de **31 fichiers et 3 255 lignes ajoutées** : tout le reste de
  leur volume vient des fichiers ajoutés d'un bloc (modules séparés,
  tests) qui ne se remplacent pas comme ça.

Les 31 fichiers, par poids : `draw.rs` (+633), `descriptor/host.rs` (+446),
`host.rs` (+353), `emit/valu_tests.rs` (+334, des tests), `compose.rs`
(+325, nouveau : deux plans présentés en une image, Series S),
`residency.rs` (+201, nouveau), `texture.rs` (+148), `emit/mod.rs` (+113),
`resource.rs` (+108), `pipeline/make.rs` (+88), `pool.rs` (+86),
`queue.rs` (+84), `texture/ahead.rs` (+64), `emit/lower.rs` (+62).

Un remplacement en bloc casserait nos 23 correctifs propres (60 fps,
`SHOW_NEW_LATER`, garde des lanes, `INDIRECT_REST`, `SlotMake`, fence par
ressource, stats mémoire) : leurs fichiers ne les contiennent pas. La façon
sûre de tout récupérer est donc **fichier par fichier, avec une mesure
après chacun**, en commençant par `residency.rs` (le plus autonome et le
plus utile pour la RAM).

## 25. Peppacodes/playablegames-peppa : le fork d'un PC identique au tien (26/11)

C'est le fork **le plus proche de ta machine** : Windows 11, **RTX 5080 16 GB**
(driver 621.24), Ryzen 7 5700X3D, **32 GB de RAM**, 1080p, profil Series X.
Son objectif annoncé : 40 ms par frame (25 fps) en extérieur.

### 25.1 Ce qu'il vaut vraiment

Mesuré entre son dernier commit et cool_fox : **5 fichiers, 101 lignes**. Ce
n'est pas un arbre nouveau, c'est cool_fox plus trois petites choses
(`VICTORY_NO_WATCHDOG=1`, piles de 64 MB pour les workers de pipelines,
correctif du plan double Series S). Les 3 255 lignes qui nous manquaient
viennent du gros merge `acfd2bb`.

### 25.2 Ses réglages gagnants (`Play-VIctory.cmd`)

`VICTORY_JOIN_PARTS=8`, `VICTORY_CARD_BUFFERS_MIN_KB=64`, `VICTORY_THIN=1`,
`VICTORY_RELAY=2`, `VICTORY_RELAY_COPY=1`, `VICTORY_WIDE_BACK=1`,
`VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_JOIN_COPIES=1`,
`VICTORY_INDIRECT_WINDOW=1`, `VICTORY_INDIRECT_BATCH=2`,
`VICTORY_CARD_STEPS=2`, `VICTORY_MAP_THREADS=1`, `VICTORY_CARD_AWAKE=1`,
`VICTORY_SHOW_AFTER=1`, `VICTORY_AMBIENT=0.8`, `VICTORY_AMBIENT_AIR=1.5`,
`VICTORY_UPLOAD_AHEAD=1`, `VICTORY_LEVELS_ON_DEMAND=1`,
`VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_LOG=warn`.

Chez nous existent déjà : `RELAY`, `WIDE_BACK`, `CARD_REHASH_RUNS`,
`JOIN_COPIES`, `INDIRECT_WINDOW`, `AMBIENT`, `JOIN_PARTS`, `TEXTURE_NO_WAIT`,
`NO_WATCHDOG`, `LEVELS_ON_DEMAND`. **Manquent** (et leurs modules aussi) :
`THIN`, `CARD_STEPS`, `UPLOAD_AHEAD`, `SHOW_AFTER`, `RELAY_COPY`,
`INDIRECT_BATCH`, `MAP_THREADS`, `CARD_AWAKE`, `AMBIENT_AIR`,
`CARD_BUFFERS_MIN_KB`.

### 25.3 Mesuré chez nous

| Run | Médiane | RAM (4 min) |
|---|---|---|
| nos réglages + résidence | **30,0** | +14 357 MB |
| réglages Peppa | **3,1** | +19 691 MB |

Les réglages de Peppa **ne se transposent pas** : `VICTORY_THIN` et
`CARD_STEPS` n'existent pas chez nous, et `JOIN_PARTS=8` sur notre arbre
donne 3,1 fps. Ce n'est pas une question de valeur mais de modules : chez
eux, ces leviers sont accompagnés de 3 255 lignes qui les font marcher.

### 25.4 Porté et poussé

- **`residency.rs`** (201 lignes, tel quel) + `host::coarse_ms()` +
  `residency::refresh()` dans le sweep d'une seconde. C'est la brique qui
  fait décider à la mémoire de l'hôte ce que la prélecture, l'upload-ahead
  et le staging peuvent prendre. Médiane inchangée (30,0), 3 avis de
  pression émis. Les **points d'admission** dans `draw.rs`, `texture.rs`,
  `pool.rs`, `pipeline/make.rs` sont l'étape suivante (c'est là que la RAM
  baisse).

### 25.5 Pour un vrai « mon fork complet », ce qu'il reste à porter

72 modules absents, **37 523 lignes** : `list/thin.rs` (912),
`host/card_steps.rs` (1 334), `land.rs` (695), `compose.rs` (325),
`residency.rs` (201, fait), `pool.rs`, `host/` (13 modules),
`indirect/batch.rs`, `apps/victory/src/warm.rs`, `replay.rs`... Un
remplacement en bloc effacerait nos 24 correctifs (60 fps, `SHOW_NEW_LATER`,
garde des lanes, `INDIRECT_REST`, fence par ressource, stats mémoire). Le
plan sûr : un module à la fois, compilé et mesuré, dans cet ordre :
1. points d'admission de `residency` (RAM),
2. `pool.rs` + `texture/ahead.rs` (upload-ahead, staging borné),
3. `host/card_steps.rs` (fills sur la carte) + `CARD_BUFFERS_MIN_KB`,
4. `list/thin.rs` (enregistrement léger) — le plus gros gain CPU, le plus long.
