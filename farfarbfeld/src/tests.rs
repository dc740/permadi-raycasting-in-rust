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
use std::io::Cursor;

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::HEADER_LEN;

#[test]
fn decode() {
    let buf = Cursor::new(IMAGE_DATA);
    let mut img = Decoder::new(buf).unwrap();
    let (w, h) = img.dimensions();
    let data = img.read_image().unwrap();
    assert_eq!(w, 3);
    assert_eq!(h, 3);
    assert_eq!(data, &IMAGE_DATA[HEADER_LEN as usize..])
}

#[test]
fn encode() {
    let mut buf: Vec<u8> = Vec::new();
    if let Err(e) = Encoder(&mut buf).encode(3, 3, &IMAGE_DATA[HEADER_LEN as usize..]) {
        panic!("{}", e)
    }
    assert_eq!(&buf[..], IMAGE_DATA)
}

pub const IMAGE_DATA: &'static [u8] =
    b"farbfeld\
      \x00\x00\x00\x03\
      \x00\x00\x00\x03\
      \xff\xff\x00\x00\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff\
      \x00\x00\x00\x00\xff\xff\xff\xff\x80\x00\x80\x00\x80\x00\x80\x00\x00\x00\xff\xff\x00\x00\xff\xff\
      \x00\x00\xff\xff\x00\x00\xff\xff\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff";
