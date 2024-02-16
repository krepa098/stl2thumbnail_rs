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

use stl2thumbnail::gcode;
use stl2thumbnail::picture::Picture;
use stl2thumbnail::stl::parser::Parser;
use stl2thumbnail::stl::rasterbackend::RasterBackend;

pub unsafe fn read_all_from_stream(data: &RefCell<Vec<u8>>, pstream: LPSTREAM, _grf_mode: DWORD) -> HRESULT {
    // figure out the length of the stream
    let mut stat: STATSTG = std::mem::zeroed();

    if (*pstream).Stat(&mut stat, STATFLAG_NONAME) != S_OK {
        return -2;
    }

    let len = *stat.cbSize.QuadPart() as usize;

    // read the entire stream
    data.replace(vec![0; len as usize]);
    let res = (*pstream).Read(
        data.borrow_mut().as_mut_ptr() as *mut c_void,
        len as u32,
        std::ptr::null_mut(),
    );

    if res != S_OK {
        return -1; // error
    }

    return S_OK;
}

pub mod stl_impl {
    use super::*;

    pub unsafe fn get_thumbnail(
        data: &RefCell<Vec<u8>>,
        cx: UINT,            // size in x & y dimension
        phbmp: *mut HBITMAP, // data ptr
        pdw_alpha: PUINT,
    ) -> com::sys::HRESULT {
        let data = data.borrow();
        let reader = Cursor::new(data.as_slice());
        let mut parser = Parser::from_buf(reader, false).expect("Invalid input");

        if let Ok(mesh) = parser.read_all() {
            let mut backend = RasterBackend::new(cx as u32, cx as u32);
            let (aabb, scale) = backend.fit_mesh_scale(&mesh);
            backend.render_options.zoom = 1.05;
            backend.render_options.draw_size_hint = cx >= 256;
            backend.render_options.grid_visible = false;
            let pic = backend.render(&mesh, scale, &aabb, Some(Duration::from_secs(20)));

            *phbmp = create_hbitmap_from_picture(&pic);
            *pdw_alpha = 0x2; // WTSAT_ARGB

            return S_OK;
        }

        -1 // error
    }
}

pub mod gcode_impl {
    use super::*;

    pub unsafe fn get_thumbnail(
        data: &RefCell<Vec<u8>>,
        cx: UINT,            // size in x & y dimension
        phbmp: *mut HBITMAP, // data ptr
        pdw_alpha: PUINT,
    ) -> com::sys::HRESULT {
        let data = data.borrow();

        if let Ok(mut previews) = gcode::extract_previews_from_data(data.as_slice()) {
            if let Some(pic) = previews.last_mut() {
                pic.resize_keep_aspect_ratio(cx as u32, cx as u32);

                *phbmp = create_hbitmap_from_picture(&pic);
                *pdw_alpha = 0x2; // WTSAT_ARGB

                return S_OK;
            }
        }

        -1 // error
    }
}

fn create_hbitmap_from_picture(pic: &Picture) -> HBITMAP {
    let bgra_data = pic.to_bgra();
    let data = bgra_data.as_ptr() as *mut c_void;
    let width = pic.width() as i32;
    let height = pic.height() as i32;

    // Windows allocates the memory for this bitmap and copies the 'data' to its own buffer
    // Important: The image format here is B8G8R8A8
    unsafe { CreateBitmap(width, height, 1, 32, data) }
}
