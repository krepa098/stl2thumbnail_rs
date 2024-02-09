#![cfg(windows)]

mod com_interface;
mod generator;
mod generator_impl;

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

static mut _HMODULE: *mut ::core::ffi::c_void = ::core::ptr::null_mut();

#[no_mangle]
unsafe extern "system" fn DllMain(
    hinstance: *mut ::core::ffi::c_void,
    fdw_reason: u32,
    _reserved: *mut ::core::ffi::c_void,
) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if fdw_reason == DLL_PROCESS_ATTACH {
        unsafe {
            _HMODULE = hinstance;
        }
    }
    1
}
#[no_mangle]
unsafe extern "system" fn DllGetClassObject(
    class_id: *const ::com::sys::CLSID,
    iid: *const ::com::sys::IID,
    result: *mut *mut ::core::ffi::c_void,
) -> ::com::sys::HRESULT {
    assert!(
        !class_id.is_null(),
        "class id passed to DllGetClassObject should never be null"
    );
    let class_id = unsafe { &*class_id };
    if class_id == &CLSID_GENERATOR_CLASS_STL {
        let instance = <WinSTLThumbnailGenerator as ::com::production::Class>::Factory::allocate();
        instance.QueryInterface(&*iid, result)
    } else if class_id == &CLSID_GENERATOR_CLASS_GCODE {
        let instance = <WinGCodehumbnailGenerator as ::com::production::Class>::Factory::allocate();
        instance.QueryInterface(&*iid, result)
    } else {
        ::com::sys::CLASS_E_CLASSNOTAVAILABLE
    }
}
#[no_mangle]
extern "system" fn DllRegisterServer() -> ::com::sys::HRESULT {
    ::com::production::registration::dll_register_server(&mut get_relevant_registry_keys())
}
#[no_mangle]
extern "system" fn DllUnregisterServer() -> ::com::sys::HRESULT {
    ::com::production::registration::dll_unregister_server(&mut get_relevant_registry_keys())
}
fn get_relevant_registry_keys() -> Vec<::com::production::registration::RegistryKeyInfo> {
    use com::production::registration::RegistryKeyInfo;
    let file_path = unsafe { ::com::production::registration::get_dll_file_path(_HMODULE) };
    vec![
        RegistryKeyInfo::new(
            &::com::production::registration::class_key_path(CLSID_GENERATOR_CLASS_STL),
            "",
            stringify!(WinSTLThumbnailGenerator),
        ),
        RegistryKeyInfo::new(
            &::com::production::registration::class_inproc_key_path(CLSID_GENERATOR_CLASS_STL),
            "",
            &file_path,
        ),
        RegistryKeyInfo::new(
            &::com::production::registration::class_key_path(CLSID_GENERATOR_CLASS_GCODE),
            "",
            stringify!(WinGCodehumbnailGenerator),
        ),
        RegistryKeyInfo::new(
            &::com::production::registration::class_inproc_key_path(CLSID_GENERATOR_CLASS_GCODE),
            "",
            &file_path,
        ),
    ]
}
