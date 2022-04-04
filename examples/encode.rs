use image::ColorType;
use qoi_qoi::{decode, encode};
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
        let input = decode(&data).unwrap();

        let output = encode(&input);

        assert_eq!(data, output);

        fs::write(format!("images/output/{}.qoi", name), &output);
    }
}
