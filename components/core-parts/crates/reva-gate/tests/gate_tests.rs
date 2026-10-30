use reva_gate::gate::Gate;
use reva_x86_base::layout::{GATE_BASE, GATE_END, GATE_STRIDE};

#[test]
fn gates_fit_in_their_region() {
    assert!(Gate::ALL.len() as u32 * GATE_STRIDE <= GATE_END - GATE_BASE);
    for (i, g) in Gate::ALL.iter().enumerate() {
        assert_eq!(*g as usize, i);
        assert_eq!(Gate::from_addr(g.addr()), Some(*g));
    }
    assert_eq!(Gate::from_addr(GATE_BASE + 1), None);
    assert_eq!(Gate::from_addr(GATE_END), None);
}
