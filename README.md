# text-image

A Rust macro crate for converting text and images to various image formats, primarily for use with embedded graphics systems.

## Features

- Convert text to grayscale images with customizable options
- Convert color images to monochrome (1-bit) images
- Convert color images to 4-color (2-bit) images
- Convert images to grayscale with adjustable bit depth (1, 2, 4, or 8-bit)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
text-image = "0.1.0"
```

### Text to Image

Convert text to a grayscale image:

```rust
use text_image::text_image;
use embedded_graphics::{image::ImageRaw, pixelcolor::Gray8};
fn main() {
    let (w, h, raw) = text_image!(
        text = "Hello, world!哈哈这样也行",
        font = "LXGWWenKaiScreen.ttf",
        font_size = 48.0,
        inverse,
        Gray4,
    );
    let raw_image = ImageRaw::<Gray8>::new(raw, w);
}
```

### Image to Monochrome

Convert a color image to a 1-bit monochrome image:

```rust
use text_image::monochrome_image;
let (w, h, img_raw) = monochrome_image!("path/to/image.png", channel = 1);
```

### Image to 4-color

Convert a color image to a 2-bit 4-color image:

```rust
use text_image::quadcolor_image;
let (w, h, img_raw) = quadcolor_image!("path/to/image.png");
```

### Image to Grayscale

Convert an image to grayscale with specified bit depth:

```rust
use text_image::gray_image;
let (w, h, img_raw) = gray_image!("path/to/image.png", Gray4);
```

## Options

- `text`: The text to convert (required for `text_image!`)
- `font`: Path to the font file (required for `text_image!`)
- `font_size`: Font size in pixels (default: 16.0)
- `inverse`: Invert the colors (optional)
- `line_spacing`: Additional space between lines (optional)
- `Gray2`, `Gray4`, `Gray8`: Specify the bit depth for grayscale output

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
