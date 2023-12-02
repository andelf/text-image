#![feature(iter_array_chunks)]

use image::{GenericImageView, GrayImage, Luma, Rgb};
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
///     Gray4,
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
    let line_height = (metric.ascent - metric.descent + metric.line_gap)
        .abs()
        .ceil() as i32;

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
    if w % 8 != 0 {
        w = (w / 8 + 1) * 8;
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

#[derive(Debug)]
struct MonochromeImageOptions {
    image: String,
    palette: Vec<u32>,
    /// index of the channel to use
    channel: u8,
}

impl MonochromeImageOptions {
    fn map_palette(&self, c: &Rgb<u8>) -> u8 {
        let mut min = 0;
        let mut min_dist = 0x7FFF_FFFF;
        for (i, p) in self.palette.iter().enumerate() {
            let dist = (c.0[0] as i32 - (p >> 16) as i32).pow(2)
                + (c.0[1] as i32 - ((p >> 8) & 0xFF) as i32).pow(2)
                + (c.0[2] as i32 - (p & 0xFF) as i32).pow(2);
            if dist < min_dist {
                min_dist = dist;
                min = i;
            }
        }
        min as u8
    }
}

impl Parse for MonochromeImageOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut opts = MonochromeImageOptions {
            image: "".to_string(),
            palette: vec![0x000000, 0xFFFFFF, 0xFF0000],
            channel: 0,
        };

        let name: Lit = input.parse()?;

        let image = if let Lit::Str(image) = &name {
            image.value()
        } else {
            return Err(syn::Error::new_spanned("image", "expected a string literal"));
        };
        opts.image = image;

        while let Ok(_) = input.parse::<Token![,]>() {
            if input.is_empty() {
                break;
            }

            let name: Ident = input.parse()?;

            match &*name.to_string() {
                "channel" => {
                    input.parse::<Token![=]>()?;
                    let channel: Lit = input.parse()?;

                    let channel = if let Lit::Int(channel) = &channel {
                        channel.base10_parse()?
                    } else {
                        return Err(syn::Error::new_spanned(
                            channel,
                            "expected a integer literal",
                        ));
                    };

                    opts.channel = channel;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        name,
                        "expected `palette` or `channel`",
                    ));
                }
            }
        }

        Ok(opts)
    }
}

struct BWR;

impl BWR {
    fn map_palette(&self, c: &Rgb<u8>) -> u8 {
        let palette = vec![0x000000, 0xFFFFFF, 0xFF0000];
        let mut min = 0;
        let mut min_dist = 0x7FFF_FFFF;
        for (i, p) in palette.iter().enumerate() {
            let dist = (c.0[0] as i32 - (p >> 16) as i32).pow(2)
                + (c.0[1] as i32 - ((p >> 8) & 0xFF) as i32).pow(2)
                + (c.0[2] as i32 - (p & 0xFF) as i32).pow(2);
            if dist < min_dist {
                min_dist = dist;
                min = i;
            }
        }
        min as u8
    }
}

impl image::imageops::colorops::ColorMap for BWR {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let palette = vec![0x000000, 0xFFFFFF, 0xFF0000];
        let mut min = 0;
        let mut min_dist = 0x7FFF_FFFF;
        for (i, p) in palette.iter().enumerate() {
            let dist = (color.0[0] as i32 - (p >> 16) as i32).pow(2)
                + (color.0[1] as i32 - ((p >> 8) & 0xFF) as i32).pow(2)
                + (color.0[2] as i32 - (p & 0xFF) as i32).pow(2);
            if dist < min_dist {
                min_dist = dist;
                min = i;
            }
        }
        min
    }
    fn map_color(&self, color: &mut Self::Color) {
        let idx = self.index_of(color);
        let palette =
            [
                Rgb([0x00, 0x00, 0x00]),
                Rgb([0xFF, 0xFF, 0xFF]),
                Rgb([0xFF, 0x00, 0x00]),
            ];
        *color = palette[idx];
    }
}

#[proc_macro]
pub fn monochrome_image(input: TokenStream) -> TokenStream {
    let opts = parse_macro_input!(input as MonochromeImageOptions);
    println!("text_image: {:#?}", opts);

    let im = image::open(&opts.image).expect("Can not read image file");
    let (mut w, h) = im.dimensions();

    let mut im = im.to_rgb8();

    // Floyd-Steinberg dithering
    image::imageops::colorops::dither(&mut im, &BWR);

    let mut ret = vec![];

    // convert each 8 pixel to a compressed byte
    for (y, row) in im.enumerate_rows() {
        let mut n = 0u8;
        for (x, (_, _, px)) in row.enumerate() {
            println!("{}x{}: {:?}", x, y, px);
            let ix = BWR.map_palette(px);
            if ix == opts.channel {
                n |= 1 << (7 - x % 8);
            }
            if x % 8 == 7 {
                println!("=> {}", n);
                ret.push(n);
                n = 0;
            }
        }
        if w % 8 != 0 {
            println!("=> {}", n);
            ret.push(n);
        }
    }

    w = (w / 8 + if w % 8 != 0 { 1 } else { 0 }) * 8;

    let raw_bytes = Lit::ByteStr(LitByteStr::new(&ret, proc_macro2::Span::call_site()));

    let expanded = quote! {
        (#w, #h, #raw_bytes)
    };

    TokenStream::from(expanded)
}

struct BWYR;

impl BWYR {
    fn map_palette(&self, c: &Rgb<u8>) -> u8 {
        let palette = vec![0x000000, 0xFFFFFF, 0xFF0000, 0xFFFF00];
        let mut min = 0;
        let mut min_dist = 0x7FFF_FFFF;
        for (i, p) in palette.iter().enumerate() {
            let dist = (c.0[0] as i32 - (p >> 16) as i32).pow(2)
                + (c.0[1] as i32 - ((p >> 8) & 0xFF) as i32).pow(2)
                + (c.0[2] as i32 - (p & 0xFF) as i32).pow(2);
            if dist < min_dist {
                min_dist = dist;
                min = i;
            }
        }
        min as u8
    }
}

impl image::imageops::colorops::ColorMap for BWYR {
    type Color = Rgb<u8>;

    fn index_of(&self, color: &Self::Color) -> usize {
        let palette = vec![0x000000, 0xFFFFFF, 0xFFFF00, 0xFF0000];
        let mut min = 0;
        let mut min_dist = 0x7FFF_FFFF;
        for (i, p) in palette.iter().enumerate() {
            let dist = (color.0[0] as i32 - (p >> 16) as i32).abs()
                + (color.0[1] as i32 - ((p >> 8) & 0xFF) as i32).abs()
                + (color.0[2] as i32 - (p & 0xFF) as i32).abs();
            if dist < min_dist {
                min_dist = dist;
                min = i;
            }
        }
        min
    }
    fn map_color(&self, color: &mut Self::Color) {
        let idx = self.index_of(color);
        let palette = [
            Rgb([0x00, 0x00, 0x00]),
            Rgb([0xFF, 0xFF, 0xFF]),
            Rgb([0xFF, 0x00, 0x00]),
            Rgb([0xFF, 0xFF, 0x00]),
        ];
        *color = palette[idx];
    }
}

// for BWRY palette
#[proc_macro]
pub fn quadcolor_image(input: TokenStream) -> TokenStream {
    let opts = parse_macro_input!(input as MonochromeImageOptions);
    println!("text_image: {:#?}", opts);

    let im = image::open(&opts.image).expect("Can not read image file");
    let (w, h) = im.dimensions();

    let mut im = im.to_rgb8();

    // Floyd-Steinberg dithering
    image::imageops::colorops::dither(&mut im, &BWYR);

    let mut ret = vec![];

    for pixels in im.pixels().array_chunks::<4>() {
        let mut n = 0u8;
        for pix in pixels {
            let ix = BWYR.map_palette(pix);
            if ix != 0 && ix != 1 && ix != 2 {
                println!("ix => {}", ix);
            }
            n = (n << 2) | (ix & 0b11);
        }
        ret.push(n);
    }

    let raw_bytes = Lit::ByteStr(LitByteStr::new(&ret, proc_macro2::Span::call_site()));

    let expanded = quote! {
        (#w, #h, #raw_bytes)
    };

    TokenStream::from(expanded)
}
