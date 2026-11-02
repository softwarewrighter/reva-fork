//! The drawing mode and what was written last.

/// `--draw`: what the CLI does with a drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// At a terminal: PNG shown inline where the terminal shows images,
    /// else SVG; nothing when output is redirected (scripts, goldens).
    #[default]
    Auto,
    Svg,
    Png,
    Off,
}

/// A file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Svg,
    Png,
}

/// The drawings of one run of the CLI.
#[derive(Debug, Default)]
pub struct Drawings {
    /// `None`: drawings are not written.
    pub format: Option<Format>,
    /// Show each PNG in the terminal too.
    pub inline: bool,
    /// The last drawing written (as SVG), so an unchanged one is not.
    pub last: Option<String>,
}

/// `--draw WHEN`.
pub fn parse_mode(s: &str) -> Result<Mode, String> {
    match s {
        "auto" => Ok(Mode::Auto),
        "svg" => Ok(Mode::Svg),
        "png" => Ok(Mode::Png),
        "off" => Ok(Mode::Off),
        w => Err(format!("--draw {w}: expected auto, svg, png or off")),
    }
}

/// The format and inline display for `mode`, given whether stdout is a
/// terminal and whether it shows images.
pub fn resolve(mode: Mode, terminal: bool, images: bool) -> (Option<Format>, bool) {
    let show = terminal && images;
    match mode {
        Mode::Auto if show => (Some(Format::Png), true),
        Mode::Auto if terminal => (Some(Format::Svg), false),
        Mode::Auto | Mode::Off => (None, false),
        Mode::Svg => (Some(Format::Svg), false),
        Mode::Png => (Some(Format::Png), show),
    }
}
