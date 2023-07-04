use crate::cli::CompressCommand;
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ImageEncoder, RgbaImage,
};
use imagequant::RGBA;
use oxipng::{optimize, InFile, Options, OutFile};
use std::{
    fs::{self, read_dir},
    path::{Path, PathBuf},
    rc::Rc,
    vec,
};
struct PngCompressMeta {
    input_path: PathBuf,
    output_path: PathBuf,
}

pub fn run_compress(compress_opts: CompressCommand) {
    let input_dir: Rc<PathBuf> = Rc::from(compress_opts.input_dir);
    let output_dir: Rc<PathBuf> = Rc::from(compress_opts.output_dir);
    let options = Options::from_preset(compress_opts.level as u8);

    let targets = if let Some(file_name) = compress_opts.file_name {
        let mut input_dir = Rc::clone(&input_dir).to_path_buf();
        let mut output_dir = Rc::clone(&output_dir).to_path_buf();

        input_dir.push(&file_name);
        output_dir.push(&file_name);

        let ext = input_dir
            .extension()
            .unwrap_or_else(|| panic!("png 파일 확장자를 명시해주세요"));

        if ext != "png" {
            panic!("png 파일만 압축할 수 있습니다")
        }
        vec![PngCompressMeta {
            input_path: input_dir,
            output_path: output_dir,
        }]
    } else {
        read_dir(input_dir.to_path_buf())
            .unwrap_or_else(|e| panic!("이미지 디렉토리 읽기 실패:\n{e:?}"))
            .filter_map(|result_entry| result_entry.ok())
            .map(|entry| entry.file_name())
            .filter_map(|file_name| {
                let ext = Path::new(&file_name)
                    .extension()
                    .and_then(|os_str| os_str.to_str());

                match ext {
                    Some("png") => Some(
                        file_name
                            .to_str()
                            .unwrap_or_else(|| {
                                panic!("파일명을 str로 형변환 하는 과정에서 실패했습니다")
                            })
                            .to_string(),
                    ),
                    _ => None,
                }
            })
            .map(|png_file_name| {
                let mut input_dir = Rc::clone(&input_dir).to_path_buf();
                let mut output_dir = Rc::clone(&output_dir).to_path_buf();

                input_dir.push(&png_file_name);
                output_dir.push(&png_file_name);

                PngCompressMeta {
                    input_path: input_dir,
                    output_path: output_dir,
                }
            })
            .collect::<Vec<PngCompressMeta>>()
    };

    for PngCompressMeta {
        input_path,
        output_path,
    } in targets
    {
        if compress_opts.drop_color {
            let png = image::open(&input_path)
                .unwrap_or_else(|e| panic!("PNG 파일 열기에 실패했습니다:\n{e:?}"))
                .to_rgba8();
            let (width, height) = png.dimensions();
            let bitmap: Vec<RGBA> = png
                .pixels()
                .map(|p| RGBA {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: p[3],
                })
                .collect();

            let mut img_q = imagequant::new();

            let mut described_bitmap = img_q
                // 정확한 이해가 없지만 그냥 gamma는 0.0쓰면 된다고 new_image 메서드 설명에서 나와있습니다.
                .new_image(&bitmap[..], width as usize, height as usize, 0.0)
                .unwrap_or_else(|e| panic!("비트맵 describe에 실패했습니다:\n{e:?}"));

            img_q
                .set_speed(compress_opts.speed as i32)
                .unwrap_or_else(|e| {
                    panic!("image quant 압축 시도 중 압축 속도 설정에 실패했습니다:\n{e:?}",)
                });

            img_q
                .set_quality(0, compress_opts.quality as u8)
                .unwrap_or_else(|e| {
                    panic!("quantize를 실행할 quality 설정에 실패했습니다:\n{e:?}")
                });

            let mut qt_result = match img_q.quantize(&mut described_bitmap) {
                Ok(res) => res,
                Err(e) => panic!("quantize에 실패했습니다:\n{e:?}"),
            };

            // 부드러운 이미지 출력. 1.0이 최댓값. 대부분의 경우 1.0쓰면 된다고 합니다.
            qt_result.set_dithering_level(1.0).unwrap_or_else(|e| {
                panic!("image quant 압축 중 dithering 레벨 설정에 실패했습니다:\n{e:?}")
            });

            let (palette, pixels) = qt_result
                .remapped(&mut described_bitmap)
                .unwrap_or_else(|e| panic!("quantize result unwrap을 실패했니다:\n{e:?}",));

            let mut new_png: RgbaImage = RgbaImage::new(width, height);

            for (i, pixel) in new_png.pixels_mut().enumerate() {
                let color = palette[pixels[i] as usize];
                *pixel = image::Rgba([color.r, color.g, color.b, color.a]);
            }

            let output_file = fs::File::create(&output_path)
                .unwrap_or_else(|e| panic!("파일 생성에 실패했습니다:\n{e:?}"));
            let png_encoder = PngEncoder::new_with_quality(
                output_file,
                CompressionType::Best,
                FilterType::Adaptive,
            );

            png_encoder
                .write_image(&new_png.into_raw(), width, height, image::ColorType::Rgba8)
                .unwrap_or_else(|e| {
                    panic!("png encoder에 png 데이터를 쓰는 도중 실패했습니다:\n{e:?}")
                });

            println!("손실 압축 🟢: {:?} -> {:?}", input_path, output_path)
        } else {
            let (in_file, out_file) = (InFile::Path(input_path), OutFile::Path(Some(output_path)));

            match optimize(&in_file, &out_file, &options) {
                Ok(_) => println!("무손실 압축 🟢: {in_file} -> {out_file:#?}"),
                Err(_) => {
                    eprintln!("무손실 압축🔴: {in_file}")
                }
            };
        }
    }
}
