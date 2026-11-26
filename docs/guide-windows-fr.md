<!-- coderabbit-ai-generated:v1 -->

# Compiler, exporter et mesurer sur Windows

VIctory compile la couche de compatibilité. Il lance l'exécutable du jeu que tu fournis, charge ses ressources depuis son dossier et traduit les shaders pour Direct3D 12. Garde le dossier complet du jeu, les caches et les exports en dehors de tous les dossiers source VIctory.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:b7726afd42feb1d78e762f78758a9ba569073b0d47de709d47cb8731635c7a7c -->

## Préparer et lancer depuis CMD

Ouvre **Invite de commandes** (`cmd.exe`). Avec le ZIP Windows, place-toi dans le dossier extrait qui contient `victory.cmd` ; les deux exécutables sont déjà compilés. Avec un clone Git, installe Rust via rustup et les outils MSVC C++ x64 avec le SDK Windows, puis compile les exécutables dans ce dossier :

```cmd
cd /d "C:\Users\sendr\Desktop\victory-fork"
rustup toolchain install
victory.cmd build
victory.cmd build diag
```

Remplace le chemin ci-dessous par **l'exécutable Xbox** situé dans ton dossier complet du jeu, à côté de ses données. Dans le même CMD :

```cmd
set "JEU=D:\MonDump\MonJeu.exe"
victory.cmd check "%JEU%"
victory.cmd warm "%JEU%"
victory.cmd play "%JEU%"
```

`check` prépare l'exécutable et affiche ses imports sans le lancer. `warm` est facultatif : il détecte les réglages D3D12 du PC et prépare des shaders dans le cache local. Il cesse de parcourir les fichiers au bout de cinq minutes par défaut ; une lecture ou traduction déjà commencée peut finir après le délai. Pour modifier le délai : `set "VICTORY_WARM_SECONDS=600"`. Le jeu traduit encore ce que le scanner n'a pas trouvé et son pilote prépare des pipelines lors des premiers lancements.

`play` utilise Series X par défaut. Pour un essai séparé en Series S, lance `victory.cmd play "%JEU%" series-s`. Garde le même mode et les mêmes réglages quand tu compares les performances.

<!-- /coderabbit-ai-generated:v1 -->

## Exporter sur le PC

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:73f784f8484b8e7aa3b7bdcfd310f872e3b4e88b4fabd7da3913e0e462b9c45e -->

Les commandes suivantes utilisent les fichiers que tu fournis et écrivent sous `%LOCALAPPDATA%\victory\exports`. `capture` demande la version diagnostic.

```cmd
victory.cmd shaders "%JEU%"
victory.cmd capture "%JEU%"
```

<!-- /coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:632d86059148a7c93988be09563972e03561a39216a43a02a5533b359b1115f1 -->

- **Shaders :** les shaders que le scanner trouve et que VIctory arrive à traduire sont exportés en DXIL. Le hash dans leur nom identifie le contenu. Le résumé indique les échecs de traduction. Le validateur du SDK Windows est disponible en appelant directement `target\release\victory.exe translate "D:\MonDump" --detect-host --validate --export-dxil "%LOCALAPPDATA%\victory\exports\shaders"`.
- **Textures :** la version diagnostic enregistre des captures DDS de textures 2D couleur et BC détilées pendant la partie. Un fichier contient un niveau ; le niveau 0 est la résolution maximale. Les nouvelles captures sont limitées à 256 fichiers par lancement et à 16 Mio par fichier. Les tableaux, cubes, volumes, formats vidéo, textures générées uniquement côté GPU et textures que le jeu ne charge pas pendant la partie restent hors de cette capture. Règle `VICTORY_TEXTURE_DUMP_AFTER_FRAME` pour commencer plus tard si les menus remplissent le quota. Le seuil s'applique aux textures détilées à partir de cette image.
- **Audio :** le WAV contient le mix joué par le processus. Quitte normalement pour que l'enregistreur finalise le fichier. Chaque lancement de `capture` choisit un nouveau nom de fichier WAV.

<!-- /coderabbit-ai-generated:v1 -->

Cette procédure ne reconstruit pas un inventaire complet des archives du jeu et ne conserve pas leurs noms de fichiers d'origine.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:be1eab5d24888ca419e1c41fe4efb46570ff9d926800923555f98063b8916f92 -->

## Changements du rendu

Les liaisons de cibles couleur et profondeur sont réutilisées lorsque la description de chaque vue et sa texture hôte sont identiques. Le cache est remis à zéro au changement de liste de commandes. `VICTORY_BIND_CACHE=0` permet une comparaison à réglages identiques. Ce changement adapte [ca1d519 de cool_fox](https://github.com/carlm125/cool_fox/commit/ca1d519f18ccece3daf2b05191a43fae17ac6f91).

Le bornage hors liste (`VICTORY_HOIST_CLAMPS`) a été retiré. Il réutilisait le même emplacement pour plusieurs commandes différées et pouvait lire les arguments GPU avant leur écriture. Les bornages se font dans la liste qui consomme les arguments. Les fenêtres de bornage existantes restent disponibles.

## Mesurer les 30 FPS dans le monde

<!-- /coderabbit-ai-generated:v1 -->

Fais chauffer les caches pour chaque version. Repars de la même sauvegarde et joue au moins une minute sur le même trajet, avec les mêmes réglages de console, de résolution et d'éclairage. Teste au moins une scène chargée. Coupe les exports et les captures pendant la mesure.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:36a98fbf236d3db59d2b4ea5e8254b3229b7ff46ca26fd248345dc22ce4b6434 -->

```cmd
victory.cmd bench "%JEU%"
findstr /c:"frame times:" "%LOCALAPPDATA%\victory\bench\fork.log"
```

Le lanceur écrit la sortie dans `fork.log` et remplace ce fichier à chaque mesure. Renomme-le si tu veux comparer deux exécutions. Joue dans le prologue avec les captures coupées ; le mode `bench` les désactive pour cette exécution.

<!-- /coderabbit-ai-generated:v1 -->

Dans les lignes `frame times:`, compare le **p99** (temps en dessous duquel passent environ 99 % des images) et le compteur `over 33.33 ms`. Pour des 30 FPS stables, il faut que la scène respecte ce budget sur la durée. Les menus et les premières compilations de pipelines se mesurent séparément. Un plafond à 30 FPS ne réduit pas le temps de rendu d'une image.

Les correctifs de suivi des shaders et de reprise XMA reprennent des changements de [cool_fox, f4b48ad](https://github.com/carlm125/cool_fox/commit/f4b48ad766ddbec1a1867022cd42f7645b0dd55e) et [8cc31ab](https://github.com/carlm125/cool_fox/commit/8cc31ab37afcae231e46696042342bb5af2f0f2a). La conservation des objets D3D12 par liste s'appuie sur [ab86ef7](https://github.com/carlm125/cool_fox/commit/ab86ef7f3ffba5cb7e8e21009f28f437222087da) et garde le suivi des temps par pipeline de ce fork. La [mesure RTX 5090 de cool_fox](https://github.com/carlm125/cool_fox/blob/f4b48ad766ddbec1a1867022cd42f7645b0dd55e/docs/bug-journal.md) rapportait environ 54 ms par image sur l'ouverture : elle ne démontre pas 30 FPS dans le jeu.

Référence format : [guide DDS de Microsoft](https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide).
