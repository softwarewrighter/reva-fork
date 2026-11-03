use reva_port::lex::{Kind, pieces};
use reva_port::migrate::migrate;
use reva_port::rules::{Rule, check_line};

fn rules(line: &str) -> Vec<Rule> {
    check_line(line.as_bytes()).iter().map(|f| f.rule).collect()
}

#[test]
fn lines_split_into_code_strings_and_comments() {
    let line = br#": t s" a b" type ( x -- ) ` w | rest"#;
    let kinds: Vec<(Kind, &[u8])> = pieces(line)
        .iter()
        .map(|p| (p.kind, &line[p.start..p.end]))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (Kind::Code, b":".as_slice()),
            (Kind::Code, b"t"),
            (Kind::Code, b"s\""),
            (Kind::Str, b"a b"),
            (Kind::Code, b"type"),
            (Kind::Comment, b"( x -- )"),
            (Kind::Str, b"w"),
            (Kind::Comment, b"| rest"),
        ]
    );
}

#[test]
fn a_backslash_given_to_a_splitting_word_is_a_separator() {
    assert_eq!(rules(r#"name s" \" ss -1 _at"#), vec![Rule::PathSeparator]);
    assert_eq!(rules(r#"name ` \ toksplt"#), vec![Rule::PathSeparator]);
    // backslashes in word names, banners and converters are not paths
    assert!(rules(r#": cut\ _i ; ." \/ ops \/ " cr"#).is_empty());
    assert!(rules(r#": \>/ ` \ ` / ,L ssr ;"#).is_empty());
}

#[test]
fn drives_in_code_and_strings_but_not_comments() {
    let f = check_line(br#": job ` C:/apps/data/base.db F> ;"#);
    assert_eq!(f.len(), 1);
    assert_eq!(
        (f[0].rule, f[0].fix.as_deref()),
        (Rule::DrivePath, Some(b"./base.db".as_slice()))
    );
    assert_eq!(
        rules(r#"" D:\work\notes.txt" open/r"#),
        vec![Rule::DrivePath]
    );
    assert!(rules(r#"| was in C:\apps\lib"#).is_empty());
}

#[test]
fn shell_commands_need_a_person() {
    let f = check_line(br#"s" del " name cL shell^"#);
    assert_eq!(
        (f[0].rule, f[0].fix.is_none()),
        (Rule::WindowsCommand, true)
    );
    assert!(f[0].advice.contains("delete"));
    // a drive inside the command goes with it, not fixed alone
    assert_eq!(
        rules(r#"s" start C:\Windows\cmd.exe " shell^"#),
        vec![Rule::WindowsCommand]
    );
    // the same words without the shell are not commands
    assert!(rules(r#"s" type " type"#).is_empty());
    assert_eq!(rules(r#"s" run.bat " start"#), vec![Rule::BatFile]);
}

#[test]
fn copies_keep_line_ends_and_change_only_the_fixes() {
    let src = b"a s\" \\\" ss\r\n| C:\\x\r\nb ` C:\\d\\f.txt\n";
    let m = migrate(src);
    assert_eq!(m.fixed, b"a s\" /\" ss\r\n| C:\\x\r\nb ` ./f.txt\n");
    let lines: Vec<usize> = m.findings.iter().map(|(n, _)| *n).collect();
    assert_eq!((lines, m.manual()), (vec![1, 3], 0));
}
