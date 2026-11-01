//! The image format's errors and reader.

/// Why an image did not load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    BadMagic,
    BadVersion(u32),
    Truncated,
    BadSegment { start: u32, len: u32 },
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::BadMagic => write!(f, "not a Reva vx86 image"),
            ImageError::BadVersion(v) => write!(f, "unsupported image version {v}"),
            ImageError::Truncated => write!(f, "image is truncated"),
            ImageError::BadSegment { start, len } => {
                write!(f, "image segment {start:#x}+{len:#x} is out of range")
            }
        }
    }
}

impl std::error::Error for ImageError {}

/// Reading an image's bytes in order.
pub struct Reader<'a> {
    pub b: &'a [u8],
    pub at: usize,
}

impl<'a> Reader<'a> {
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ImageError> {
        let end = self.at.checked_add(n).ok_or(ImageError::Truncated)?;
        let s = self.b.get(self.at..end).ok_or(ImageError::Truncated)?;
        self.at = end;
        Ok(s)
    }

    pub fn u32(&mut self) -> Result<u32, ImageError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
}
