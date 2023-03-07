/*  Farbfeld is a simple image encoding format from suckless.
    Copyright (C) 2021  Emilio Moretti <emilio.moretti@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::{Error, Result, HEADER_LEN};
use std::convert::AsMut;
use std::io::{Read, Seek, SeekFrom};
/// A farbfeld decoder
#[derive(Debug)]
pub struct Decoder<R> {
    r: R,
    width: u32,
    height: u32,
}

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

impl<R: Read + Seek> Decoder<R> {
    /// Create a new decoder from `r` and parse the header.
    /// # Failures
    /// Returns Error::FormatError if the magic number does not match `farbfeld`
    pub fn new(mut r: R) -> Result<Decoder<R>> {
        let head = &mut [0; HEADER_LEN as usize];
        r.seek(SeekFrom::Start(0))?;
        r.read_exact(head)?;
        if &head[..8] != b"farbfeld" {
            return Err(Error::FormatError("unexpected magic number".to_string()));
        }

        Ok(Decoder {
            r,
            width: u32::from_be_bytes(clone_into_array(&head[8..12])),
            height: u32::from_be_bytes(clone_into_array(&head[12..16])),
        })
    }

    /// Returns the `(width, height)` of the image.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the length in bytes for a row.
    pub fn row_len(&self) -> usize {
        self.width as usize * 8
    }

    /// Read a single row from the image and return the bytes read.
    /// # Failures
    /// Returns a `Error::ImageEnd` if the `row` is greater as the `height`
    pub fn read_row(&mut self, row: u32, buf: &mut [u8]) -> Result<usize> {
        if row > self.height {
            return Err(Error::ImageEnd);
        }

        let row_len = self.row_len();
        let offset = HEADER_LEN + row as u64 * row_len as u64;
        self.r.seek(SeekFrom::Start(offset))?;
        self.r.read_exact(&mut buf[..row_len])?;
        Ok(row_len)
    }

    /// Read whole image into a `Vec<u8>`.
    pub fn read_image(&mut self) -> Result<Vec<u8>> {
        self.r.seek(SeekFrom::Start(HEADER_LEN))?;
        let num_raw_bytes = self.height as usize * self.row_len();
        let mut buf = vec![0; num_raw_bytes];
        self.r.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::IMAGE_DATA;
    use crate::Decoder;
    use crate::Error;
    use std::io::{Cursor, ErrorKind, Write};

    #[test]
    fn invalid_magic() {
        let mut img_data = Vec::new();
        img_data.write(b"test fail").unwrap();
        img_data.write(&IMAGE_DATA[8..]).unwrap();
        let buf = Cursor::new(img_data);

        match Decoder::new(buf) {
            Err(e) => match e {
                Error::FormatError(_) => return,
                e => panic!("{:?}", e),
            },
            Ok(_) => panic!("Got Ok expected Error::FormatError"),
        }
    }

    #[test]
    fn truncate_header() {
        let buf = Cursor::new(&IMAGE_DATA[0..8]);

        match Decoder::new(buf) {
            Err(Error::IoError(e)) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return;
                } else {
                    panic!("{:?}", e)
                }
            }
            Err(e) => panic!("{:?}", e),
            Ok(_) => panic!("Got Ok expected Error::FormatError"),
        }
    }

    #[test]
    fn truncate_data() {
        let buf = Cursor::new(&IMAGE_DATA[..IMAGE_DATA.len() - 1]);
        let mut img = Decoder::new(buf).unwrap();
        match img.read_image() {
            Err(Error::IoError(e)) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return;
                } else {
                    panic!("{:?}", e)
                }
            }
            Err(e) => panic!("{:?}", e),
            Ok(_) => panic!("Got Ok expected Error::FormatError"),
        }
    }
}
