use crate::{cli::ResizeCommand, exit_with_error};
use fast_image_resize as fr;
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
    io::Reader as ImageReader,
    ColorType, ImageEncoder,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    num::NonZeroU32,
    path::{self, PathBuf},
};

#[derive(Debug)]
enum ImageType {
    Jpeg,
    Png,
}
#[derive(Debug)]
struct ImageMeta {
    file_name: String,
    work_dir: PathBuf,
    image_type: ImageType,
}

fn resize<'a>(
    work_dir: &mut PathBuf,
    file_name: &str,
    target_width: u32,
    target_height: u32,
) -> (fast_image_resize::Image<'a>, NonZeroU32, NonZeroU32) {
    work_dir.push(file_name);
    let img = ImageReader::open(work_dir).unwrap().decode().unwrap();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    let dst_width = NonZeroU32::new(target_width).unwrap();
    let dst_height = NonZeroU32::new(target_height).unwrap();
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    let mut dst_view = dst_image.view_mut();

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    (dst_image, dst_width, dst_height)
}

pub fn run_resize(resize_opts: ResizeCommand) {
    let work_dir = &resize_opts.input_dir;
    let targets: Vec<ImageMeta> = if let Some(file_name) = resize_opts.file_name {
        let ext = path::Path::new(&file_name)
            .extension()
            .and_then(|os_str| os_str.to_str());

        match ext {
            Some("jpg") | Some("jpeg") => vec![ImageMeta {
                file_name,
                work_dir: work_dir.to_owned(),
                image_type: ImageType::Jpeg,
            }],
            Some("png") => vec![ImageMeta {
                file_name,
                work_dir: work_dir.to_owned(),
                image_type: ImageType::Png,
            }],
            _ => exit_with_error!(
                "올바른 경로가 아니거나, jpeg 혹은 png 파일이 아닙니다. 파일명: {}",
                file_name
            ),
        }
    } else {
        std::fs::read_dir(&resize_opts.input_dir)
            .unwrap_or_else(|e| exit_with_error!("이미지 디렉토리 읽기 실패:\n{}", e))
            .map(|result_entry| result_entry.unwrap().file_name())
            .filter_map(|file_name| {
                let ext = path::Path::new(&file_name)
                    .extension()
                    .and_then(|os_str| os_str.to_str());

                match ext {
                    Some("jpg") | Some("jpeg") => Some(ImageMeta {
                        file_name: file_name.to_str().unwrap().to_string(),
                        work_dir: work_dir.to_owned(),
                        image_type: ImageType::Jpeg,
                    }),
                    Some("png") => Some(ImageMeta {
                        file_name: file_name.to_str().unwrap().to_string(),
                        work_dir: work_dir.to_owned(),
                        image_type: ImageType::Png,
                    }),
                    _ => None,
                }
            })
            .collect()
    };

    for image_meta in targets {
        let (dst_image, dst_width, dst_height) = resize(
            &mut image_meta.work_dir.to_owned(),
            image_meta.file_name.as_str(),
            resize_opts.width,
            resize_opts.height,
        );

        let mut result_buf = BufWriter::new(Vec::new());
        let result_buf = match image_meta.image_type {
            ImageType::Jpeg => {
                JpegEncoder::new(&mut result_buf)
                    .write_image(
                        dst_image.buffer(),
                        dst_width.get(),
                        dst_height.get(),
                        ColorType::Rgba8,
                    )
                    .unwrap();

                result_buf
            }
            ImageType::Png => {
                PngEncoder::new(&mut result_buf)
                    .write_image(
                        dst_image.buffer(),
                        dst_width.get(),
                        dst_height.get(),
                        ColorType::Rgba8,
                    )
                    .unwrap();

                result_buf
            }
        };

        let mut output_path = resize_opts.output_dir.to_owned();
        output_path.push(image_meta.file_name);

        let file = File::create(output_path).unwrap();
        let mut file_writer = BufWriter::new(file);
        file_writer
            .write_all(&result_buf.into_inner().unwrap())
            .unwrap();
    }
}
