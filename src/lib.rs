#![feature(slice_take)]

mod decode;

pub use decode::decode;

use structview::{u32_be, View};

const MAGIC: &[u8] = b"qoif";
const STREAM_END: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 1];

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
pub(crate) struct QOIHeader {
    magic: [u8; 4],
    width: u32_be,
    height: u32_be,
    channels: u8,
    colorspace: u8,
}

pub(crate) type Pixel = [u8; 4];

#[inline]
pub(crate) fn hash(pixel: Pixel) -> usize {
    ((pixel[0]
        .wrapping_mul(3)
        .wrapping_add(pixel[1].wrapping_mul(5))
        .wrapping_add(pixel[2].wrapping_mul(7))
        .wrapping_add(pixel[3].wrapping_mul(11)))
        % 64) as usize
}
