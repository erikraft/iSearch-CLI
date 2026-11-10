; iSearch CLI™ NSIS Installer Script
; Official Author: ErikrafT
; Copyright © 2026 ErikrafT

!include "MUI2.nsh"

Name "iSearch CLI™"
OutFile "..\dist\isearch-installer-x86_64.exe"
InstallDir "$PROGRAMFILES\iSearch CLI"
InstallDirRegKey HKLM "Software\iSearch CLI" "Install_Dir"

RequestExecutionLevel admin

; Interface Settings
!define MUI_ABORTWARNING
!define MUI_ICON "icons\windows-cmd.ico"
!define MUI_UNICON "icons\windows-cmd.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

Section "Install" SecInstall
    SetOutPath "$INSTDIR"
    File "..\target\x86_64-pc-windows-msvc\release\isearch-cli.exe"
    File "icons\windows-cmd.ico"

    ; Write uninstall information to Registry
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "DisplayName" "iSearch CLI™"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "Publisher" "ErikrafT"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "DisplayVersion" "2.1.0"

    CreateDirectory "$SMPROGRAMS\iSearch CLI"
    CreateShortCut "$SMPROGRAMS\iSearch CLI\iSearch CLI.lnk" "$INSTDIR\isearch-cli.exe" "" "$INSTDIR\windows-cmd.ico"
    CreateShortCut "$DESKTOP\iSearch CLI.lnk" "$INSTDIR\isearch-cli.exe" "" "$INSTDIR\windows-cmd.ico"
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\isearch-cli.exe"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$INSTDIR\windows-cmd.ico"
    Delete "$SMPROGRAMS\iSearch CLI\iSearch CLI.lnk"
    RMDir "$SMPROGRAMS\iSearch CLI"
    Delete "$DESKTOP\iSearch CLI.lnk"
    RMDir "$INSTDIR"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI"
    DeleteRegKey HKLM "Software\iSearch CLI"
SectionEnd
