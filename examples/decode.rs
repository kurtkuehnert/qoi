use image::ColorType;
use qoi_qoi::decode;
use std::fs;

fn main() {
    let input = [
        "dice",
        "kodim10",
        "kodim23",
        "qoi_logo",
        "testcard",
        "testcard_rgba",
        "wikipedia_008",
    ];

    for name in input.iter() {
        let data = fs::read(format!("images/input/{}.qoi", name)).unwrap();
        let output = decode(&data).unwrap();

        image::save_buffer(
            format!("images/output/{}.png", name),
            &output.data,
            output.width,
            output.height,
            if output.channels == 3 {
                ColorType::Rgb8
            } else {
                ColorType::Rgba8
            },
        )
        .unwrap();
    }
}
