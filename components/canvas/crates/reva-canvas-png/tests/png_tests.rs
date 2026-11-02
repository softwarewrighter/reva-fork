use reva_canvas::canvas::Canvas;
use reva_canvas_png::png::CanvasPng;

#[test]
fn a_drawing_is_a_png_of_the_world_square() {
    let mut c = Canvas::new();
    c.forward(100.0);
    let png = c.png().expect("encodes");
    assert_eq!(&png[1..4], b"PNG");
    // IHDR: width and height, big-endian, after the 8-byte signature.
    assert_eq!(png[16..24], [0, 0, 3, 32, 0, 0, 3, 32]);
}

#[test]
fn an_empty_canvas_is_the_background() {
    let a = Canvas::new().png().expect("encodes");
    let mut c = Canvas::new();
    c.forward(100.0);
    assert_ne!(a, c.png().expect("encodes"), "the line changes the image");
}
