use crate::com_interface::{IInitializeWithStream, IThumbnailProvider};

use com::sys::HRESULT;

use winapi::shared::minwindef::{DWORD, PUINT, UINT};
use winapi::shared::windef::HBITMAP;
use winapi::um::objidlbase::LPSTREAM;

use std::cell::RefCell;

com::class! {
    pub class WinSTLThumbnailGenerator: IThumbnailProvider, IInitializeWithStream {
        data: RefCell<Vec<u8>>, // will be initialized to Default::default()
    }

    impl IThumbnailProvider for WinSTLThumbnailGenerator {
        unsafe fn get_thumbnail(
            &self,
            cx: UINT,            // size in x & y dimension
            phbmp: *mut HBITMAP, // data ptr
            pdw_alpha: PUINT,
        ) -> com::sys::HRESULT {
            super::generator_impl::stl_impl::get_thumbnail(&self.data, cx, phbmp, pdw_alpha)
        }
    }


    impl IInitializeWithStream for WinSTLThumbnailGenerator {
        unsafe fn initialize(&self, pstream: LPSTREAM, _grf_mode: DWORD) -> HRESULT {
            super::generator_impl::read_all_from_stream(&self.data, pstream, _grf_mode)
        }
    }
} // class

com::class! {
    pub class WinGCodehumbnailGenerator: IThumbnailProvider, IInitializeWithStream {
        data: RefCell<Vec<u8>>, // will be initialized to Default::default()
    }

    impl IThumbnailProvider for WinGCodehumbnailGenerator {
        unsafe fn get_thumbnail(
            &self,
            cx: UINT,            // size in x & y dimension
            phbmp: *mut HBITMAP, // data ptr
            pdw_alpha: PUINT,
        ) -> com::sys::HRESULT {
            super::generator_impl::gcode_impl::get_thumbnail(&self.data, cx, phbmp, pdw_alpha)
        }
    }

    impl IInitializeWithStream for WinGCodehumbnailGenerator {
        unsafe fn initialize(&self, pstream: LPSTREAM, _grf_mode: DWORD) -> HRESULT {
            super::generator_impl::read_all_from_stream(&self.data, pstream, _grf_mode)
        }
    }
} // class

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn com() {
        let instance = WinSTLThumbnailGenerator::allocate(Default::default());
        let ithumbnail_handle = instance.query_interface::<IThumbnailProvider>();

        assert!(ithumbnail_handle.is_some());
    }

    #[test]
    fn com_create_thumbnail() {
        let instance = WinSTLThumbnailGenerator::allocate(Default::default());
        let ithumbnail_handle = instance.query_interface::<IThumbnailProvider>().unwrap();

        let hbitmap: HBITMAP = std::ptr::null_mut();
        let pdw_alpha: PUINT = std::ptr::null_mut();

        let data = r"solid Exported from Blender-2.82 (sub 7)
        facet normal 0.000000 0.000000 1.000000
        outer loop
        vertex -1.000000 -1.000000 0.000000
        vertex 1.000000 -1.000000 0.000000
        vertex 0.000000 1.000000 0.000000
        endloop
        endfacet
        facet normal 0.000000 0.000000 1.000000
        outer loop
        vertex -1.000000 -1.000000 1.000000
        vertex 1.000000 -1.000000 1.000000
        vertex 0.000000 1.000000 1.000000
        endloop
        endfacet
        endsolid Exported from Blender-2.82 (sub 7)";

        instance.data.replace(data.as_bytes().to_vec());

        unsafe {
            ithumbnail_handle.get_thumbnail(
                512,
                &hbitmap as *const _ as *mut HBITMAP,
                &pdw_alpha as *const _ as PUINT,
            );
        }

        println!("Bitmap handle {:?}, alpha {:?}", hbitmap, pdw_alpha);
        assert_ne!(hbitmap, std::ptr::null_mut());
        assert_ne!(pdw_alpha, std::ptr::null_mut());
    }
}
