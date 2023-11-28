use text_image::text_image;

use embedded_graphics::{image::ImageRaw, pixelcolor::Gray8};

fn main() {
    let (w, h, raw) = text_image!(
        text = "Hello, world!哈哈这样也行",
        font = "LXGWWenKaiScreen.ttf",
        font_size = 48.0,
    );

    // println!("=> {:?} {}x{}", raw, w, h);
    let raw_image = ImageRaw::<Gray8>::new(raw, w);

    println!("=> {:#?}", raw_image);

    println!("image size {}", w * h);
    println!("bytes size {}", raw.len());
}
