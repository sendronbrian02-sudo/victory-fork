@echo off
setlocal DisableDelayedExpansion
pushd "%~dp0." || exit /b 1

set "mode=%~1"
if /i "%mode%"=="build" goto build
if /i "%mode%"=="help" goto help
if /i "%mode%"=="check" goto setup
if /i "%mode%"=="play" goto setup
if /i "%mode%"=="warm" goto setup
if /i "%mode%"=="shaders" goto setup
if /i "%mode%"=="bench" goto setup
if /i "%mode%"=="capture" goto setup
goto usage_error

:build
where cargo.exe >nul 2>&1
if errorlevel 1 goto missing_cargo
if "%~2"=="" goto build_release
if /i "%~2"=="diag" goto build_diag
goto usage_error

:build_release
cargo build --release --locked -p victory
goto done

:build_diag
cargo build --profile diag --locked -p victory --features diagnostics
goto done

:setup
if /i not "%~x2"==".exe" goto bad_game
if not exist "%~f2" goto bad_game
if exist "%~f2\" goto bad_game
set "game=%~f2"
set "VICTORY_GAME_DIR=%~dp2"
set "console=%~3"
if not defined console set "console=series-x"
if /i "%console%"=="series-x" set "console=series-x"
if /i "%console%"=="series-s" set "console=series-s"
if not "%console%"=="series-x" if not "%console%"=="series-s" goto usage_error
set "launcher=target\release\victory.exe"
if /i "%mode%"=="capture" set "launcher=target\diag\victory.exe"
if not exist "%launcher%" goto missing_launcher
if /i "%mode%"=="check" goto check
if /i "%mode%"=="warm" goto warm
if /i "%mode%"=="shaders" goto shaders
if /i "%mode%"=="bench" goto bench
if /i "%mode%"=="capture" goto capture
goto play

:check
"%launcher%" config
if errorlevel 1 goto done
"%launcher%" run "%game%" --console %console%
goto done

:play
"%launcher%" run "%game%" --execute --console %console%
goto done

:warm
if not defined VICTORY_WARM_SECONDS set "VICTORY_WARM_SECONDS=300"
"%launcher%" translate "%VICTORY_GAME_DIR%." --detect-host --warm all
goto done

:shaders
if not defined LOCALAPPDATA goto missing_localappdata
"%launcher%" translate "%VICTORY_GAME_DIR%." --detect-host --export-dxil "%LOCALAPPDATA%\victory\exports\shaders"
goto done

:bench
if not defined LOCALAPPDATA goto missing_localappdata
set "logdir=%LOCALAPPDATA%\victory\bench"
if not exist "%logdir%\" mkdir "%logdir%"
if not exist "%logdir%\" goto log_error
for %%V in (VICTORY_CAPTURE VICTORY_CAPTURE_WORLD VICTORY_AUDIO_CAPTURE VICTORY_XMA_DUMP VICTORY_TEXTURE_DUMP VICTORY_GPU_TIMING) do set "%%V="
set "VICTORY_FRAME_LOG=1"
set "VICTORY_CARD_METER=0"
set "VICTORY_LOG=info"
echo Journal : "%logdir%\fork.log"
"%launcher%" run "%game%" --execute --console %console% >"%logdir%\fork.log" 2>&1
goto done

:capture
if not defined LOCALAPPDATA goto missing_localappdata
set "VICTORY_TEXTURE_DUMP=%LOCALAPPDATA%\victory\exports\textures"
set "VICTORY_AUDIO_CAPTURE=%LOCALAPPDATA%\victory\exports\jeu-%RANDOM%-%RANDOM%.wav"
echo Textures : "%VICTORY_TEXTURE_DUMP%"
echo Audio : "%VICTORY_AUDIO_CAPTURE%"
"%launcher%" run "%game%" --execute --console %console%
goto done

:bad_game
echo Indique le chemin de l'executable .exe de ta copie Xbox. 1>&2
goto error

:missing_cargo
echo Cargo introuvable. Installe Rust avec rustup, puis ouvre un nouveau CMD. 1>&2
goto error

:missing_launcher
echo Lanceur absent : "%launcher%". Utilise victory.cmd build ou victory.cmd build diag. 1>&2
goto error

:missing_localappdata
echo LOCALAPPDATA est indisponible. 1>&2
goto error

:log_error
echo Impossible de creer "%logdir%". 1>&2
goto error

:usage_error
call :usage
goto error

:help
call :usage
popd
exit /b 0

:usage
echo victory.cmd build [diag]
echo victory.cmd check^|play^|warm^|shaders^|bench^|capture "D:\MonDump\MonJeu.exe" [series-x^|series-s]
exit /b 0

:error
popd
exit /b 1

:done
set "result=%errorlevel%"
popd
exit /b %result%
