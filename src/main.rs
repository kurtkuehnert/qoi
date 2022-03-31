use std::fs;
use structview::{u32_be, View};

#[derive(Clone, Copy, View, Debug)]
#[repr(C)]
struct QOIHeader {
  magic: [u8; 4],
  width: u32_be,
  height: u32_be,
  channels: u8,
  colorspace: u8,
}

struct Image {
  width: u32,
  height: u32,
  channels: u8,
  colorspace: u8,
  data: Vec<u8>,
}

#[inline]
fn hash(pixel: [u8; 4]) -> usize {
  (pixel[0] as usize * 3 + pixel[1] as usize * 5 + pixel[2] as usize * 7 + pixel[3] as usize * 9) % 64
}

#[inline]
fn apply_difference(byte: &mut u8, delta: i32) {
  *byte = ((*byte as i32 + delta) % 256) as u8;
}

fn decode(data: &[u8]) -> Image {
  let header = QOIHeader::view(&data[0..14]).unwrap();

  let channels = header.channels as usize;
  let size = header.width.to_int() as usize * header.height.to_int() as usize * channels;

  let mut output = Vec::with_capacity(size);

  let mut pixel: [u8; 4] = [0, 0, 0, 255];
  let mut pixel_cache: [[u8; 4]; 64] = [[0; 4]; 64];
  let mut index = 14;
  
  while output.len() != size {
        let byte0 = data[index];
        index += 1;

        if byte0 == 0xFE {
          pixel[0] = data[index + 0];
          pixel[1] = data[index + 1];
          pixel[2] = data[index + 2];
          index += 3;
          
          pixel_cache[hash(pixel)] = pixel;
          output.extend_from_slice(&pixel);
          continue;
        }

        if byte0 == 0xFF {
          pixel[0] = data[index + 0];
          pixel[1] = data[index + 1];
          pixel[2] = data[index + 2];
          pixel[3] = data[index + 3];
          index += 4;

          pixel_cache[hash(pixel)] = pixel;
          output.extend_from_slice(&pixel);
          continue;
        }

        match byte0 & 0xC0 {
          0x00 => {
            pixel = pixel_cache[(byte0 & 0x3F) as usize];
            output.extend_from_slice(&pixel);
          }
          0x40 => {
            let dr = (byte0 & 0x30 >> 4) as i32 - 2;
            let dg = (byte0 & 0x0C >> 2) as i32 - 2;
            let db = (byte0 & 0x03 >> 0) as i32 - 2;
            
            apply_difference(&mut pixel[0], dr);
            apply_difference(&mut pixel[1], dg);
            apply_difference(&mut pixel[2], db);

            pixel_cache[hash(pixel)] = pixel;
            output.extend_from_slice(&pixel);
          }
          0x80 => {
             let byte1 = data[index];
            index += 1;
            
            let dg = (byte0 & 0x3F >> 0) as i32 - 32;
            let dr = (byte1 & 0xF0 >> 4) as i32 - 8 + dg;
            let db = (byte1 & 0x0F >> 0) as i32 - 8 + dg;

            apply_difference(&mut pixel[0], dr);
            apply_difference(&mut pixel[1], dg);
            apply_difference(&mut pixel[2], db);

            pixel_cache[hash(pixel)] = pixel;
            output.extend_from_slice(&pixel);

          }
          0xC0 => {
            let run_length = (byte0 & 0x3F) as usize + 1;
            
            output.extend(pixel.iter().cycle().take(run_length * channels));
          }
          _ => unreachable!(),
        }
  }

  Image {
   width: header.width.to_int(),
   height: header.height.to_int(),
   channels: header.channels,
   colorspace: header.colorspace,
   data: output,
  }
}

fn main() {
  let data = fs::read("test.qoi").unwrap();

  let output = decode(&data);
  image::save_buffer("output.png", &output.data, output.width, output.height, image::ColorType::Rgba8);
}