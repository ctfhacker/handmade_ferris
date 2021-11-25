//! X11 wrapper

#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]

mod display;
pub use display::{Display, Screen, Visual, Window, XImage, Drawable, GC};
 
/// Errors for the linux platform
#[derive(Debug)]
pub enum Error {
    /// Default screen index did not fit in `usize`
    InvalidDefaultScreen,
}

/// Wrapper `Result` type for the linux platform
pub type Result<T> = core::result::Result<T, Error>;

#[link(name = "X11")]
extern "system" {
    fn XOpenDisplay(display_name: *const u8) -> DisplayPtr;
    fn XCreateSimpleWindow(display: *mut Display, window: Window, 
        x: i32, y: i32, width: u32, height: u32, border_width: u32, border: u64,
        background: u64) -> Window;

    fn XMapWindow(display: *mut Display, window: Window) -> u32;
    fn XSelectInput(display: *mut Display, window: Window, event_mask: i64) -> i32;
    fn XNextEvent(display: *mut Display, event: *mut XEvent) -> i32;
    fn XFillRectangle(display: *mut Display, event: *mut XEvent, gc: GC,
        x: i32, y: i32, width: u32, height: u32) -> i32;
    fn XCreateImage(display: *mut Display, visual: *mut Visual, depth: u32, format: i32,
        offset: i32, data: *const u32, width: u32, height: u32, bitmap_pad: i32, 
        bytes_per_line: u32) -> *mut XImage;
    fn XPutImage(display: *mut Display, d: Drawable, gc: GC, image: *mut XImage,
        src_x: i32, src_y: i32, dest_x: i32, dest_y: i32, width: u32, height: u32) 
        -> i32;
    fn XSync(display: *mut Display, discard: bool);
}

#[repr(C)]
pub struct XEvent {
    type_: i32,
    pad: [u8; 0x400]
}

impl std::default::Default for XEvent {
    fn default() -> XEvent {
        XEvent {
            type_: i32::MAX,
            pad: [0x41_u8; 0x400]
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Event {
    KeyPress,
    Expose,
}

impl From<i32> for Event {
    fn from(val: i32) -> Event {
        match val {
            2 => Event::KeyPress,
           12 => Event::Expose,
           _  => panic!("Unknown event: {}", val)
        }
    }
}


const EXPOSURE_MASK:  i64 = 1 << 15;
const KEY_PRESS_MASK: i64 = 1 << 0;

/// Opaque display pointer returned from X11
#[repr(transparent)]
pub struct DisplayPtr(*mut Display);

impl std::ops::Deref for DisplayPtr {
    type Target = *mut Display;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Open an X11 display
fn open_display() -> Option<DisplayPtr> {
    unsafe {
        let display = XOpenDisplay(std::ptr::null());

        if display.0.is_null() {
            None
        } else {
            Some(display)
        }
    }
}

const ZPIXMAP: i32 = 2;

pub struct SimpleWindow {
    pub display: DisplayPtr,
    pub window: Window,
    pub framebuffer: Vec<u32>,
    pub image: Option<XImage>,
    pub width: u32,
    pub height: u32
}

impl SimpleWindow {
    /// Get the builder for a `SimpleWindow`
    pub fn build() -> SimpleWindowBuilder {
        SimpleWindowBuilder {
            x:            None, 
            y:            None, 
            width:        None, 
            height:       None, 
            border_width: None,
            border:       None, 
            background:   None
        }
    }
    
    /// Get the next event from the window
    pub fn next_event(&self) -> Event {
        let mut event = XEvent::default();
        unsafe { 
            XNextEvent(*self.display, &mut event); 
        }
        event.type_.into()
    }

    /// Get a reference to the display of the window
    pub fn display(&self) -> &Display {
        unsafe { &*(*self.display) }
    }

    /// Get a screen from the [`Display`] for this [`SimpleWindow`]
    pub fn screen(&self, screen: usize) -> &Screen {
        self.display().screen(screen)

    }

    /// Get the screen index for the default [`Screen`] of this [`Display`]
    pub fn default_screen(&self) -> usize {
        usize::try_from(self.display().default_screen).unwrap()
    }

    /// Get the root [`Window`] of this [`Display`]
    pub fn root_window(&self) -> u64 {
        self.display().screen(self.default_screen()).root
    }

    /// Get the default [`Visual`] of this [`Display`]
    pub fn default_visual(&self) -> &Visual {
        unsafe {
            &*self.screen(self.default_screen()).root_visual
        }
    }

    /// Get the default [`Visual`] of this [`Display`]
    pub fn default_visual_mut(&mut self) -> &mut Visual {
        unsafe {
            &mut *self.screen(self.default_screen()).root_visual
        }
    }

    /// Get the default depth of this [`Display`]
    pub fn default_depth(&self) -> u32 {
        let screen = self.screen(self.default_screen());
        u32::try_from(screen.root_depth).unwrap()
    }

    /// Get the default GC of this [`Display`]
    pub fn default_gc(&self) -> GC {
        let screen = self.screen(self.default_screen());
        screen.default_gc
    }

    pub fn create_image(&mut self) {
        self.image = Some(unsafe {
            *XCreateImage(
                *self.display, 
                self.default_visual_mut(), 
                /* depth:         */ self.default_depth(), 
                /* format:        */ ZPIXMAP,
                /* offset:        */ 0, 
                /* data:          */ self.framebuffer.as_ptr(),
                /* width:         */ self.width, 
                /* height:        */ self.height, 
                /* bitmap_pad:    */ 8, 
                /* bytes_per_line */ 0)
        });

        println!("Image: {:#x?}", self.image);
    }

    pub fn put_image(&mut self) {
        unsafe {
            let result = XPutImage(
                /* display: */ *self.display,
                /* d:       */ self.window,
                /* gc:      */ self.default_gc(), 
                /* image:   */ &mut self.image.unwrap(),
                /* src_x:   */ 0, 
                /* src_y:   */ 0, 
                /* dest_x:  */ 0, 
                /* dest_y:  */ 0, 
                /* width:   */ self.width, 
                /* height:  */ self.height);

            assert_eq!(result, 0);

            XSync(*self.display, false);
        };
    }
}

/// Builder to create a simple window
#[derive(Debug)]
pub struct SimpleWindowBuilder {
    x:            Option<i32>, 
    y:            Option<i32>, 
    width:        Option<u32>, 
    height:       Option<u32>, 
    border_width: Option<u32>,
    border:       Option<u64>, 
    background:   Option<u64>
}


impl SimpleWindowBuilder {
    pub fn x(mut self, val: i32) -> Self {
        self.x = Some(val);
        self
    }

    pub fn y(mut self, val: i32) -> Self {
        self.y = Some(val);
        self
    }

    pub fn width(mut self, val: u32) -> Self {
        self.width = Some(val);
        self
    }

    pub fn height(mut self, val: u32) -> Self {
        self.height = Some(val);
        self
    }

    pub fn border_width(mut self, val: u32) -> Self {
        self.border_width = Some(val);
        self
    }

    pub fn border(mut self, val: u64) -> Self {
        self.border = Some(val);
        self
    }

    pub fn background(mut self, val: u64) -> Self {
        self.background = Some(val);
        self
    }

    /// Create the `SimpleWindow` from the given parameters
    pub fn finish(self) -> Result<SimpleWindow> {
        let display = open_display().expect("Failed to open x11 display");

        unsafe {
            let curr_display = &*(*display);
            let screen_index = curr_display.default_screen()?;
            let screen       = curr_display.screen(screen_index);
            let root_window  = screen.root;

            let width  = self.width.unwrap_or(600);
            let height = self.width.unwrap_or(800);

            let window = XCreateSimpleWindow(
                *display, 
                root_window, 
                self.x.unwrap_or(0),
                self.y.unwrap_or(0),
                width,
                height,
                self.border_width.unwrap_or(2), 
                0,
                0
            );

            XSelectInput(*display, window, EXPOSURE_MASK | KEY_PRESS_MASK);

            XMapWindow(*display, window);

            let num_bytes   = usize::try_from(width * height).unwrap();
            let framebuffer = vec![0; num_bytes];

            Ok(SimpleWindow {
                display,
                window,
                framebuffer,
                image: None,
                width,
                height,
            })
        }
    }
}

