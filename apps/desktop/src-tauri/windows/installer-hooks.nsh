; ClipMill's additions to Tauri's NSIS installer (bundle > windows > nsis >
; installerHooks in tauri.release.conf.json).
;
; ClipMill's components need Microsoft's Visual C++ runtime. ONNX Runtime (in
; vad, align, speakers and faces) links msvcp140.dll and msvcp140_1.dll, which
; no wheel bundles, and it and the pinned Python were built with toolset 14.44,
; the oldest runtime they can rely on. When Windows records no x64 runtime of
; 14.44 or newer, the installer runs Microsoft's own redistributable, pinned in
; bom.toml and staged beside this file by tools/release/stage.py. That asks
; Windows for permission itself; if it is refused, ClipMill is still installed
; and the installer says what is missing. The daemon needs no runtime: it links
; its own in.

; Tauri includes this file by its absolute path, so this is its folder.
!define CLIPMILL_HOOKS_DIR "${__FILEDIR__}"
!define CLIPMILL_VC_KEY "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"
!define CLIPMILL_VC_MINOR 44
!define CLIPMILL_VC_SETUP "$TEMP\clipmill_vc_redist.x64.exe"

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
