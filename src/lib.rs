#![feature(slice_take)]

mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;

use structview::{u32_be, View};

const QOI_OP_INDEX: u8 = 0x00;
const QOI_OP_DIFF: u8 = 0x40;
const QOI_OP_LUMA: u8 = 0x80;
const QOI_OP_RUN: u8 = 0xC0;
const QOI_OP_RGB: u8 = 0xFe;
const QOI_OP_RGBA: u8 = 0xFF;

const QOI_MAGIC: &[u8] = b"qoif";
const QOI_END: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const INITIAL_PIXEL: [u8; 4] = [0, 0, 0, 255];

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseError {
    InvalidMagic,
    TruncatedFile,
}

#[derive(Clone, Copy, View, Debug)]
#[repr(C)]
struct QOIHeader {
    magic: [u8; 4],
    width: u32_be,
    height: u32_be,
    channels: u8,
    colorspace: u8,
}

type Pixel = [u8; 4];

#[inline]
fn hash(pixel: Pixel) -> usize {
    ((pixel[0]
        .wrapping_mul(3)
        .wrapping_add(pixel[1].wrapping_mul(5))
        .wrapping_add(pixel[2].wrapping_mul(7))
        .wrapping_add(pixel[3].wrapping_mul(11)))
        % 64) as usize
}
