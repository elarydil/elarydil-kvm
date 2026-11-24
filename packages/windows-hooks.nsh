; Stop the app-owned agent before overwriting its executable during upgrades.
!macro ELARYDIL_STOP_AGENT
  IfFileExists "$INSTDIR\bin\elarydil-kvm.exe" 0 elarydil_agent_done
  nsExec::ExecToStack /TIMEOUT=35000 '"$INSTDIR\bin\elarydil-kvm.exe" agent stop --wait'
  Pop $0
  Pop $1
  StrCmp $0 "2" 0 elarydil_agent_done
  ; Older CLI builds did not expose --wait; they also had no timed DDC tests.
  nsExec::ExecToStack /TIMEOUT=10000 '"$INSTDIR\bin\elarydil-kvm.exe" agent stop'
  Pop $0
  Pop $1
  Sleep 500
elarydil_agent_done:
!macroend
!macro NSIS_HOOK_PREINSTALL
  !insertmacro ELARYDIL_STOP_AGENT
!macroend
!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro ELARYDIL_STOP_AGENT
!macroend
