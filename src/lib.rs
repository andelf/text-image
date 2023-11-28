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
    inverse: bool,
}

impl Parse for TextImageOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut opts = TextImageOptions {
            text: "".to_string(),
            font: "".to_string(),
            font_size: 16.0,
            inverse: false,
        };

        loop {
            let name: Ident = input.parse()?;

            match &*name.to_string() {
                "text" => {
                    input.parse::<Token![=]>()?;
                    let text: Lit = input.parse()?;

                    let text = if let Lit::Str(text) = &text {
                        text.value()
                    } else {
                        return Err(syn::Error::new_spanned(text, "expected a string literal"));
                    };

                    opts.text = text;
                }
                "font" => {
                    input.parse::<Token![=]>()?;
                    let font: Lit = input.parse()?;

                    let font = if let Lit::Str(font) = &font {
                        font.value()
                    } else {
                        return Err(syn::Error::new_spanned(font, "expected a string literal"));
                    };

                    opts.font = font;
                }
                "font_size" => {
                    input.parse::<Token![=]>()?;
                    let font_size: Lit = input.parse()?;

                    let font_size = if let Lit::Float(font_size) = &font_size {
                        font_size.base10_parse()?
                    } else {
                        return Err(syn::Error::new_spanned(
                            font_size,
                            "expected a float literal",
                        ));
                    };

                    opts.font_size = font_size;
                }
                "inverse" => {
                    opts.inverse = true;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        name,
                        "expected `text`, `font`, `font_size` or `inverse`",
                    ));
                }
            }

            let _ = input.parse::<Token![,]>();
            if input.is_empty() {
                break;
            }
        }

        Ok(opts)
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
///     inverse,
///   );
///   let raw_image = ImageRaw::<Gray8>::new(raw, w);
/// }
///
/// ````
#[proc_macro]
pub fn text_image(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TextImageOptions);
    println!("text_image: {:#?}", input);

    let font_raw = std::fs::read(input.font).unwrap();
    let font = Font::try_from_vec(font_raw).unwrap();

    let scale = Scale {
        x: input.font_size,
        y: input.font_size,
    };

    let metric = font.v_metrics(scale);
    let line_height = (metric.ascent - metric.descent + metric.line_gap)
        .abs()
        .ceil() as i32;

    let mut h = 0;
    let mut w = 0;
    let mut lines = 0;

    for line in input.text.lines() {
        let (lw, _lh) = text_size(scale, &font, line);
        w = w.max(lw);
        h += line_height;
        lines += 1;
    }
    w += 1;
    if w % 16 != 0 {
        w += 16 - (w % 16);
    }
    println!("text_image: result size {}x{}, {} lines", w, h, lines);

    let mut image: image::ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(w as _, h as _);

    let mut luma = 0xFF;
    if input.inverse {
        image.fill(0xFF);
        luma = 0x00;
    }

    for (i, line) in input.text.lines().enumerate() {
        // 1 px offset for blending
        draw_text_mut(
            &mut image,
            Luma([luma]),
            1,
            line_height * (i as i32),
            scale,
            &font,
            &line,
        );
    }

    let raw = image.into_raw();

    // convert from 8-bit grayscale to 1-bit compressed bytes
    let raw: Vec<u8> = raw
        .chunks(2)
        .map(|ch| (ch[1] >> 4) | (ch[0] & 0xF0))
        .collect();

    let raw_bytes = Lit::ByteStr(LitByteStr::new(&raw, proc_macro2::Span::call_site()));

    let w = w as u32;
    let h = h as u32;

    // TODO: binary support https://github.com/image-rs/image/issues/640

    let expanded = quote! {
        (#w, #h, #raw_bytes)
    };

    TokenStream::from(expanded)
}
