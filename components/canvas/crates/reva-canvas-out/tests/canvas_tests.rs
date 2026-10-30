use reva_canvas::canvas::{
    Canvas, MAX_OPS_PER_TAKE, MAX_SEGMENTS, OP_CLEAR, OP_LINE, OP_TURTLE, OP_WORDS, Turtle,
};
use reva_canvas_out::ops::CanvasOps;
use reva_canvas_out::svg::{CanvasSvg, num};

#[test]
fn a_square_closes_exactly() {
    let mut c = Canvas::new();
    for _ in 0..4 {
        c.forward(100.0);
        c.right(90.0);
    }
    assert_eq!(c.segments().len(), 4);
    assert_eq!((c.turtle.x, c.turtle.y, c.turtle.heading), (0.0, 0.0, 0.0));
    assert_eq!(c.segments()[0].y2, 100.0); // north is up
    assert_eq!(c.segments()[1].x2, 100.0); // right turns clockwise
}

#[test]
fn pen_up_moves_without_drawing() {
    let mut c = Canvas::new();
    c.turtle.pen_down = false;
    c.forward(50.0);
    assert!(c.segments().is_empty());
    assert_eq!(c.turtle.y, 50.0);
}

#[test]
fn ops_stream_in_batches_and_resend() {
    let mut c = Canvas::new();
    c.clear();
    c.forward(10.0);
    let ops = c.take_ops();
    assert_eq!(ops.len(), 3 * OP_WORDS); // CLEAR, LINE, TURTLE
    assert_eq!(ops[0], OP_CLEAR);
    assert_eq!(ops[OP_WORDS], OP_LINE);
    assert_eq!(ops[2 * OP_WORDS], OP_TURTLE);
    assert!(!c.has_ops());
    assert!(c.take_ops().is_empty());
    c.resend();
    assert_eq!(c.take_ops().len(), 3 * OP_WORDS);
}

#[test]
fn batches_are_capped() {
    let mut c = Canvas::new();
    c.path((0..40_000).map(|i| f64::from(i % 4) * 90.0));
    let mut lines = 0;
    let mut takes = 0;
    while c.has_ops() {
        let ops = c.take_ops();
        assert!(ops.len() <= MAX_OPS_PER_TAKE * OP_WORDS);
        lines += ops.chunks(OP_WORDS).filter(|r| r[0] == OP_LINE).count();
        takes += 1;
    }
    assert_eq!(lines, 40_000);
    assert_eq!(takes, 3);
}

#[test]
fn the_drawing_has_a_limit() {
    let mut c = Canvas::new();
    c.turtle.step = 0.0;
    assert!(c.path(std::iter::repeat_n(0.0, MAX_SEGMENTS)));
    assert!(!c.step());
    assert_eq!(c.segments().len(), MAX_SEGMENTS);
}

#[test]
fn push_and_pop_restore_the_turtle() {
    let mut c = Canvas::new();
    c.push();
    c.forward(30.0);
    c.right(45.0);
    assert!(c.pop());
    assert_eq!(c.turtle, Turtle::default());
    assert!(!c.pop());
}

#[test]
fn svg_joins_runs_into_paths() {
    let mut c = Canvas::new();
    c.forward(100.0);
    c.right(90.0);
    c.forward(100.0);
    c.turtle.rgb = 0xff0000;
    c.forward(-50.0);
    let svg = c.svg();
    assert_eq!(svg.matches("<path").count(), 2);
    assert!(svg.contains("d=\"M0 0 L0 -100 L100 -100\""), "{svg}");
    assert!(svg.contains("stroke=\"#ff0000\""));
    assert_eq!(num(-0.001), "0");
    assert_eq!(num(1.5), "1.5");
}
