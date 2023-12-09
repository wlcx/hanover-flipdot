#![no_std]

#[cfg(feature = "std")]
extern crate std;
use std::io::Write;
use std::boxed::Box;
use std::vec;

use core_error::Error;

use bitvec::prelude::*;
#[cfg(feature = "embedded-graphics")]
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

const HEX: &[u8; 16] = b"0123456789ABCDEF";

// Turn one byte into its two byte hexadecimal representation.
// Yeah, there are crates for this but it's so simple to do...
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
    /// Initialise a new Flipdot with the given width, height and address.
    /// The address is the value selected on the address rotary switch on the flipdot
    /// controller PCB.
    pub fn new(w: u32, h: u32, addr: u8) -> Self {
        assert!(addr < 16);
        Self {
            addr,
            w,
            h,
            framebuffer: bitvec![u8, Lsb0; 0; (w*h) as usize],
        }
    }

    pub fn get_packet_size(&self) -> usize {
        // One bit per pixel, plus addr, resolution and checksum bytes
        (((self.h * self.w)/8 + 3)
        * 2 // Encoded as hex (1 byte == 2 ascii hex bytes)
        + 2) // Plus start and end bytes
        as usize
    }

    /// Encode the current frame buffer contents into a serial protocol message.
    /// `buf` must have >= `get_packet_size()` of capacity.
    pub fn make_packet(&self, buf: &mut [u8]) {
        let mut i = 0;
        buf[i] = 0x2;
        i += 1;
        for b in [self.addr + 17, ((self.h * self.w) / 8) as u8]
        .iter()
        .chain(self.framebuffer.as_raw_slice().into_iter())
        .flat_map(byte_to_hex) {
            buf[i] = b;
            i+=1;
        }
        buf[i] = 0x03;
        let sum = buf[1..=i].iter().map(|x| *x as u64).sum::<u64>() as u8;
        i += 1;
        [buf[i], buf[i+1]] = byte_to_hex(&((sum ^ 0xFF).wrapping_add(1)));
    }

    #[cfg(feature = "std")]
    /// Encode the frame and write it to `writer`
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

    fn write_vec(&self) -> vec::Vec<u8> {
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
        data
    }
}

#[cfg(feature = "embedded-graphics")]
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

#[cfg(feature = "embedded-graphics")]
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

    #[test]
    fn lol() {
        let fd = HanoverFlipdot::new(96, 16, 0);
        let one = fd.write_vec();
        assert!(one.len() == fd.get_packet_size());
        let mut two = vec!(0; 2048);
        fd.make_packet(&mut two);
        assert_eq!(one, two);
    }
}
