use std::path::Path;

use compression_image::QuadTree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open("./image.png")?;
    let img = img.as_rgba8().unwrap();
    let q = QuadTree::initialize(img);
    println!("{}", q.byte_size());
    q.average_compression();
    println!("{}", q.byte_size());
    q.generate_image(&Path::new("./image-traité.png"))?;

    Ok(())
}
