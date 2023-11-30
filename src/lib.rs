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
    line_spacing: i32,
    // 2, 4, or 8
    gray_depth: i32,
}

impl Parse for TextImageOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut opts = TextImageOptions {
            text: "".to_string(),
            font: "".to_string(),
            font_size: 16.0,
            inverse: false,
            line_spacing: 0,
            gray_depth: 1,
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
                "line_spacing" => {
                    input.parse::<Token![=]>()?;
                    let line_spacing: Lit = input.parse()?;

                    let line_spacing = if let Lit::Int(line_spacing) = &line_spacing {
                        line_spacing.base10_parse()?
                    } else {
                        return Err(syn::Error::new_spanned(
                            line_spacing,
                            "expected a integer literal",
                        ));
                    };

                    opts.line_spacing = line_spacing;
                }
                "inverse" => {
                    opts.inverse = true;
                }
                "Gray2" => {
                    opts.gray_depth = 2;
                }
                "Gray4" => {
                    opts.gray_depth = 4;
                }
                "Gray8" => {
                    opts.gray_depth = 8;
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

        // check required
        if opts.text.is_empty() {
            return Err(syn::Error::new_spanned("text", "required option `text` is missing"));
        }
        if opts.font.is_empty() {
            return Err(syn::Error::new_spanned("font", "required option `font` is missing"));
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
    let opts = parse_macro_input!(input as TextImageOptions);
    println!("text_image: {:#?}", opts);

    let font_raw = std::fs::read(opts.font).expect("Can not read font file");
    let font = Font::try_from_vec(font_raw).unwrap();

    let scale = Scale {
        x: opts.font_size,
        y: opts.font_size,
    };

    let metric = font.v_metrics(scale);
    println!("metric: {:#?}", metric);
    let line_height = (metric.ascent - metric.descent + metric.line_gap)
        .abs()
        .ceil() as i32;
    println!("line_height: {}", line_height);

    let mut h = 0;
    let mut w = 0;
    let mut lines = 0;

    for line in opts.text.lines() {
        let (lw, _lh) = text_size(scale, &font, line);
        println!("lh => {}", _lh);
        w = w.max(lw);
        h += line_height;
        lines += 1;
    }
    w += 1;
    h += opts.line_spacing as i32 * (lines - 1);

    // align to byte
    if w / opts.gray_depth % 8 != 0 {
        w += opts.gray_depth as i32 * (8 - (w / opts.gray_depth % 8));
    }
    println!("text_image: result size {}x{}, {} lines", w, h, lines);

    let mut image: image::ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(w as _, h as _);

    let mut luma = 0xFF;
    if opts.inverse {
        image.fill(0xFF);
        luma = 0x00;
    }

    for (i, line) in opts.text.lines().enumerate() {
        // 1 px offset for blending
        draw_text_mut(
            &mut image,
            Luma([luma]),
            1,
            (line_height + opts.line_spacing) * (i as i32) - 1,
            scale,
            &font,
            &line,
        );
    }

    let raw = image.into_raw();

    // convert depth
    let raw: Vec<u8> = match opts.gray_depth {
        8 => raw,
        4 => raw
            .chunks(2)
            .map(|ch| (ch[1] >> 4) | (ch[0] & 0xF0))
            .collect(),
        2 => {
            let mut ret = Vec::with_capacity(raw.len() / 4);
            for ch in raw.chunks(4) {
                ret.push(
                    (ch[3] >> 6) | ((ch[2] >> 4) & 0x0C) | ((ch[1] >> 2) & 0x30) | (ch[0] & 0xC0),
                );
            }
            ret
        }
        1 => {
            let mut ret = Vec::with_capacity(raw.len() / 8);
            for ch in raw.chunks(8) {
                ret.push(
                    (ch[7] >> 7)
                        | ((ch[6] >> 6) & 0x02)
                        | ((ch[5] >> 5) & 0x04)
                        | ((ch[4] >> 4) & 0x08)
                        | ((ch[3] >> 3) & 0x10)
                        | ((ch[2] >> 2) & 0x20)
                        | ((ch[1] >> 1) & 0x40)
                        | (ch[0] & 0x80),
                );
            }
            ret
        }
        _ => unreachable!(),
    };

    // convert from 8-bit grayscale to 1-bit compressed bytes

    let raw_bytes = Lit::ByteStr(LitByteStr::new(&raw, proc_macro2::Span::call_site()));

    let w = w as u32;
    let h = h as u32;

    // TODO: binary support https://github.com/image-rs/image/issues/640

    let expanded = quote! {
        (#w, #h, #raw_bytes)
    };

    TokenStream::from(expanded)
}
