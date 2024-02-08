#![cfg(windows)]

mod com_interface;
mod generator;

use com::sys::IID;
use generator::*;

/*

1.
Create key 'Computer\HKEY_CLASSES_ROOT\.STL\ShellEx\{E357FCCD-A995-4576-B01F-234630154E96}'
Note: {E357FCCD-A995-4576-B01F-234630154E96} stands for an IThumbnailProvider shell extension

2.
Set the default key to the GUID of this DLL
(Default) = {3F37FD04-2E82-4140-AD72-546484EDDABB}

3.
Register the DLL with '%windir%\System32\regsvr32.exe <path_to_dll>'

4.
Check Computer\HKEY_LOCAL_MACHINE\SOFTWARE\Classes\CLSID\{3F37FD04-2E82-4140-AD72-546484EDDABB}
pointing to the correct path

5.
Add GUID to approved shell extensions in
Computer\HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Approved


Refs:
    * http://www.benryves.com/?mode=filtered&single_post=3189294
    * https://stackoverflow.com/questions/4897685/how-do-i-register-a-dll-file-on-windows-7-64-bit
    * https://mtaulty.com/2006/07/21/m_5884/
*/

// GUID: 3F37FD04-2E82-4140-AD72-546484EDDABB
pub const CLSID_GENERATOR_CLASS_STL: IID = IID {
    data1: 0x3F37FD04,
    data2: 0x2E82,
    data3: 0x4140,
    data4: [0xAD, 0x72, 0x54, 0x64, 0x84, 0xED, 0xDA, 0xBB],
};

// GUID: 3F37FD04-2E82-4140-AD72-546484EDDABC
pub const CLSID_GENERATOR_CLASS_GCODE: IID = IID {
    data1: 0x3F37FD04,
    data2: 0x2E82,
    data3: 0x4140,
    data4: [0xAD, 0x72, 0x54, 0x64, 0x84, 0xED, 0xDA, 0xBC],
};

com::inproc_dll_module![
    (CLSID_GENERATOR_CLASS_STL, WinSTLThumbnailGenerator),
    (CLSID_GENERATOR_CLASS_GCODE, WinGCodehumbnailGenerator)
];
