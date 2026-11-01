use reva_x86_base::layout::*;

#[test]
fn regions_are_ordered_and_disjoint() {
    let marks = [
        NULL_GUARD,
        GATE_END,
        STATIC_END,
        NPAD,
        TIB,
        TIB_TOP,
        APP_FILE_NAME,
        FZT_BUF,
        PARSE_BUF,
        DSTACK_LO,
        S0,
        DSTACK_HI,
        RP0,
        ARGV_END,
        D0,
        DTOP,
        H0,
        HTOP,
        MEM_SIZE,
    ];
    for w in marks.windows(2) {
        assert!(w[0] <= w[1], "{:#x} > {:#x}", w[0], w[1]);
    }
}
