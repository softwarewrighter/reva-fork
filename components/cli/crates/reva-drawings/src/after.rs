//! After a run: write the drawing if it changed.

use std::io;
use std::path::Path;

use reva_canvas::canvas::Canvas;
use reva_canvas_out::svg::CanvasSvg;
use reva_canvas_png::png::CanvasPng;

use crate::drawings::{Drawings, Format, Mode, resolve};
use crate::inline::{images_inline, inline_image};
use crate::save::save_new;

/// What a run's drawing became: the file written, and the escape that
/// shows it inline (empty if not shown).
pub struct Written {
    pub name: String,
    pub inline: String,
}

impl Drawings {
    /// The drawings for `mode`; `terminal`: stdout is one.
    pub fn new(mode: Mode, terminal: bool) -> Drawings {
        let (format, inline) = resolve(mode, terminal, images_inline());
        Drawings {
            format,
            inline,
            last: None,
        }
    }

    /// Write `canvas` into `dir` if it has a drawing that changed since
    /// the last one written; `Ok(None)` if nothing was written.
    pub fn after(&mut self, canvas: &Canvas, dir: &Path) -> io::Result<Option<Written>> {
        let Some(format) = self.format else {
            return Ok(None);
        };
        if canvas.segments.is_empty() {
            self.last = None;
            return Ok(None);
        }
        let svg = canvas.svg();
        if self.last.as_ref() == Some(&svg) {
            return Ok(None);
        }
        let (ext, bytes) = match format {
            Format::Svg => ("svg", svg.clone().into_bytes()),
            Format::Png => (
                "png",
                canvas
                    .png()
                    .ok_or_else(|| io::Error::other("PNG encoding failed"))?,
            ),
        };
        let name = save_new(dir, "turtle", ext, &bytes)?;
        let inline = if self.inline && format == Format::Png {
            inline_image(&name, &bytes)
        } else {
            String::new()
        };
        self.last = Some(svg);
        Ok(Some(Written { name, inline }))
    }
}
