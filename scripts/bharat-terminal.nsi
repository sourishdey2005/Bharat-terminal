; Bharat Terminal Installer — Made by Sourish Dey
!include "MUI2.nsi"

Name "Bharat Terminal"
OutFile "BharatTerminal-v3.0.0-Setup.exe"
InstallDir "$PROGRAMFILES\Bharat Terminal"
InstallDirRegKey HKLM "Software\BharatTerminal" "InstallDir"
RequestExecutionLevel user

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\bt-app.exe"
    File "target\release\bt-cli.exe"
    File /r "assets"
    File "README.md"
    
    CreateDirectory "$SMPROGRAMS\Bharat Terminal"
    CreateShortCut "$SMPROGRAMS\Bharat Terminal\Bharat Terminal.lnk" "$INSTDIR\bt-app.exe"
    CreateShortCut "$SMPROGRAMS\Bharat Terminal\Bharat CLI.lnk" "$INSTDIR\bt-cli.exe"
    CreateShortCut "$DESKTOP\Bharat Terminal.lnk" "$INSTDIR\bt-app.exe"
    
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\BharatTerminal" "InstallDir" "$INSTDIR"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\bt-app.exe"
    Delete "$INSTDIR\bt-cli.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir /r "$INSTDIR\assets"
    Delete "$INSTDIR\README.md"
    RMDir "$INSTDIR"
    Delete "$SMPROGRAMS\Bharat Terminal\*.*"
    RMDir "$SMPROGRAMS\Bharat Terminal"
    Delete "$DESKTOP\Bharat Terminal.lnk"
    DeleteRegKey HKLM "Software\BharatTerminal"
SectionEnd
