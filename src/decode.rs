use crate::{hash, Image, ParseError, Pixel, QOIHeader, QOI_END, QOI_MAGIC};
use std::mem::size_of;
use structview::View;

pub fn decode(data: &[u8]) -> Result<Image, ParseError> {
    let mut data = data;

    let header = data
        .take(..size_of::<QOIHeader>())
        .ok_or(ParseError::TruncatedFile)?;
    let header = QOIHeader::view(header).expect("Could not parse header.");

    let end = data
        .take(data.len() - QOI_END.len()..)
        .ok_or(ParseError::TruncatedFile)?;

    if header.magic != QOI_MAGIC {
        return Err(ParseError::InvalidMagic);
    }
    if end != QOI_END {
        return Err(ParseError::TruncatedFile);
    }

    let iter = PixelIter::new(data);

    let output = if header.channels as usize == 3 {
        iter.flat_map(|pixel| pixel.into_iter().take(3))
            .collect::<Vec<u8>>()
    } else {
        iter.flatten().collect::<Vec<u8>>()
    };

    Ok(Image {
        width: header.width.to_int(),
        height: header.height.to_int(),
        channels: header.channels,
        colorspace: header.colorspace,
        data: output,
    })
}

#[derive(Debug)]
pub struct PixelIter<'a> {
    pub pixel: [u8; 4],
    pub pixel_cache: [Pixel; 64],
    pub run_length: u8,
    pub data: &'a [u8],
}

impl<'a> PixelIter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            pixel: [0, 0, 0, 255],
            data,
            run_length: 0,
            pixel_cache: [[0; 4]; 64],
        }
    }

    #[inline]
    fn update_cache(&mut self) {
        self.pixel_cache[hash(self.pixel)] = self.pixel;
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.run_length > 0 {
            self.run_length -= 1;

            return Some(self.pixel);
        }

        let byte0 = self.data.take_first()?;

        match byte0 {
            0xFE => {
                // QOI_OP_RGB
                self.pixel[0..3].copy_from_slice(self.data.take(..3)?);
                self.update_cache()
            }
            0xFF => {
                // QOI_OP_RGBA
                self.pixel[0..4].copy_from_slice(self.data.take(..4)?);
                self.update_cache()
            }
            _ => match byte0 & 0xC0 {
                0x00 => {
                    // QOI_OP_INDEX
                    self.pixel = self.pixel_cache[(byte0 & 0x3F) as usize];
                }
                0x40 => {
                    // QOI_OP_DIFF
                    let dr = (byte0 >> 4 & 0x3).wrapping_sub(2);
                    let dg = (byte0 >> 2 & 0x3).wrapping_sub(2);
                    let db = (byte0 >> 0 & 0x3).wrapping_sub(2);

                    self.pixel[0] = self.pixel[0].wrapping_add(dr);
                    self.pixel[1] = self.pixel[1].wrapping_add(dg);
                    self.pixel[2] = self.pixel[2].wrapping_add(db);

                    self.update_cache()
                }
                0x80 => {
                    // QOI_OP_LUMA
                    let byte1 = self.data.take_first()?;

                    let dg = (byte0 & 0x3F).wrapping_sub(32);
                    let dr = (byte1 >> 4).wrapping_sub(8).wrapping_add(dg);
                    let db = (byte1 & 0xF).wrapping_sub(8).wrapping_add(dg);

                    self.pixel[0] = self.pixel[0].wrapping_add(dr);
                    self.pixel[1] = self.pixel[1].wrapping_add(dg);
                    self.pixel[2] = self.pixel[2].wrapping_add(db);

                    self.update_cache()
                }
                0xC0 => {
                    // QOI_OP_RUN
                    self.run_length = byte0 & 0x3F;
                }
                _ => unreachable!(),
            },
        }

        Some(self.pixel)
    }
}
