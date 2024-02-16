use std::{ffi::CStr, mem::forget, os::raw::c_char};

use crate::{gcode, stl::parser::Parser, stl::rasterbackend::RasterBackend, threemf};

#[repr(C)]
pub struct PictureBuffer {
    /// data in rgba8888 format
    data: *const u8,
    /// length of the buffer
    len: u32,
    /// stride of the picture
    stride: u32,
    /// depth of the picture
    depth: u32,
    /// width of the picture
    width: u32,
    /// height of the picture
    height: u32,
}

#[repr(C)]
pub struct RenderSettings {
    /// width of the image
    width: u32,
    /// height of the image
    height: u32,
    /// embed a size hint
    size_hint: bool,
    /// draw grid
    grid: bool,
    /// max duration of the rendering, 0 to disable
    timeout: u64,
}

#[no_mangle]
/// Renders a mesh to a picture
///
/// Free the buffer with free_picture_buffer
///
/// # Safety
/// path has to be a valid pointer to a null terminated string
pub unsafe extern "C" fn render_stl(path: *const c_char, settings: RenderSettings) -> PictureBuffer {
    if !path.is_null() {
        let path = CStr::from_ptr(path).to_str();

        if let Ok(path) = path {
            let mut backend = RasterBackend::new(settings.width, settings.height);
            let parser = Parser::from_file(path, true);

            if let Ok(mut parser) = parser {
                let mesh = parser.read_all();

                if let Ok(mesh) = mesh {
                    let (aabb, scale) = backend.fit_mesh_scale(&mesh);

                    // set flags
                    backend.render_options.draw_size_hint = settings.size_hint;
                    backend.render_options.grid_visible = settings.grid;

                    // render
                    let pic = backend.render(&mesh, scale, &aabb, None);

                    let boxed_data = pic.data_as_boxed_slice();
                    let data = boxed_data.as_ptr();
                    let len = pic.data().len() as u32;

                    // leak the memory owned by boxed_data
                    forget(boxed_data);

                    return PictureBuffer {
                        data,
                        len,
                        stride: pic.stride(),
                        depth: pic.depth(),
                        width: pic.width(),
                        height: pic.height(),
                    };
                }
            }
        }
    }

    PictureBuffer {
        data: std::ptr::null(),
        len: 0,
        stride: 0,
        depth: 0,
        width: 0,
        height: 0,
    }
}

#[no_mangle]
/// Extracts the thumbnail embedded into the gcode
/// If there are multiple thumbnails, the one with
/// the highest resolution is returned
///
/// Free the buffer with free_picture_buffer
///
/// # Safety
/// path has to be a valid pointer to a null terminated string
pub unsafe extern "C" fn extract_gcode_preview(path: *const c_char, width: u32, height: u32) -> PictureBuffer {
    if !path.is_null() {
        let path = CStr::from_ptr(path).to_str();

        if let Ok(path) = path {
            if let Ok(mut previews) = gcode::extract_previews_from_file(path) {
                if let Some(pic) = previews.last_mut() {
                    pic.resize_keep_aspect_ratio(width, height);

                    let boxed_data = pic.data_as_boxed_slice();
                    let data = boxed_data.as_ptr();
                    let len = boxed_data.len() as u32;

                    // leak the memory owned by boxed_data
                    forget(boxed_data);

                    return PictureBuffer {
                        data,
                        len,
                        stride: pic.stride(),
                        depth: pic.depth(),
                        width: pic.width(),
                        height: pic.height(),
                    };
                }
            }
        }
    }

    PictureBuffer {
        data: std::ptr::null(),
        len: 0,
        stride: 0,
        depth: 0,
        width: 0,
        height: 0,
    }
}

#[no_mangle]
/// Extracts the thumbnail embedded into the 3mf file
///
/// Free the buffer with free_picture_buffer
///
/// # Safety
/// path has to be a valid pointer to a null terminated string
pub unsafe extern "C" fn extract_3mf_preview(path: *const c_char, width: u32, height: u32) -> PictureBuffer {
    if !path.is_null() {
        let path = CStr::from_ptr(path).to_str();

        if let Ok(path) = path {
            if let Ok(mut pic) = threemf::extract_preview_from_file(path) {
                pic.resize_keep_aspect_ratio(width, height);

                let boxed_data = pic.data_as_boxed_slice();
                let data = boxed_data.as_ptr();
                let len = boxed_data.len() as u32;

                // leak the memory owned by boxed_data
                forget(boxed_data);

                return PictureBuffer {
                    data,
                    len,
                    stride: pic.stride(),
                    depth: pic.depth(),
                    width: pic.width(),
                    height: pic.height(),
                };
            }
        }
    }

    PictureBuffer {
        data: std::ptr::null(),
        len: 0,
        stride: 0,
        depth: 0,
        width: 0,
        height: 0,
    }
}

#[no_mangle]
/// Frees the memory of a PictureBuffer
pub extern "C" fn free_picture_buffer(buffer: &mut PictureBuffer) {
    unsafe {
        let s = std::slice::from_raw_parts_mut(buffer.data as *mut u8, buffer.len as usize);

        // put the memory back into the box such that is can be freed
        drop(Box::from_raw(s as *mut [u8]));

        buffer.data = std::ptr::null();
    }
}
