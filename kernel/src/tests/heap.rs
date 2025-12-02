pub fn test_heap_allocations() {
    use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};

    // 1) Basic Box allocation
    let heap_value = Box::new(1234);
    assert_eq!(*heap_value, 1234);

    // 2) Vector test
    let mut vec = Vec::new();
    for i in 0..100 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 100);
    assert_eq!(vec.iter().sum::<u64>(), 4950);

    // 3) String allocation
    let s = String::from("hello heap!");
    assert_eq!(s, "hello heap!");

    // 4) Reference counting test
    let rc_a = Rc::new(vec![1, 2, 3]);
    let rc_b = rc_a.clone();
    assert_eq!(Rc::strong_count(&rc_a), 2);
    assert_eq!(&*rc_a, &*rc_b);
}
