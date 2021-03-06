//! An image processing crate that provides a set of functions for image filtering, convolution, colour manipulation, and more.
//! 
//! #### Functions
//! 96 functions are available, including:
//! - **Image correction**: Hue rotation, sharpening, brightness adjustment, adjusting saturation, lightening/darkening all within various colour spaces. 
//! - **Convolutions**: Sobel filters, blurs, Laplace effects, edge detection, etc., 
//! - **Channel manipulation**: Increasing/decreasing RGB channel values, swapping channels, removing channels, etc.
//! - **Monochrome effects**: Duotoning, greyscaling of various forms, thresholding, sepia, averaging RGB values
//! - **Colour manipulation**: Work with the image in various colour spaces such as HSL, LCh, and sRGB, and adjust the colours accordingly. 
//! - **Filters**: Over 30 pre-set filters available, incorporating various effects and transformations. 
//! - **Text**: Apply text to imagery in artistic ways, or to watermark, etc.,
//! - **Watermarking**: Watermark images in multiple formats. 
//! - **Blending**: Blend images together using 10 different techniques, change image backgrounds. 
//! 
//! ## Example 
//! ```rust
//! extern crate photon;

//! fn main() {
//!     let img = photon::helpers::open_image("valley.PNG");
//!     photon::effects::solarize(&mut img);
//!     // Write the contents of this image in PNG format.
//!     photon::helpers::save_image(img, "new_image.PNG");
//! }
//! ```
//! 
//! This crate contains built-in preset functions, which provide default image processing functionality, as well as functions
//! that allow for direct, low-level access to channel manipulation.
//! To view a full demo of filtered imagery, visit the [official website](https://silvia-odwyer.github.io/photon).
//! 
//! ### WebAssembly Use
//! To allow for universal communication between the core Rust library and WebAssembly, the functions have been generalised to allow for both native and in-browser use. 
//! Due to this, image data from the browser must first be converted to a PhotonImage before being passed to the image processing functions. 
//! The PhotonImage can then be converted back to JS-compatible ImageData so that it can be displayed in-browser.
//! See the code snippet below:
//! 
//! ```js
//!function filterImage() {
//!     // Create a canvas and get a 2D context from the canvas
//!     var canvas = document.getElementById("canvas");
//!     var ctx = canvas.getContext("2d"); 
//!     
//!     // Draw the image element onto the canvas
//!     ctx.drawImage(newimg, 0, 0);
//!     
//!     // Convert the ImageData found in the canvas to a PhotonImage (so that it can communicate with the core Rust library)
//!     let rust_image = module.open_image(canvas, ctx);
//!     
//!     // Filter the image, the PhotonImage's raw pixels are modified
//!     module.filter(rust_image, "radio");
//!     
//!     // Place the PhotonImage back on the canvas
//!     ctx.putImageData(rust_image, 0, 0)
//! }
//! ```
//! 
//! ### Live Demo
//! View the [official demo of WASM in action](https://silvia-odwyer.github.io/photon).
//! Not all functions available in the core Rust library are available in WebAssembly (currently investigating this). Only WASM-friendly functions have been annotated with #[wasm_bindgen]. All supported WASM functions are displayed in the starter demo. 

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, ImageData, HtmlCanvasElement};
use wasm_bindgen::Clamped;
use image::{GenericImage, GenericImageView};
use base64::decode;
use serde::{Serialize, Deserialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Provides the image's height, width, and contains the image's raw pixels.
/// For use when communicating between JS and WASM, and also natively. 
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct PhotonImage {
    raw_pixels: Vec<u8>,
    width: u32, 
    height: u32,
}

#[wasm_bindgen]
impl PhotonImage {   
    #[wasm_bindgen(constructor)]
    pub fn new(raw_pixels: Vec<u8>, width: u32, height: u32) -> PhotonImage {
        return PhotonImage { raw_pixels: raw_pixels, width: width, height: height};
    }

    /// Create a new PhotonImage from a base64 string.
    pub fn new_from_base64(base64: &str) -> PhotonImage {
        let image = base64_to_image(base64);
        return image;
    }

    pub fn new_from_byteslice(vec: Vec<u8>) -> PhotonImage {    
        let slice = vec.as_slice();

        let img = image::load_from_memory(slice).unwrap();
        
        let raw_pixels = img.raw_pixels();
        
        return PhotonImage { raw_pixels: raw_pixels, width: img.width(), height: img.height()};
    
    }

    /// Get the width of the PhotonImage.
    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_raw_pixels(&self) -> Vec<u8> {
        self.raw_pixels.clone()
    }

    /// Get the height of the PhotonImage.
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Convert the PhotonImage's raw pixels to JS-compatible ImageData.
    pub fn get_image_data(&mut self) -> ImageData {
        let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.raw_pixels), self.width, self.height).unwrap();
        new_img_data
    }

    /// Convert ImageData to raw pixels, and update the PhotonImage's raw pixels to this.
    pub fn set_imgdata(&mut self, img_data: ImageData) {
        let width = img_data.width();
        let height = img_data.height();
        let raw_pixels = to_raw_pixels(img_data);
        self.width = width;
        self.height = height;
        self.raw_pixels = raw_pixels;
    }
}

/// Create a new PhotonImage from a raw Vec of u8s representing raw image pixels.
impl From<ImageData> for PhotonImage {
    fn from(imgdata: ImageData) -> Self {
        let width = imgdata.width();
        let height = imgdata.height();
        let raw_pixels = to_raw_pixels(imgdata);
        return PhotonImage {raw_pixels: raw_pixels, width: width, height: height}
    }
}

/// RGB color type.
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8
}

#[wasm_bindgen]
impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        return Rgb {r: r, g: g, b: b}
    }

    pub fn set_red(&mut self, r: u8) {
        self.r = r;
    }

    pub fn set_green(&mut self, g: u8) {
        self.g = g;
    }

    pub fn set_blue(&mut self, b: u8) {
        self.b = b;
    }

    pub fn get_red(&self) -> u8 {
        self.r
    }

    pub fn get_green(&self) -> u8 {
        self.g
    }

    pub fn get_blue(&self) -> u8 {
        self.b
    }
}

impl From<Vec<u8>> for Rgb {
    fn from(vec: Vec<u8>) -> Self {
        if vec.len() != 3 {
            panic!("Vec length must be equal to 3.")
        }
        let rgb = Rgb::new(vec[0], vec[1], vec[2]);
        rgb
    }
}

/// Create a PhotonImage from a byte slice.


///! [temp] Check if WASM is supported.
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let window = web_sys::window().expect("No Window found, should have a Window");
    let document = window.document().expect("No Document found, should have a Document");

    let p: web_sys::Node = document.create_element("p")?.into();
    p.set_text_content(Some("You're successfully running WASM!"));

    let body = document.body().expect("ERR: No body found, should have a body");
    let body: &web_sys::Node = body.as_ref();
    body.append_child(&p)?;
    Ok(())
}

/// Get the ImageData from a 2D canvas context
#[wasm_bindgen]
pub fn get_image_data(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) -> ImageData {
    set_panic_hook();
    let width = canvas.width();
    let height = canvas.height();

    // let data: ImageData = ctx.get_image_data(0.0, 0.0, 100.0, 100.0).unwrap();
    let data = ctx.get_image_data(0.0, 0.0, width as f64, height as f64).unwrap();
    let _vec_data = data.data().to_vec();
    return data;
}

/// Place a PhotonImage onto a 2D canvas.
#[wasm_bindgen]
pub fn putImageData(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d, mut new_image: PhotonImage) {
    // Convert the raw pixels back to an ImageData object.
    let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut new_image.raw_pixels), canvas.width(), canvas.height());

    // Place the new imagedata onto the canvas
    ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0);
}

/// Convert a HTML5 Canvas Element to a PhotonImage.
/// 
/// This converts the ImageData found in the canvas context to a PhotonImage,
/// which can then have effects or filters applied to it.
#[wasm_bindgen]
#[no_mangle]
pub fn open_image(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> PhotonImage {
    let imgdata = get_image_data(&canvas, &ctx);
    let raw_pixels = to_raw_pixels(imgdata);
    return PhotonImage {raw_pixels: raw_pixels, width: canvas.width(), height: canvas.height() }
}


/// Convert ImageData to a raw pixel vec of u8s.
#[wasm_bindgen]
pub fn to_raw_pixels(imgdata: ImageData) -> Vec<u8> {
    let img_vec = imgdata.data().to_vec();
    return img_vec;
}

/// Convert a base64 string to a PhotonImage.
#[wasm_bindgen]
pub fn base64_to_image(base64: &str) -> PhotonImage {

    let base64_to_vec: Vec<u8> = base64_to_vec(base64);

    let slice = base64_to_vec.as_slice();

    let img = image::load_from_memory(slice).unwrap();
    
    let raw_pixels = img.raw_pixels();
    
    return PhotonImage { raw_pixels: raw_pixels, width: img.width(), height: img.height()};

}

/// Convert a base64 string to a Vec of u8s.
#[wasm_bindgen]
pub fn base64_to_vec(base64: &str) -> Vec<u8> {
    let vec = decode(base64).unwrap();
    return vec;
}

/// Convert a PhotonImage to JS-compatible ImageData.
#[wasm_bindgen]
pub fn to_image_data(photon_image: PhotonImage) -> ImageData {
    let mut raw_pixels = photon_image.raw_pixels;
    let width = photon_image.width;
    let height = photon_image.height;
    let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut raw_pixels), width, height).unwrap();

    return new_img_data;
}

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub mod channels;
pub mod effects;
pub mod transform;
pub mod conv;
pub mod filters;
pub mod monochrome;
pub mod native;
pub mod text;
pub mod colour_spaces;
pub mod multiple;
pub mod noise;
pub mod helpers;
mod tests;