; Bharat Terminal Installer — Made by Sourish Dey
[Setup]
AppName=Bharat Terminal
AppVersion=3.0.0
AppPublisher=Sourish Dey
AppPublisherURL=https://github.com/sourishdey/bharat-terminal
DefaultDirName={autopf}\Bharat Terminal
DefaultGroupName=Bharat Terminal
OutputBaseFilename=BharatTerminal-v3.0.0-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
SetupIconFile=assets\icons\icon.ico
UninstallDisplayIcon={app}\bt-app.exe

[Files]
Source: "target\release\bt-app.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\release\bt-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: recursesubdirs
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Bharat Terminal"; Filename: "{app}\bt-app.exe"
Name: "{group}\Bharat CLI"; Filename: "{app}\bt-cli.exe"
Name: "{autodesktop}\Bharat Terminal"; Filename: "{app}\bt-app.exe"
Name: "{group}\Uninstall"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\bt-app.exe"; Description: "Launch Bharat Terminal"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\BharatTerminal"
