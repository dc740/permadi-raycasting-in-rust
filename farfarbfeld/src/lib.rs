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

//! # Related Links
//! * http://git.suckless.org/farbfeld/tree/FORMAT.
#![deny(unsafe_code)]
#![deny(trivial_casts, trivial_numeric_casts)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![deny(unused_extern_crates, unused_import_braces, unused_qualifications)]

use std::error;
use std::fmt;
use std::io;

mod decoder;
mod encoder;
#[cfg(test)]
mod tests;

pub use decoder::Decoder;
pub use encoder::Encoder;

/// Fixed size of farfbfeld headers
pub const HEADER_LEN: u64 = 8 + 4 + 4;

/// Result of an image decoding/encoding process
pub type Result<T> = ::std::result::Result<T, Error>;

/// An enumeration of decoding/encoding Errors
#[derive(Debug)]
pub enum Error {
    /// The Image is not formatted properly
    FormatError(String),

    /// Not enough data was provided to the Decoder
    /// to decode the image
    NotEnoughData,

    /// An I/O Error occurred while decoding the image
    IoError(io::Error),

    /// The end of the image has been reached
    ImageEnd,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::FormatError(ref e) => write!(fmt, "Format error: {}", e),
            &Error::NotEnoughData => write!(
                fmt,
                "Not enough data was provided to the \
                                                         Decoder to decode the image"
            ),
            &Error::IoError(ref e) => e.fmt(fmt),
            &Error::ImageEnd => write!(fmt, "The end of the image has been reached"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FormatError(..) => &"Format error",
            Error::NotEnoughData => &"Not enough data",
            Error::IoError(..) => &"IO error",
            Error::ImageEnd => &"Image end",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::IoError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}
