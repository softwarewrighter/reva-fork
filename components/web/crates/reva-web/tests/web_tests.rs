use reva_vm::version::BANNER;
use reva_web::*;

fn feed(s: &str) {
    let p = reva_input_buffer(s.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len()) };
    reva_feed(s.len() as u32);
}

fn output() -> String {
    let (p, n) = (reva_output_ptr(), reva_output_len());
    let s = unsafe { std::slice::from_raw_parts(p, n as usize) };
    let s = String::from_utf8_lossy(s).into_owned();
    reva_output_clear();
    s
}

#[test]
fn host_calls_go_to_the_page_and_back() {
    assert_eq!(reva_boot(), NEED_INPUT);
    output();
    feed("\" reva-sqlite\" lib rsql 2 func: rsql_open 1 func: rsql_errmsg 0 func: rsql_ptr\n");
    feed("\" a.db\" rsql_open .\n");
    let mut s = YIELDED;
    while s == YIELDED {
        s = reva_run(100_000);
    }
    assert_eq!(s, HOST_CALL);
    let json = unsafe {
        let p = reva_host_call_ptr();
        String::from_utf8_lossy(std::slice::from_raw_parts(p, reva_host_call_len() as usize))
            .into_owned()
    };
    assert_eq!(
        json,
        r#"{"service":"sqlite","op":"open","ints":[],"bytes":[[97,46,100,98]]}"#
    );
    reva_resume_int(3.0);
    assert_eq!(run_out(), "\nok> 3 \nok> ");
    // refused: the line stops with the page's reason
    feed("9 rsql_errmsg .\n");
    assert_eq!(run_until_host_call(), HOST_CALL);
    let why = b"SQLite is not available in the browser yet";
    let p = reva_fs_buffer(why.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(why.as_ptr(), p, why.len()) };
    reva_refuse(why.len() as u32);
    assert!(run_out().contains("SQLite is not available in the browser yet"));
}

fn run_until_host_call() -> i32 {
    let mut s = YIELDED;
    while s == YIELDED {
        s = reva_run(100_000);
    }
    s
}

#[test]
fn the_c_abi_drives_a_session() {
    assert_eq!(reva_run(10), NOT_BOOTED);
    assert_eq!(reva_boot(), NEED_INPUT);
    assert_eq!(output(), format!("{BANNER}\n\nok> "));
    feed("2 3 + . : sq dup * ; 7 sq .\n");
    let mut s = YIELDED;
    while s == YIELDED {
        s = reva_run(1000);
    }
    assert_eq!(s, NEED_INPUT);
    assert_eq!(output(), "5 49 \nok> ");
    // files: one added from outside, one written by Forth
    put("/home/a.f", b": greet .\" hi\" ; ");
    assert_eq!(reva_fs_changes(), 0);
    feed("include /home/a.f greet\n\" /home/b\" creat value h \" xyz\" h write h close\n");
    while reva_run(1000) == YIELDED {}
    assert_eq!(output(), "hi\nok> \nok> ");
    let n = reva_fs_changes() as usize;
    let rec = unsafe { std::slice::from_raw_parts(reva_fs_data(), n) };
    let mut want = vec![1u8, 7, 0, 0, 0];
    want.extend_from_slice(b"/home/b");
    want.extend_from_slice(&[3, 0, 0, 0]);
    want.extend_from_slice(b"xyz");
    assert_eq!(rec, &want[..]);
    // a runaway loop, stopped
    feed(": spin repeat again ; spin\n");
    assert_eq!(reva_run(10_000), YIELDED);
    reva_interrupt();
    while reva_run(1000) == YIELDED {}
    assert!(output().contains("*** interrupted (in spin"));
    // the clock the page sets
    reva_set_time(1_792_886_400.0, 0);
    feed("time&date . . .\n");
    while reva_run(1000) == YIELDED {}
    assert_eq!(output(), "2026 10 25 \nok> ");
    feed("3 ~sys.(bye)\n");
    while reva_run(1000) == YIELDED {}
    assert_eq!(reva_exit_code(), 3);
}

fn run_out() -> String {
    while reva_run(1000) == YIELDED {}
    output()
}

#[test]
fn images_and_restart() {
    // each test runs on its own thread, so this is a fresh module state
    assert_eq!(reva_boot(), NEED_INPUT);
    output();
    feed(": hi .\" from the image\" ; save /app.img\n");
    assert!(run_out().contains("Saving: /app.img"));
    let n = reva_fs_changes() as usize;
    let rec = unsafe { std::slice::from_raw_parts(reva_fs_data(), n) }.to_vec();
    let image = &rec[5 + 8 + 4..];
    // a fresh start from the image
    let p = reva_fs_buffer(image.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(image.as_ptr(), p, image.len()) };
    assert_eq!(reva_boot_image(image.len() as u32), NEED_INPUT);
    assert_eq!(output(), format!("{BANNER}\n\nok> "));
    feed(": gone ; hi restart\nhi gone\n");
    // restart: the banner again, the image's words, not later ones;
    // the next line was not read yet, so it reaches the new machine
    let out = run_out();
    assert!(
        out.starts_with(&format!("from the image{BANNER}\n\nok> from the image")),
        "{out:?}"
    );
    assert!(out.contains("unknown word:  gone"), "{out:?}");
    // not an image
    let p = reva_fs_buffer(4);
    unsafe { std::ptr::copy_nonoverlapping(b"nope".as_ptr(), p, 4) };
    assert_eq!(reva_boot_image(4), BAD_IMAGE);
    let why = unsafe { std::slice::from_raw_parts(reva_fs_data(), reva_fs_len() as usize) };
    assert_eq!(why, b"image is truncated");
}

#[test]
fn restart_reads_the_start_input_again() {
    assert_eq!(reva_boot(), NEED_INPUT);
    output();
    put("/app.f", b": hello .\" hello\" ; ");
    let start = "include /app.f\n";
    let p = reva_input_buffer(start.len() as u32);
    unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), p, start.len()) };
    reva_feed_start(start.len() as u32);
    feed(": gone ; restart\nhello gone\n");
    let out = run_out();
    assert!(out.contains("ok> hello"), "{out:?}");
    assert!(out.contains("unknown word:  gone"), "{out:?}");
}

fn canvas() -> Vec<f32> {
    let n = reva_canvas_take() as usize;
    unsafe { std::slice::from_raw_parts(reva_canvas_ptr(), n) }.to_vec()
}

#[test]
fn turtle_ops_reach_the_page() {
    assert_eq!(reva_boot(), NEED_INPUT);
    output();
    assert!(canvas().is_empty());
    feed("needs turtle/core : sq 4 0do 100 forward 90 right loop ; clear sq\n");
    run_out();
    let ops = canvas();
    let lines = ops.chunks(8).filter(|r| r[0] == 2.0).count();
    assert_eq!((ops[0], lines), (1.0, 4));
    assert!(canvas().is_empty());
    reva_canvas_resend();
    assert_eq!(canvas().len(), ops.len());
    assert_eq!(reva_canvas_reset(), 0);
    feed("restart\n");
    run_out();
    assert_eq!(reva_canvas_reset(), 1);
    assert_eq!(reva_canvas_reset(), 0);
    assert!(canvas().is_empty());
}

fn put(path: &str, data: &[u8]) {
    let p = reva_fs_buffer((path.len() + data.len()) as u32);
    unsafe {
        std::ptr::copy_nonoverlapping(path.as_ptr(), p, path.len());
        std::ptr::copy_nonoverlapping(data.as_ptr(), p.add(path.len()), data.len());
    }
    assert_eq!(reva_fs_put(path.len() as u32), 0);
}
