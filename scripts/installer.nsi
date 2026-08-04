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

    ; Write uninstall information to Registry
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "DisplayName" "iSearch CLI™"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "Publisher" "ErikrafT"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI" "DisplayVersion" "0.1.0"

    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\isearch-cli.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\iSearch CLI"
    DeleteRegKey HKLM "Software\iSearch CLI"
SectionEnd
