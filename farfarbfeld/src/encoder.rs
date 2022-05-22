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
use crate::{Error, Result};
use std::io::Write;

/// A farbfeld encoder
#[derive(Debug)]
pub struct Encoder<W: Write>(pub W);

impl<W: Write> Encoder<W> {
    /// Encodes a image with `width`, `height` and `data` into a farbfeld.
    /// # Failures
    /// Returns a `Error::NotEnoughData` if the provided `data` slice is too short.
    pub fn encode(self, width: u32, height: u32, data: &[u8]) -> Result<()> {
        let mut w = self.0;
        let len = (width * height) as usize * 4;
        if data.len() < len {
            return Err(Error::NotEnoughData);
        }
        w.write_all(b"farbfeld")?;
        w.write(&width.to_be_bytes())?;
        w.write(&height.to_be_bytes())?;
        w.write_all(data)?;
        Ok(())
    }
}
