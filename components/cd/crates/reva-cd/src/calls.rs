//! The CD functions Reva programs call (`lib/ui/cd`), on [`Cd`]: their
//! arguments are the `(call)`'s cells, first C argument first; strings and
//! bitmaps are read through [`Memory`]; what a function returns through
//! pointers comes back as writes for the caller to make.

use crate::ops::{draw, rgb};
use crate::state::{Bitmap, Canvas, Cd};

/// The CD functions served.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CdFn {
    ContextIup,
    CreateCanvas,
    KillCanvas,
    Activate,
    ActiveCanvas,
    Clear,
    Foreground,
    Background,
    LineStyle,
    LineWidth,
    InteriorStyle,
    MarkType,
    MarkSize,
    Font,
    Line,
    Box,
    Rect,
    Mark,
    Pixel,
    Begin,
    Vertex,
    End,
    Text,
    VectorText,
    CanvasVectorText,
    EncodeColor,
    GetCanvasSize,
    InitBitmap,
    PutBitmap,
    KillBitmap,
    /// Returns 0 and does nothing (flush, text direction, versions ...).
    Zero,
}

/// The VM's memory, as CD calls need it.
pub trait Memory {
    /// `n` bytes at `a`, or `None` outside memory.
    fn bytes(&self, a: u32, n: u32) -> Option<Vec<u8>>;
}

/// What a call returns, and the writes it asks for: (address, bytes).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Done {
    pub value: u32,
    pub writes: Vec<(u32, Vec<u8>)>,
}

/// CD_QUERY: ask for an attribute without setting it.
const QUERY: u32 = u32::MAX;

impl Cd {
    /// Run CD function `f` with arguments `a`.
    pub fn call(&mut self, f: CdFn, a: &[u32], mem: &dyn Memory) -> Done {
        let arg = |i: usize| a.get(i).copied().unwrap_or(0);
        let value = match f {
            CdFn::ContextIup => 0x4344_0000,
            CdFn::CreateCanvas => {
                let n = self.number();
                self.canvases.insert(n, Canvas::new(arg(1)));
                n
            }
            CdFn::KillCanvas => {
                self.canvases.remove(&arg(0));
                self.active = if self.active == arg(0) {
                    0
                } else {
                    self.active
                };
                0
            }
            CdFn::Activate if arg(0) == 0 || self.canvases.contains_key(&arg(0)) => {
                self.active = arg(0);
                0
            }
            CdFn::Activate => u32::MAX,
            CdFn::ActiveCanvas => self.active,
            CdFn::EncodeColor => (arg(0) & 255) << 16 | (arg(1) & 255) << 8 | (arg(2) & 255),
            CdFn::GetCanvasSize => return self.canvas_size(a),
            CdFn::InitBitmap => self.init_bitmap(a, mem),
            CdFn::KillBitmap => {
                self.bitmaps.remove(&arg(0));
                0
            }
            CdFn::Zero => 0,
            _ => self.on_active(f, a, mem),
        };
        Done {
            value,
            writes: Vec::new(),
        }
    }

    /// An attribute or a drawing on the active canvas (0 with none).
    fn on_active(&mut self, f: CdFn, a: &[u32], mem: &dyn Memory) -> u32 {
        let n = |i: usize| a.get(i).copied().unwrap_or(0) as i32;
        let text = |i: usize| zstr(mem, a.get(i).copied().unwrap_or(0));
        let bitmap = self.bitmaps.get(&a.first().copied().unwrap_or(0)).cloned();
        let canvas = match f {
            CdFn::CanvasVectorText => self.canvases.get_mut(&a.first().copied().unwrap_or(0)),
            _ => self.canvases.get_mut(&self.active),
        };
        let Some(c) = canvas else { return 0 };
        let set = |v: &mut i32, x: i32| if x == -1 { *v } else { std::mem::replace(v, x) };
        let old = match f {
            CdFn::Foreground if a.first() != Some(&QUERY) => {
                std::mem::replace(&mut c.foreground, rgb(n(0))) as i32
            }
            CdFn::Foreground => c.foreground as i32,
            CdFn::Background if a.first() != Some(&QUERY) => {
                std::mem::replace(&mut c.background, rgb(n(0))) as i32
            }
            CdFn::Background => c.background as i32,
            CdFn::LineStyle => set(&mut c.line_style, n(0)),
            CdFn::LineWidth => set(&mut c.line_width, n(0)),
            CdFn::InteriorStyle => set(&mut c.interior, n(0)),
            CdFn::MarkType => set(&mut c.mark_type, n(0)),
            CdFn::MarkSize => set(&mut c.mark_size, n(0)),
            CdFn::Font => set(&mut c.text_size, n(2).abs()),
            _ => return draw(c, f, &n, text, bitmap),
        };
        old as u32
    }

    /// `cdGetCanvasSize(int *w, int *h, double *w_mm, double *h_mm)`.
    /// The four are the last cells, so a declaration with a cell more (as
    /// `lib/ui/cd` had: the canvas form's count) still writes them.
    fn canvas_size(&self, a: &[u32]) -> Done {
        let a = &a[a.len().saturating_sub(4)..];
        let element = self.canvases.get(&self.active).map_or(0, |c| c.element);
        let (w, h) = self.size(element);
        let mm = |px: i32| (px as f64 * 25.4 / 96.0).to_le_bytes().to_vec();
        let all = [
            (0, (w as u32).to_le_bytes().to_vec()),
            (1, (h as u32).to_le_bytes().to_vec()),
            (2, mm(w)),
            (3, mm(h)),
        ];
        let writes = all
            .into_iter()
            .filter_map(|(i, b)| a.get(i).filter(|p| **p != 0).map(|p| (*p, b)))
            .collect();
        Done { value: 0, writes }
    }

    /// `cdInitBitmap(w, h, type, ...)`: CD_RGB (r, g, b planes) or CD_MAP
    /// (an index per pixel and a 256-colour table); a new bitmap's number.
    fn init_bitmap(&mut self, a: &[u32], mem: &dyn Memory) -> u32 {
        let arg = |i: usize| a.get(i).copied().unwrap_or(0);
        let (w, h) = (arg(0) as i32, arg(1) as i32);
        let n = (w.max(0) * h.max(0)) as u32;
        let read = |p: u32, len: u32| mem.bytes(p, len).unwrap_or_else(|| vec![0; len as usize]);
        let pixels = match arg(2) {
            1 => {
                let index = read(arg(3), n);
                let colors = read(arg(4), 256 * 4);
                let color = |i: u8| {
                    u32::from_le_bytes(
                        colors[4 * i as usize..4 * i as usize + 4]
                            .try_into()
                            .unwrap(),
                    ) & 0xff_ffff
                };
                index.into_iter().map(color).collect()
            }
            _ => {
                let (r, g, b) = (read(arg(3), n), read(arg(4), n), read(arg(5), n));
                (0..n as usize)
                    .map(|i| (r[i] as u32) << 16 | (g[i] as u32) << 8 | b[i] as u32)
                    .collect()
            }
        };
        let id = self.number();
        self.bitmaps.insert(id, Bitmap { w, h, pixels });
        id
    }
}

/// The C string at `a` (up to 4 KiB), Latin-1.
fn zstr(mem: &dyn Memory, a: u32) -> String {
    let bytes = mem
        .bytes(a, 4096)
        .or_else(|| mem.bytes(a, 256))
        .unwrap_or_default();
    bytes
        .iter()
        .take_while(|b| **b != 0)
        .map(|b| *b as char)
        .collect()
}
