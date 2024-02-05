!include "FileFunc.nsh"
!include "Library.nsh"

!define LIBRARY_X64
!define LIBRARY_SHELL_EXTENSION
!define LIBRARY_COM

!include "MUI.nsh"

; STL2Thumbnail install script
OutFile "Installer.exe"
Name STL2Thumbnail

RequestExecutionLevel admin
Unicode True

; Vars
Var guid

; Pages
!define MUI_DIRECTORYPAGE_VARIABLE $InstDir
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; ---------------------------------------------------------------------------------------------------------------
; Install section
; ---------------------------------------------------------------------------------------------------------------
Section
    SetOutPath $InstDir

    ; Install library and register with regsvr32
    !insertmacro InstallLib REGDLL NOTSHARED REBOOT_NOTPROTECTED "stl2thumbnail_win32.dll" "$InstDir\stl2thumbnail_win32.dll" "$SYSDIR"

    ; Write register keys
    ; Register as thumbnail provider for STL files
    WriteRegStr HKEY_CLASSES_ROOT ".STL\ShellEx\{E357FCCD-A995-4576-B01F-234630154E96}" "" "{$guid}"

    ; Write the uninstaller
    WriteUninstaller "$InstDir\uninstall.exe"

    DetailPrint "Important: You may have to clear your cached thumbnails for this extension to work"
SectionEnd

; ---------------------------------------------------------------------------------------------------------------
; Uninstall section
; ---------------------------------------------------------------------------------------------------------------
Section "Uninstall"
    ; Remove library
    !insertmacro UnInstallLib REGDLL NOTSHARED REBOOT_NOTPROTECTED "$INSTDIR\stl2thumbnail_win32.dll"
 
    ; Remove registry keys
    DeleteRegKey HKEY_CLASSES_ROOT ".STL\ShellEx\{E357FCCD-A995-4576-B01F-234630154E96}"

    ; Remove remaining files
    Delete $INSTDIR\uninstall.exe
    RMDir "$INSTDIR"
SectionEnd

; ---------------------------------------------------------------------------------------------------------------
; Init
; ---------------------------------------------------------------------------------------------------------------
Function .onInit
    SetRegView 64
    StrCpy $InstDir "$PROGRAMFILES64\stl2thumbnail_win32"
    StrCpy $guid "3F37FD04-2E82-4140-AD72-546484EDDABB"
FunctionEnd
