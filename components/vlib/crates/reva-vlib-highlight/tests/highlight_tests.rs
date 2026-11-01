use reva_vlib_highlight::lex::tokens;
use reva_vlib_highlight::render::render;
use reva_vlib_highlight::table::Table;

const FORTH: &str = include_str!("../../../../../lib/highlight/forth.tbl");

fn classes(line: &str) -> Vec<(String, String)> {
    let mut t = Table::default();
    t.add(FORTH);
    tokens(line.as_bytes(), &t)
        .into_iter()
        .filter(|(c, a, b)| !c.is_empty() || !line[*a..*b].trim().is_empty())
        .map(|(c, a, b)| (c.to_string(), line[a..b].to_string()))
        .collect()
}

fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter()
        .map(|(c, s)| (c.to_string(), s.to_string()))
        .collect()
}

#[test]
fn definition_line() {
    assert_eq!(
        classes(": sq ( n -- n*n ) dup * ; | square"),
        pairs(&[
            ("colon", ":"),
            ("def", "sq"),
            ("stk", "( n -- n*n )"),
            ("stack", "dup"),
            ("stack", "*"),
            ("colon", ";"),
            ("com", "| square"),
        ])
    );
}

#[test]
fn strings_numbers_control() {
    assert_eq!(
        classes(r#"10 0do ." hi there" 'x emit $ff IF loop"#),
        pairs(&[
            ("num", "10"),
            ("ctl", "0do"),
            ("str", r#"." hi there""#),
            ("str", "'x"),
            ("", "emit"),
            ("num", "$ff"),
            ("ctl", "IF"),
            ("ctl", "loop"),
        ])
    );
}

#[test]
fn unterminated_runs_to_end_of_line() {
    assert_eq!(classes("( open"), pairs(&[("stk", "( open")]));
    assert_eq!(classes("\" open"), pairs(&[("str", "\" open")]));
}

#[test]
fn later_table_wins_and_render_wraps_tokens() {
    let mut t = Table::default();
    t.add(FORTH);
    t.add("style cosy 1;32 color:#98c379\nwords cosy dup rho\n");
    assert_eq!(tokens(b"dup", &t)[0].0, "cosy");
    let out = render(b"7 rho\nfoo", &t);
    assert_eq!(
        String::from_utf8(out).unwrap(),
        format!(
            "\x1b[{n}m7\x1b[0m \x1b[1;32mrho\x1b[0m\nfoo",
            n = t.sgr["num"]
        )
    );
}

#[test]
fn crlf_and_plain_bytes_are_kept() {
    let t = Table::default();
    let src = b"a  b\r\n\tc \xff\n";
    assert_eq!(render(src, &t), src.to_vec());
}
