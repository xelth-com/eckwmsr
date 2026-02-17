use eckwmsr::sync::vector_clock::{ClockRelation, VectorClock};

#[test]
fn test_vector_clock_causality() {
    let mut vc1 = VectorClock::new();
    vc1.increment("A");

    let mut vc2 = VectorClock::new();
    vc2.increment("A");

    assert_eq!(vc1.compare(&vc2), ClockRelation::Equal);

    vc2.increment("B");
    // vc1: {A:1}, vc2: {A:1, B:1} -> vc1 is Before vc2
    assert_eq!(vc1.compare(&vc2), ClockRelation::Before);
    assert_eq!(vc2.compare(&vc1), ClockRelation::After);

    vc1.increment("C");
    // vc1: {A:1, C:1}, vc2: {A:1, B:1} -> Concurrent
    assert_eq!(vc1.compare(&vc2), ClockRelation::Concurrent);
}

#[test]
fn test_vector_clock_merge() {
    let mut vc1 = VectorClock::new();
    vc1.0.insert("A".to_string(), 1);
    vc1.0.insert("B".to_string(), 2);

    let mut vc2 = VectorClock::new();
    vc2.0.insert("B".to_string(), 3);
    vc2.0.insert("C".to_string(), 1);

    vc1.merge(&vc2);

    assert_eq!(vc1.get("A"), 1);
    assert_eq!(vc1.get("B"), 3); // Max(2,3)
    assert_eq!(vc1.get("C"), 1);
}
