use crate::{
    hash, Image, Pixel, QOIHeader, INITIAL_PIXEL, QOI_END, QOI_MAGIC, QOI_OP_DIFF, QOI_OP_LUMA,
    QOI_OP_RGB, QOI_OP_RGBA, QOI_OP_RUN,
};
use structview::{u32_be, View};

#[inline]
fn diff(r: u8, g: u8, b: u8) -> Option<u8> {
    let dr = r.wrapping_add(2);
    let dg = g.wrapping_add(2);
    let db = b.wrapping_add(2);

    match dr | dg | db {
        0..=0x03 => Some(QOI_OP_DIFF | dr << 4 | dg << 2 | db),
        _ => None,
    }
}

#[inline]
fn luma(r: u8, g: u8, b: u8) -> Option<[u8; 2]> {
    let dr = r.wrapping_add(8).wrapping_sub(g);
    let dg = g.wrapping_add(32);
    let db = b.wrapping_add(8).wrapping_sub(g);

    match (dr | db, dg) {
        (0..=0x0F, 0..=0x3F) => Some([QOI_OP_LUMA | dg, dr << 4 | db]),
        _ => None,
    }
}

pub fn encode(input: &Image) -> Vec<u8> {
    let size = input.width as usize * input.height as usize;
    let end = size - 1;

    let mut output = Vec::with_capacity(size);
    output.extend_from_slice(QOI_MAGIC);
    output.extend_from_slice(&input.width.to_be_bytes());
    output.extend_from_slice(&input.height.to_be_bytes());
    output.push(input.channels);
    output.push(input.colorspace);

    let mut previous_pixel = INITIAL_PIXEL;
    let mut pixel_cache = [[0; 4]; 64];
    let mut run_length = 0;

    for (position, pixel) in input
        .data
        .chunks_exact(input.channels as usize)
        .map(|pixel| {
            if input.channels == 4 {
                [pixel[0], pixel[1], pixel[2], pixel[3]]
            } else {
                [pixel[0], pixel[1], pixel[2], 255]
            }
        })
        .enumerate()
    {
        if pixel == previous_pixel {
            run_length += 1;

            if run_length == 62 || position == end {
                output.push(QOI_OP_RUN | run_length - 1);
                run_length = 0;
            }
            continue;
        }

        if run_length > 0 {
            output.push(QOI_OP_RUN | run_length - 1);
            run_length = 0;
        }

        let index = hash(pixel);

        if pixel_cache[index] == pixel {
            output.push(index as u8);
        } else {
            pixel_cache[index] = pixel;

            if pixel[3] == previous_pixel[3] {
                let dr = pixel[0].wrapping_sub(previous_pixel[0]);
                let dg = pixel[1].wrapping_sub(previous_pixel[1]);
                let db = pixel[2].wrapping_sub(previous_pixel[2]);

                if let Some(byte) = diff(dr, dg, db) {
                    output.push(byte);
                } else if let Some(bytes) = luma(dr, dg, db) {
                    output.extend_from_slice(&bytes);
                } else {
                    output.push(QOI_OP_RGB);
                    output.extend_from_slice(&pixel[0..3]);
                }
            } else {
                output.push(QOI_OP_RGBA);
                output.extend_from_slice(&pixel[0..4]);
            }
        }

        previous_pixel = pixel;
    }

    output.extend_from_slice(&QOI_END);

    output
}
