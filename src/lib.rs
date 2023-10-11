use std::{error::Error, io::Write};

use bitvec::prelude::*;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn byte_to_hex(b: &u8) -> [u8; 2] {
    [HEX[((*b & 0xf0) >> 4) as usize], HEX[(*b & 0xf) as usize]]
}

pub struct HanoverFlipdot {
    addr: u8,
    w: u32,
    h: u32,
    framebuffer: BitVec<u8, Lsb0>,
}

impl HanoverFlipdot {
    pub fn new(w: u32, h: u32, addr: u8) -> Self {
        assert!(addr < 16);
        Self {
            addr,
            w,
            h,
            framebuffer: bitvec![u8, Lsb0; 0; (w*h) as usize],
        }
    }
    pub fn write_frame<W>(&self, writer: &mut W) -> Result<(), Box<dyn Error>>
    where
        W: Write,
    {
        // Encode the address and resolution.
        // The address is the number on the address selector on the flipdot PCB + 17. For
        // some reason.
        let mut data = vec![0x2];
        data.extend(
            [self.addr + 17, ((self.h * self.w) / 8) as u8]
                .iter()
                .chain(self.framebuffer.as_raw_slice().into_iter())
                .flat_map(byte_to_hex),
        );
        data.push(0x03);
        let sum = data.iter().skip(1).map(|x| *x as u64).sum::<u64>() as u8;
        data.extend(byte_to_hex(&((sum ^ 0xFF).wrapping_add(1))));
        writer.write(&data)?;
        Ok(())
    }
}

impl DrawTarget for HanoverFlipdot {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x, y)) = <(u32, u32)>::try_from(coord) {
                if (0..self.w).contains(&x) && (0..self.h).contains(&y) {
                    let index: u32 = x * self.h + y;
                    self.framebuffer.set(index as usize, color.is_on());
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for HanoverFlipdot {
    fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b2h() {
        assert_eq!(byte_to_hex(&192), *b"C0");
        assert_eq!(byte_to_hex(&0), *b"00");
        assert_eq!(byte_to_hex(&255), *b"FF");
    }
}
