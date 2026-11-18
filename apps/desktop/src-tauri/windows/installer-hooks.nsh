; ClipMill's additions to Tauri's NSIS installer (bundle > windows > nsis >
; installerHooks in tauri.release.conf.json).
;
; ClipMill's components need Microsoft's Visual C++ runtime. ONNX Runtime (in
; vad, align, speakers and faces) links msvcp140.dll and msvcp140_1.dll, which
; no wheel bundles. It and the pinned Python were built with toolset 14.44, and
; llama.cpp's Vulkan backend, which runs the editorial model, with 14.51; a
; library needs a runtime at least as new as its toolset. When Windows records
; no x64 runtime of 14.51 or newer, the installer runs Microsoft's own
; redistributable, pinned in bom.toml and staged beside this file by
; tools/release/stage.py. That asks Windows for permission itself; if it is
; refused, ClipMill is still installed and the installer says what is missing.
; The daemon needs no runtime: it links its own in.

; Tauri includes this file by its absolute path, so this is its folder.
!define CLIPMILL_HOOKS_DIR "${__FILEDIR__}"
!define CLIPMILL_VC_KEY "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"
!define CLIPMILL_VC_MINOR 51
!define CLIPMILL_VC_SETUP "$TEMP\clipmill_vc_redist.x64.exe"

; Before 0.5, clipmill.exe was Studio. Test only that installation's file:
; another directory may contain a running CLI with the same name.
!macro NSIS_HOOK_PREINSTALL
  Push $0
  ReadRegStr $0 SHCTX "${UNINSTKEY}" "MainBinaryName"
  ${If} $0 == "clipmill.exe"
  ${AndIf} ${FileExists} "$INSTDIR\clipmill.exe"
    clipmill_wait_for_old_studio:
    ; GENERIC_WRITE, share read/write/delete, OPEN_EXISTING. A running image
    ; cannot be opened for writing. This check does not modify the executable.
    System::Call 'kernel32::CreateFileW(w "$INSTDIR\clipmill.exe", i 0x40000000, i 7, p 0, i 3, i 0x80, p 0) p.r0'
    ${If} $0 P= -1
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Close ClipMill before upgrading, then choose Retry." /SD IDCANCEL IDRETRY clipmill_wait_for_old_studio
      Pop $0
      SetErrorLevel 2
      Abort "The previous ClipMill executable is still in use or cannot be replaced."
    ${EndIf}
    System::Call 'kernel32::CloseHandle(p r0)'
  ${EndIf}
  Pop $0
!macroend

; Sets $R0 to 1 when this registry view records a new enough runtime.
!macro CLIPMILL_VC_RUNTIME_IN VIEW
  SetRegView ${VIEW}
  ReadRegDWORD $R1 HKLM "${CLIPMILL_VC_KEY}" "Installed"
  ReadRegDWORD $R2 HKLM "${CLIPMILL_VC_KEY}" "Major"
  ReadRegDWORD $R3 HKLM "${CLIPMILL_VC_KEY}" "Minor"
  ${If} $R1 = 1
    ${If} $R2 > 14
      StrCpy $R0 1
    ${ElseIf} $R2 = 14
    ${AndIf} $R3 >= ${CLIPMILL_VC_MINOR}
      StrCpy $R0 1
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Tauri copies the sidecars, removes the old registered main binary and
  ; retargets its shortcuts. Here the removed name belongs to the new CLI.
  ${If} $OldMainBinaryName == "clipmill.exe"
    File /a "/oname=$INSTDIR\clipmill.exe" "${CLIPMILL_HOOKS_DIR}\..\binaries\clipmill-x86_64-pc-windows-msvc.exe"
    ; The interactive finish page can skip the desktop shortcut. Retarget an
    ; existing legacy link even then, honoring the absence of a shortcut.
    Push $0
    Push $1
    Push $2
    Push $3
    !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
    Pop $0
    ${If} $0 = 1
      !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    ${EndIf}
    Pop $3
    Pop $2
    Pop $1
    Pop $0
  ${EndIf}

  Push $R0
  Push $R1
  Push $R2
  Push $R3
  StrCpy $R0 0
  !insertmacro CLIPMILL_VC_RUNTIME_IN 64
  !insertmacro CLIPMILL_VC_RUNTIME_IN 32
  SetRegView default
  ${If} $R0 <> 1
    DetailPrint "Installing the Microsoft Visual C++ runtime ClipMill's components need"
    Delete "${CLIPMILL_VC_SETUP}"
    File "/oname=${CLIPMILL_VC_SETUP}" "${CLIPMILL_HOOKS_DIR}\vc_redist.x64.exe"
    ClearErrors
    ${If} ${Silent}
      ExecWait '"${CLIPMILL_VC_SETUP}" /install /quiet /norestart' $R1
    ${Else}
      ExecWait '"${CLIPMILL_VC_SETUP}" /install /passive /norestart' $R1
    ${EndIf}
    ${IfThen} ${Errors} ${|} StrCpy $R1 -1 ${|}
    Delete "${CLIPMILL_VC_SETUP}"
    ; 0: installed. 3010: installed, completed at the next restart.
    ; 1638: a newer runtime is already there.
    ${If} $R1 = 0
    ${OrIf} $R1 = 3010
    ${OrIf} $R1 = 1638
      DetailPrint "The Microsoft Visual C++ runtime is installed"
    ${Else}
      DetailPrint "The Microsoft Visual C++ runtime was not installed (code $R1)"
      MessageBox MB_ICONEXCLAMATION|MB_OK "ClipMill is installed, but Microsoft's Visual C++ runtime is not (code $R1), and ClipMill's components cannot run without it. Install it from https://aka.ms/vc14/vc_redist.x64.exe" /SD IDOK
    ${EndIf}
  ${EndIf}
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
!macroend
