use codec::Decode;

use crate::Event;

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Pallet_1"]
struct UnitEvent;

#[derive(Clone, Debug, Decode, Event, PartialEq)]
#[pallet = "Pallet_2"]
struct SimpleEvent {
    f1: String,
    f2: u128,
}

impl SimpleEvent {
    pub fn new(f1: &str, f2: u128) -> Self {
        Self {
            f1: String::from(f1),
            f2,
        }
    }
}

#[derive(Clone, Debug, Decode, Event, PartialEq)]
#[pallet = "Pallet_3"]
struct ComplexEvent {
    f1: u32,
    f2: u32,
    #[event_match_ignore]
    f3: u32,
    #[event_match_ignore(default = "Ok(())")]
    result: Result<(), ()>,
}

impl ComplexEvent {
    pub fn new(f1: u32, f2: u32, f3: u32, result: Result<(), ()>) -> Self {
        Self { f1, f2, f3, result }
    }
}

#[test]
fn generates_kind_properly() {
    let event = UnitEvent {};
    assert_eq!(("Pallet_1", "UnitEvent"), event.kind());

    let event = SimpleEvent::new("", 0);
    assert_eq!(("Pallet_2", "SimpleEvent"), event.kind());

    let event = ComplexEvent::new(1, 2, 3, Ok(()));
    assert_eq!(("Pallet_3", "ComplexEvent"), event.kind());
}

#[test]
fn generates_matches_for_unit_event() {
    let event1 = UnitEvent {};
    let event2 = UnitEvent {};
    assert!(event1.matches(&event2));
}

#[test]
fn generates_matches_for_simple_event_positive_check() {
    let event1 = SimpleEvent::new("a", 10);
    let event2 = SimpleEvent::new("a", 10);

    assert!(event1.matches(&event2));
}

#[test]
fn generates_matches_for_simple_event_negative_check() {
    let event1 = SimpleEvent::new("a", 10);
    let event2 = SimpleEvent::new("a", 11);

    assert!(!event1.matches(&event2));
}

#[test]
fn generates_matches_for_complex_event_positive_check_with_identical_args() {
    let event1 = ComplexEvent::new(1, 2, 3, Ok(()));
    let event2 = ComplexEvent::new(1, 2, 3, Ok(()));

    assert!(event1.matches(&event2));
}

#[test]
fn generates_matches_for_complex_event_positive_check_with_almost_identical_args() {
    let event1 = ComplexEvent::new(1, 2, 3, Ok(()));
    let event2 = ComplexEvent::new(1, 2, 4, Ok(()));

    assert!(event1.matches(&event2));
}

#[test]
fn generates_matches_for_complex_event_negative_check() {
    let event1 = ComplexEvent::new(1, 2, 3, Ok(()));
    let event2 = ComplexEvent::new(0, 2, 3, Ok(()));

    assert!(!event1.matches(&event2));
}

#[test]
fn generates_constructor_for_unit_event() {
    let _ = UnitEvent::from_relevant_fields();
}

#[test]
fn generates_constructor_for_simple_event() {
    let event = SimpleEvent::from_relevant_fields(String::from("b"), 10);
    assert_eq!(event, SimpleEvent::new("b", 10));
}

#[test]
fn generates_constructor_for_complex_event() {
    let event = ComplexEvent::from_relevant_fields(1, 2);
    assert_eq!(event, ComplexEvent::new(1, 2, 0, Ok(())));
}
