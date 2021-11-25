//! X11 wrapper for the `Display` struct

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]

use crate::{Result, Error};

pub type Xid      = ::std::os::raw::c_ulong;
pub type XPointer = *mut ::std::os::raw::c_char;
pub type VisualID = ::std::os::raw::c_ulong;
pub type Colormap = ::std::os::raw::c_ulong;
pub type Window   = ::std::os::raw::c_ulong;
pub type Drawable = ::std::os::raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Display {
    pub ext_data: *mut XExtData,
    pub private1: *mut XPrivate,
    pub fd: ::std::os::raw::c_int,
    pub private2: ::std::os::raw::c_int,
    pub proto_major_version: ::std::os::raw::c_int,
    pub proto_minor_version: ::std::os::raw::c_int,
    pub vendor: *mut ::std::os::raw::c_char,
    pub private3: Xid,
    pub private4: Xid,
    pub private5: Xid,
    pub private6: ::std::os::raw::c_int,
    pub resource_alloc: ::std::option::Option<unsafe extern "C" fn(arg1: *mut Display) -> Xid>,
    pub byte_order: ::std::os::raw::c_int,
    pub bitmap_unit: ::std::os::raw::c_int,
    pub bitmap_pad: ::std::os::raw::c_int,
    pub bitmap_bit_order: ::std::os::raw::c_int,
    pub nformats: ::std::os::raw::c_int,
    pub pixmap_format: *mut ScreenFormat,
    pub private8: ::std::os::raw::c_int,
    pub release: ::std::os::raw::c_int,
    pub private9: *mut XPrivate,
    pub private10: *mut XPrivate,
    pub qlen: ::std::os::raw::c_int,
    pub last_request_read: ::std::os::raw::c_ulong,
    pub request: ::std::os::raw::c_ulong,
    pub private11: XPointer,
    pub private12: XPointer,
    pub private13: XPointer,
    pub private14: XPointer,
    pub max_request_size: ::std::os::raw::c_uint,
    pub db: *mut _XrmHashBucketRec,
    pub private15:
        ::std::option::Option<unsafe extern "C" fn(arg1: *mut Display) -> ::std::os::raw::c_int>,
    pub display_name: *mut ::std::os::raw::c_char,
    pub default_screen: ::std::os::raw::c_int,
    pub nscreens: ::std::os::raw::c_int,
    pub screens: *mut Screen,
    pub motion_buffer: ::std::os::raw::c_ulong,
    pub private16: ::std::os::raw::c_ulong,
    pub min_keycode: ::std::os::raw::c_int,
    pub max_keycode: ::std::os::raw::c_int,
    pub private17: XPointer,
    pub private18: XPointer,
    pub private19: ::std::os::raw::c_int,
    pub xdefaults: *mut ::std::os::raw::c_char,
}


impl Display {
    /// Get the `default_screen` 
    pub fn default_screen(&self) -> Result<usize> {
        usize::try_from(self.default_screen).map_err(|_| Error::InvalidDefaultScreen)
    }

    /// Get the [`Screen`] indexed by `screen`
    pub fn screen(&self, screen: usize) -> &Screen {
        let num_screens = usize::try_from(self.nscreens).unwrap();
        assert!(screen <= num_screens);

        let screens = unsafe {
            std::slice::from_raw_parts(self.screens, num_screens)
        };

        &screens[screen]
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Screen {
    pub ext_data: *mut XExtData,
    pub display: *mut Display,
    pub root: Window,
    pub width: ::std::os::raw::c_int,
    pub height: ::std::os::raw::c_int,
    pub mwidth: ::std::os::raw::c_int,
    pub mheight: ::std::os::raw::c_int,
    pub ndepths: ::std::os::raw::c_int,
    pub depths: *mut Depth,
    pub root_depth: ::std::os::raw::c_int,
    pub root_visual: *mut Visual,
    pub default_gc: GC,
    pub cmap: Colormap,
    pub white_pixel: ::std::os::raw::c_ulong,
    pub black_pixel: ::std::os::raw::c_ulong,
    pub max_maps: ::std::os::raw::c_int,
    pub min_maps: ::std::os::raw::c_int,
    pub backing_store: ::std::os::raw::c_int,
    pub save_unders: ::std::os::raw::c_int,
    pub root_input_mask: ::std::os::raw::c_long,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Depth {
    pub depth: ::std::os::raw::c_int,
    pub nvisuals: ::std::os::raw::c_int,
    pub visuals: *mut Visual,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Visual {
    pub ext_data: *mut XExtData,
    pub visualid: VisualID,
    pub class: ::std::os::raw::c_int,
    pub red_mask: ::std::os::raw::c_ulong,
    pub green_mask: ::std::os::raw::c_ulong,
    pub blue_mask: ::std::os::raw::c_ulong,
    pub bits_per_rgb: ::std::os::raw::c_int,
    pub map_entries: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XExtData {
    pub number: ::std::os::raw::c_int,
    pub next: *mut XExtData,
    pub free_private: ::std::option::Option<
        unsafe extern "C" fn(extension: *mut XExtData) -> ::std::os::raw::c_int,
    >,
    pub private_data: XPointer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XPrivate {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ScreenFormat {
    pub ext_data: *mut XExtData,
    pub depth: ::std::os::raw::c_int,
    pub bits_per_pixel: ::std::os::raw::c_int,
    pub scanline_pad: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _XrmHashBucketRec {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _XGC {
    _unused: [u8; 0],
}
pub type GC = *mut _XGC;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XImage {
    pub width: ::std::os::raw::c_int,
    pub height: ::std::os::raw::c_int,
    pub xoffset: ::std::os::raw::c_int,
    pub format: ::std::os::raw::c_int,
    pub data: *mut ::std::os::raw::c_char,
    pub byte_order: ::std::os::raw::c_int,
    pub bitmap_unit: ::std::os::raw::c_int,
    pub bitmap_bit_order: ::std::os::raw::c_int,
    pub bitmap_pad: ::std::os::raw::c_int,
    pub depth: ::std::os::raw::c_int,
    pub bytes_per_line: ::std::os::raw::c_int,
    pub bits_per_pixel: ::std::os::raw::c_int,
    pub red_mask: ::std::os::raw::c_ulong,
    pub green_mask: ::std::os::raw::c_ulong,
    pub blue_mask: ::std::os::raw::c_ulong,
    pub obdata: XPointer,
    pub f: XImage_funcs,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XImage_funcs {
    pub create_image: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut Display,
            arg2: *mut Visual,
            arg3: ::std::os::raw::c_uint,
            arg4: ::std::os::raw::c_int,
            arg5: ::std::os::raw::c_int,
            arg6: *mut ::std::os::raw::c_char,
            arg7: ::std::os::raw::c_uint,
            arg8: ::std::os::raw::c_uint,
            arg9: ::std::os::raw::c_int,
            arg10: ::std::os::raw::c_int,
        ) -> *mut XImage,
    >,
    pub destroy_image:
        ::std::option::Option<unsafe extern "C" fn(arg1: *mut XImage) -> ::std::os::raw::c_int>,
    pub get_pixel: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut XImage,
            arg2: ::std::os::raw::c_int,
            arg3: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_ulong,
    >,
    pub put_pixel: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut XImage,
            arg2: ::std::os::raw::c_int,
            arg3: ::std::os::raw::c_int,
            arg4: ::std::os::raw::c_ulong,
        ) -> ::std::os::raw::c_int,
    >,
    pub sub_image: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut XImage,
            arg2: ::std::os::raw::c_int,
            arg3: ::std::os::raw::c_int,
            arg4: ::std::os::raw::c_uint,
            arg5: ::std::os::raw::c_uint,
        ) -> *mut XImage,
    >,
    pub add_pixel: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut XImage,
            arg2: ::std::os::raw::c_long,
        ) -> ::std::os::raw::c_int,
    >,
}
