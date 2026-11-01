//! The op stream: what a window draws, in batches.

use reva_canvas::canvas::{Canvas, MAX_OPS_PER_TAKE, OP_CLEAR, OP_LINE, OP_TURTLE, OP_WORDS};

reva_macros::ext_trait! {
    impl Canvas;
    /// The canvas as a stream of op records (see reva-canvas's module docs).
    pub trait CanvasOps {
        /// Hand out everything drawn since the last call next time (a window
        /// opened, or opened again).
        fn resend(&mut self) {
            self.sent = 0;
            self.cleared = true;
            self.moved = true;
        }

        /// Send the turtle's state with the next batch (its visibility changed).
        fn resend_turtle(&mut self) {
            self.moved = true;
        }

        /// Whether [`CanvasOps::take_ops`] has anything to give.
        fn has_ops(&self) -> bool {
            self.cleared || self.moved || self.sent < self.segments.len()
        }

        /// The ops since the last call, at most [`MAX_OPS_PER_TAKE`] records
        /// (call again while [`CanvasOps::has_ops`]); see the module docs.
        fn take_ops(&mut self) -> Vec<f32> {
            let mut ops = Vec::new();
            if !self.has_ops() {
                return ops;
            }
            let mut rec = |r: [f32; OP_WORDS]| ops.extend_from_slice(&r);
            if std::mem::take(&mut self.cleared) {
                rec([OP_CLEAR, self.bg as f32, 0., 0., 0., 0., 0., 0.]);
            }
            let end = self.segments.len().min(self.sent + MAX_OPS_PER_TAKE - 2);
            for s in &self.segments[self.sent..end] {
                rec([OP_LINE, s.x1, s.y1, s.x2, s.y2, s.rgb as f32, s.width, 0.]);
            }
            self.sent = end;
            let t = self.turtle;
            rec([
                OP_TURTLE,
                t.x as f32,
                t.y as f32,
                t.heading as f32,
                f32::from(u8::from(self.visible)),
                0.,
                0.,
                0.,
            ]);
            self.moved = false;
            ops
        }
    }
}
