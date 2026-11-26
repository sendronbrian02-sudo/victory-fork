<!-- coderabbit-ai-generated:v1 -->

# Compiler, exporter et mesurer sur Windows

VIctory compile la couche de compatibilité. Il lance l'exécutable du jeu que tu fournis, charge ses ressources depuis son dossier et traduit les shaders pour Direct3D 12. Garde le dossier complet du jeu, les caches et les exports en dehors de tous les dossiers source VIctory.

## Compiler et lancer

Prérequis : Windows 10/11 x64, processeur AVX2, carte Direct3D 12 avec Shader Model 6.6 et mesh shaders, Rust via rustup et les outils MSVC C++ x64 avec le SDK Windows. Le dépôt sélectionne la version de Rust.

Depuis PowerShell :

```powershell
Set-Location 'C:\Users\sendr\Desktop\victory-fork'
rustup toolchain install
cargo build --release --locked -p victory
cargo build --profile diag --locked -p victory --features diagnostics

$jeu = 'D:\MonDump\MonJeu.exe' # remplace par le chemin de ton exécutable Xbox
$env:VICTORY_GAME_DIR = Split-Path -LiteralPath $jeu
& .\target\release\victory.exe config
& .\target\release\victory.exe run $jeu --execute --console series-x
```

## Exporter sur le PC

Les commandes suivantes utilisent les fichiers que tu fournis et écrivent sous `%LOCALAPPDATA%\victory\exports`.

```powershell
$exports = Join-Path $env:LOCALAPPDATA 'victory\exports'
$shaders = Join-Path $exports 'shaders'
$env:VICTORY_HOST_WAVE = '32' # pour reproduire la largeur de vague d'une carte NVIDIA
& .\target\release\victory.exe translate $env:VICTORY_GAME_DIR --export-dxil $shaders
Remove-Item Env:VICTORY_HOST_WAVE

$env:VICTORY_TEXTURE_DUMP = Join-Path $exports 'textures'
$env:VICTORY_AUDIO_CAPTURE = Join-Path $exports 'jeu.wav'
& .\target\diag\victory.exe run $jeu --execute --console series-x
Remove-Item Env:VICTORY_TEXTURE_DUMP, Env:VICTORY_AUDIO_CAPTURE
```

- **Shaders :** les shaders que le scanner trouve et que VIctory arrive à traduire sont exportés en DXIL. Le hash dans leur nom identifie le contenu. Le résumé indique les échecs de traduction. Ajoute `--validate` pour exporter seulement les shaders acceptés par le validateur DXIL du SDK Windows.
- **Textures :** la version diagnostic enregistre des captures DDS de textures 2D couleur et BC détilées pendant la partie. Un fichier contient un niveau ; le niveau 0 est la résolution maximale. Les nouvelles captures sont limitées à 256 fichiers par lancement et à 16 Mio par fichier. Les tableaux, cubes, volumes, formats vidéo, textures générées uniquement côté GPU et textures que le jeu ne charge pas pendant la partie restent hors de cette capture. Règle `VICTORY_TEXTURE_DUMP_AFTER_FRAME` pour commencer plus tard si les menus remplissent le quota. Le seuil s'applique aux textures détilées à partir de cette image.
- **Audio :** le WAV contient le mix joué par le processus. Quitte normalement pour que l'enregistreur finalise le fichier. Change le nom entre deux captures à conserver.

Cette procédure ne reconstruit pas un inventaire complet des archives du jeu et ne conserve pas leurs noms de fichiers d'origine.

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:be1eab5d24888ca419e1c41fe4efb46570ff9d926800923555f98063b8916f92 -->

## Changements du rendu

Les liaisons de cibles couleur et profondeur sont réutilisées lorsque la description de chaque vue et sa texture hôte sont identiques. Le cache est remis à zéro au changement de liste de commandes. `VICTORY_BIND_CACHE=0` permet une comparaison à réglages identiques. Ce changement adapte [ca1d519 de cool_fox](https://github.com/carlm125/cool_fox/commit/ca1d519f18ccece3daf2b05191a43fae17ac6f91).

Le bornage hors liste (`VICTORY_HOIST_CLAMPS`) a été retiré. Il réutilisait le même emplacement pour plusieurs commandes différées et pouvait lire les arguments GPU avant leur écriture. Les bornages se font dans la liste qui consomme les arguments. Les fenêtres de bornage existantes restent disponibles.

## Mesurer les 30 FPS dans le monde

<!-- /coderabbit-ai-generated:v1 -->

Fais chauffer les caches pour chaque version. Repars de la même sauvegarde et joue au moins une minute sur le même trajet, avec les mêmes réglages de console, de résolution et d'éclairage. Teste au moins une scène chargée. Coupe les exports et les captures pendant la mesure.

```powershell
$logDir = Join-Path $env:LOCALAPPDATA 'victory\bench'
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$env:VICTORY_FRAME_LOG = '1'
& .\target\release\victory.exe run $jeu --execute --console series-x 2>&1 |
    Tee-Object -FilePath (Join-Path $logDir 'fork.log')
Remove-Item Env:VICTORY_FRAME_LOG
```

Dans les lignes `frame times:`, compare le **p99** (temps en dessous duquel passent environ 99 % des images) et le compteur `over 33.33 ms`. Pour des 30 FPS stables, il faut que la scène respecte ce budget sur la durée. Les menus et les premières compilations de pipelines se mesurent séparément. Un plafond à 30 FPS ne réduit pas le temps de rendu d'une image.

Les correctifs de suivi des shaders et de reprise XMA reprennent des changements de [cool_fox, f4b48ad](https://github.com/carlm125/cool_fox/commit/f4b48ad766ddbec1a1867022cd42f7645b0dd55e) et [8cc31ab](https://github.com/carlm125/cool_fox/commit/8cc31ab37afcae231e46696042342bb5af2f0f2a). La conservation des objets D3D12 par liste s'appuie sur [ab86ef7](https://github.com/carlm125/cool_fox/commit/ab86ef7f3ffba5cb7e8e21009f28f437222087da) et garde le suivi des temps par pipeline de ce fork. La [mesure RTX 5090 de cool_fox](https://github.com/carlm125/cool_fox/blob/f4b48ad766ddbec1a1867022cd42f7645b0dd55e/docs/bug-journal.md) rapportait environ 54 ms par image sur l'ouverture : elle ne démontre pas 30 FPS dans le jeu.

Référence format : [guide DDS de Microsoft](https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide).
