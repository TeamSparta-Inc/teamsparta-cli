use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout};
use std::fs::File;
use std::io::Write;
use walkdir::WalkDir;
use webp::{Encoder, WebPMemory};

use crate::cli::WebpifyCommand;

fn is_jpeg_or_png(ext: &str) -> bool {
    ext == "png" || ext == "jpeg" || ext == "jpg"
}

pub fn run_webpify(webpify_opts: WebpifyCommand) {
    std::fs::create_dir_all(&webpify_opts.output_dir).unwrap();

    // Open path as DynamicImage
    let entries = WalkDir::new(webpify_opts.input_dir)
        .into_iter()
        .filter_map(|entry| entry.ok());
    // Put webp-image in a separate webp-folder in the location of the original image.

    entries.for_each(|file| {
        if file.file_type().is_file() {
            if let Some(ext_os_str) = file.path().extension() {
                let ext = ext_os_str.to_string_lossy();

                if is_jpeg_or_png(&ext) {
                    let image = ImageReader::open(file.path());
                    let dyn_image: DynamicImage = match image {
                        Ok(img) => img.with_guessed_format().unwrap().decode().unwrap(), //ImageReader::with_guessed_format() function guesses if image needs to be opened in JPEG or PNG format.
                        Err(e) => {
                            panic!("Error: {}", e);
                        }
                    };

                    // Make webp::Encoder from DynamicImage.
                    let encoder: Encoder = Encoder::from_image(&dyn_image).unwrap();
                    // Encode image into WebPMemory.
                    let encoded_webp: WebPMemory = encoder.encode(65f32);
                    // Get filename of original image.
                    let filename_original_image =
                        file.path().file_stem().unwrap().to_str().unwrap();
                    // Make full output path for webp-image.
                    let webp_image_path = format!(
                        "{}/{}.webp",
                        webpify_opts.output_dir.to_string_lossy(),
                        filename_original_image
                    );

                    // Make File-stream for WebP-result and write bytes into it, and save to path "output.webp".
                    let mut webp_image = File::create(webp_image_path).unwrap();

                    webp_image.write_all(encoded_webp.as_bytes()).unwrap();
                }
            }
        }
    });
}
