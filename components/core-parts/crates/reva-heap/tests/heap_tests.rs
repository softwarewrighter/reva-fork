use reva_heap::heap::Heap;

#[test]
fn alloc_free_reuse_and_coalesce() {
    let mut h = Heap::new(0x1000, 0x2000);
    let a = h.alloc(10).unwrap();
    let b = h.alloc(10).unwrap();
    let c = h.alloc(10).unwrap();
    assert_eq!((a, b, c), (0x1000, 0x1010, 0x1020));
    assert!(h.free(a));
    assert!(h.free(b));
    assert_eq!(h.alloc(32).unwrap(), 0x1000); // a+b coalesced
    assert!(h.free(c));
    assert!(!h.free(c));
    assert!(h.alloc(0x2000).is_none());
}
