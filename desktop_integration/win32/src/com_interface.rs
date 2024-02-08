use com::{interfaces::iunknown::IUnknown, sys::HRESULT};

use winapi::shared::minwindef::{DWORD, PUINT, UINT};
use winapi::shared::windef::HBITMAP;
use winapi::um::objidlbase::LPSTREAM;

com::interfaces! {
    #[uuid("E357FCCD-A995-4576-B01F-234630154E96")]
    pub unsafe interface IThumbnailProvider: IUnknown {
        pub fn get_thumbnail(&self, cx: UINT, phbmp: *mut HBITMAP, pdw_alpha: PUINT) -> HRESULT;
    }

    #[uuid("B824B49D-22AC-4161-AC8A-9916E8FA3F7F")]
    pub unsafe interface IInitializeWithStream: IUnknown {
        pub fn initialize(&self, pstream: LPSTREAM, grf_mode: DWORD) -> HRESULT;
    }
}
