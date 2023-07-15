use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{Cursor, Write},
    path::PathBuf,
};

use base64::Engine;
use image::{io::Reader, ImageOutputFormat};
use resvg::Tree;
use usvg::{Options, TreeParsing, TreeTextToPath};

// Embedded files
const IMAGE_TEMPLATE: &'static str = std::include_str!("template.svg");
const FREESERIF: &[u8] = std::include_bytes!("freeserif.ttf");

/// Converts a .png to Base64
fn create_base64_image(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let mut buf = vec![];
    let img = Reader::open(path)?.decode()?;
    let writer = base64::engine::general_purpose::STANDARD;

    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageOutputFormat::Png)?;

    Ok(writer.encode(buf))
}

fn create_png_for_img(path: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    let base64 = create_base64_image(path)?;
    let img = format!("data:image/png;base64,{base64}");

    if let Some(code) = path.file_stem().and_then(|stem| stem.to_str()) {
        let image = IMAGE_TEMPLATE
            .replace("DATA_IMAGE_URL", &img)
            .replace("__CODE", code)
            .replace(
                "__MESSAGE",
                "This is a sample message. TODO: Replace this with something better",
            );

        // For debugging purposes
        if env::var("SVG_DEBUG").is_ok() {
            let mut filename = env::current_dir()?;
            filename.push("export");
            filename.push(format!("{}.svg", code));

            let mut debug_file = File::create(filename)?;
            write!(debug_file, "{}", image)?;
        }

        // Handle SVG information
        let mut font_db = usvg::fontdb::Database::new();
        font_db.load_font_data(FREESERIF.to_vec());

        let mut svg = usvg::Tree::from_str(&image, &Options::default())?;
        svg.convert_text(&font_db);

        let mut pixmap = resvg::tiny_skia::Pixmap::new(750, 600).expect("Could not create pixmap");
        Tree::from_usvg(&svg).render(usvg::Transform::default(), &mut pixmap.as_mut());

        // Create export filename
        let mut filename = env::current_dir()?;
        filename.push("export");
        filename.push(format!("{}.png", code));
        pixmap.save_png(&filename)?;

        Ok(filename)
    } else {
        Err(Box::<dyn Error>::from(
            "Could not find code, file needs to be named <CODE>.png",
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut current_dir = env::current_dir()?;
    current_dir.push("images");

    let readable_images = fs::read_dir(current_dir)?;

    for image in readable_images {
        let img = image?;

        if img.file_type()?.is_file() && img.path().extension().is_some_and(|ext| ext == "png") {
            println!("Creating image for {:?}", img.file_name());
            match create_png_for_img(&img.path()) {
                Ok(file) => {
                    println!("Successfuly created image {:?}", file);
                    continue;
                }
                Err(e) => {
                    eprintln!("{e}");
                    break;
                }
            }
        }
    }

    Ok(())
}
