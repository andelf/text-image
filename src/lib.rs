#![feature(proc_macro_diagnostic)]

use image::{GrayImage, Luma};
use imageproc::drawing::{draw_text_mut, text_size};
use proc_macro::TokenStream;
use quote::quote;
use rusttype::{Font, Scale};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, Lit, LitByteStr, Token};

#[derive(Debug)]
struct TextImageOptions {
    text: String,
    font: String,
    font_size: f32,
}

impl Parse for TextImageOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        if name.to_string() != "text" {
            return Err(syn::Error::new_spanned(name, "expected `text` as the first argument"));
        }
        input.parse::<Token![=]>()?;
        let text: Lit = input.parse()?;

        let text = if let Lit::Str(text) = &text {
            text.value()
        } else {
            return Err(syn::Error::new_spanned(text, "expected a string literal"));
        };

        input.parse::<Token![,]>()?;
        let name: Ident = input.parse()?;
        if name.to_string() != "font" {
            return Err(syn::Error::new_spanned(name, "expected `font` as the second argument"));
        }

        input.parse::<Token![=]>()?;
        let font: Lit = input.parse()?;

        let font = if let Lit::Str(font) = &font {
            font.value()
        } else {
            return Err(syn::Error::new_spanned(font, "expected a string literal"));
        };

        input.parse::<Token![,]>()?;
        let name: Ident = input.parse()?;
        if name.to_string() != "font_size" {
            return Err(syn::Error::new_spanned(name, "expected `font_size` as the third argument"));
        }

        input.parse::<Token![=]>()?;
        let font_size: Lit = input.parse()?;

        let font_size = if let Lit::Float(font_size) = &font_size {
            font_size.base10_parse()?
        } else {
            return Err(syn::Error::new_spanned(font_size, "expected a float literal"));
        };

        let _ = input.parse::<Token![,]>();
        Ok(TextImageOptions {
            text,
            font,
            font_size,
        })
    }
}

/// Generate a text image.
///
/// Usage:
///
/// ```rust
/// use text_image::text_image;
///
/// use embedded_graphics::{image::ImageRaw, pixelcolor::Gray8};
///
/// fn main() {
///   let (w, h, raw) = text_image!(
///     text = "Hello, world!哈哈这样也行",
///     font = "LXGWWenKaiScreen.ttf",
///     font_size = 48.0,
///   );
/// }
///
/// ````
#[proc_macro]
pub fn text_image(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TextImageOptions);
    println!("text_image: {:#?}", input);

    let font = std::fs::read(input.font).unwrap();
    let font = Font::try_from_vec(font).unwrap();

    let scale = Scale {
        x: input.font_size,
        y: input.font_size,
    };

    let (mut w, h) = text_size(scale, &font, &input.text);

    if w % 8 != 0 {
        w += 8 - (w % 8);
    }

    println!("text_image: result size {}x{}", w, h);

    let mut image: image::ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(w as _, h as _);

    draw_text_mut(&mut image, Luma([255u8]), 0, 0, scale, &font, &input.text);

    let raw = image.into_raw();

    let raw_bytes = Lit::ByteStr(LitByteStr::new(&raw, proc_macro2::Span::call_site()));

    let w = w as u32;
    let h = h as u32;

    // TODO: binary support https://github.com/image-rs/image/issues/640

    let expanded = quote! {
        (#w, #h, #raw_bytes)
    };

    TokenStream::from(expanded)
}
