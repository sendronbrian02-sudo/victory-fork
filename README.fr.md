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

Le titre se limite lui-même à 30 images/s : il prend l'intervalle d'image le
plus long qui tombe juste sur des multiples de 16,67 ms (60 Hz), et 30 images/s
est le plus bas de ces intervalles. Cette couche lui répond maintenant comme si
l'écran tournait à 120 Hz : l'intervalle le plus court qu'elle accepte est
**8,33 ms (120 images/s)**, quel que soit l'intervalle demandé par le titre.
Le jeu n'est donc plus bridé à 30, et il n'est pas bridé à 120 non plus : il
tourne aussi vite que la machine le permet, 120 étant le plafond.

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

Tout ce que ces commutateurs supposent est écrit là où chacun est lu ; quand
l'un change ce que le titre observe, son commentaire le dit. Si une exécution
montre un problème d'image, désactiver d'abord `VICTORY_LATE_WAIT`
(`VICTORY_LATE_WAIT=0`), puis `VICTORY_RELAY=0`, `VICTORY_ASYNC_SUBMIT=0`.

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

