use crate::com_interface::{IInitializeWithStream, IThumbnailProvider};

use com::sys::{HRESULT, S_OK};

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, PUINT, UINT};
use winapi::shared::windef::HBITMAP;
use winapi::shared::wtypes::STATFLAG_NONAME;
use winapi::um::objidlbase::{LPSTREAM, STATSTG};
use winapi::um::wingdi::CreateBitmap;

use std::cell::RefCell;
use std::io::Cursor;
use std::time::Duration;

use stl2thumbnail::parser::Parser;
use stl2thumbnail::picture::Picture;
use stl2thumbnail::rasterbackend::RasterBackend;

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
            let data = self.data.borrow();
            let reader = Cursor::new(data.as_slice());
            let mut parser = Parser::from_buf(reader, false).expect("Invalid input");

            if let Ok(mesh) = parser.read_all() {
                let mut backend = RasterBackend::new(cx as u32, cx as u32);
                let (aabb, scale) = backend.fit_mesh_scale(&mesh);
                backend.render_options.zoom = 1.05;
                // Note: Icon cache sizes seem to be 16,32,48,96,256,768,1280,...
                // 256 this is actually too small to be readable when it first appears,
                // but 768 makes no sense either
                backend.render_options.draw_size_hint = cx >= 256;
                let pic = backend.render(&mesh, scale, &aabb, Some(Duration::from_secs(20)));

                *phbmp = create_hbitmap_from_picture(&pic);
                *pdw_alpha = 0x2; // WTSAT_ARGB

                return S_OK;
            }

            -1 // error

            // **this works for testing**
            // let mut pic = Picture::new(cx as usize, cx as usize);
            // pic.test_pattern();
            // *phbmp = create_hbitmap_from_picture(&pic);
            // *pdw_alpha = 0x2; // WTSAT_ARGB

            // return NOERROR
        }
    }
    impl IInitializeWithStream for WinSTLThumbnailGenerator {
        unsafe fn initialize(&self, pstream: LPSTREAM, _grf_mode: DWORD) -> HRESULT {
              // figure out the length of the stream
              let mut stat: STATSTG = std::mem::zeroed();

                if (*pstream).Stat(&mut stat, STATFLAG_NONAME) != S_OK {
                    return -2;
                }

                let len = *stat.cbSize.QuadPart() as usize;

                println!("Got stream of length {}", len);

                // read the entire stream
                self.data.replace(vec![0; len as usize]);
                let res = (*pstream).Read(
                    self.data.borrow_mut().as_mut_ptr() as *mut c_void,
                    len as u32,
                    std::ptr::null_mut(),
                );

                if res != S_OK {
                    return -1; // error
                }

                return S_OK;
        }
    }
} // class

fn create_hbitmap_from_picture(pic: &Picture) -> HBITMAP {
    let bgra_data = pic.to_bgra();
    let data = bgra_data.as_ptr() as *mut c_void;
    let width = pic.width() as i32;
    let height = pic.height() as i32;

    // Windows allocates the memory for this bitmap and copies the 'data' to its own buffer
    // Important: The image format here is B8G8R8A8
    unsafe { CreateBitmap(width, height, 1, 32, data) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;

    #[test]
    fn win_bitmap() {
        let mut pic = Picture::new(512, 512);
        pic.fill(&(1.0, 0.0, 0.0, 1.0).into());

        let hbitmap = create_hbitmap_from_picture(&pic);

        assert!(hbitmap != null_mut());
    }

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
