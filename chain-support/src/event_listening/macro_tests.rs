use anyhow::Result as AnyResult;
use codec::Decode;

use types::*;

use crate::Event;

/// Internal type mocks representing those from Substrate world.
mod types {
    use codec::Decode;

    #[derive(Clone, Debug, Decode, Eq, PartialEq)]
    pub struct AccountId(pub String);

    #[derive(Clone, Debug, Decode, Default, Eq, PartialEq)]
    pub struct Timepoint {
        pub height: u32,
        pub index: u32,
    }

    pub type CallHash = [u8; 32];

    pub type DispatchResult = Result<(), String>;
}

#[derive(Clone, Debug, Decode, Event)]
#[pallet = "Utility"]
struct BatchCompleted;

#[derive(Clone, Debug, Decode, Event, PartialEq)]
#[pallet = "Balances"]
struct Transfer {
    from: AccountId,
    to: AccountId,
    amount: u128,
}

impl Transfer {
    pub fn new(from: &str, to: &str, amount: u128) -> Self {
        Self {
            from: AccountId(String::from(from)),
            to: AccountId(String::from(to)),
            amount,
        }
    }
}

#[derive(Clone, Debug, Decode, Event, PartialEq)]
#[pallet = "Multisig"]
struct MultisigExecuted {
    approving: AccountId,
    #[event_ignore]
    timepoint: Timepoint,
    multisig: AccountId,
    call_hash: CallHash,
    #[event_ignore = "Ok(())"]
    result: DispatchResult,
}

impl MultisigExecuted {
    pub fn new(
        approving: &str,
        timepoint: Timepoint,
        multisig: &str,
        call_hash: CallHash,
        result: DispatchResult,
    ) -> Self {
        Self {
            approving: AccountId(String::from(approving)),
            timepoint,
            multisig: AccountId(String::from(multisig)),
            call_hash,
            result,
        }
    }
}

#[test]
fn generates_kind_properly() -> AnyResult<()> {
    let event = BatchCompleted {};
    assert_eq!(("Utility", "BatchCompleted"), event.kind());

    let event = Transfer::new("", "", 0);
    assert_eq!(("Balances", "Transfer"), event.kind());

    let event = MultisigExecuted::new("", Default::default(), "", Default::default(), Ok(()));
    assert_eq!(("Multisig", "MultisigExecuted"), event.kind());

    Ok(())
}

#[test]
fn generates_matches_for_unit_struct() -> AnyResult<()> {
    let event = BatchCompleted {};
    let event2 = BatchCompleted {};
    assert!(event.matches(&event2));

    Ok(())
}

#[test]
fn generates_matches_for_normal_struct() -> AnyResult<()> {
    let event1 = Transfer::new("a", "b", 10);
    let event2 = Transfer::new("a", "b", 10);
    let event3 = Transfer::new("b", "b", 10);
    let event4 = Transfer::new("a", "b", 11);

    assert!(event1.matches(&event2));
    assert!(!event1.matches(&event3));
    assert!(!event1.matches(&event4));

    Ok(())
}

#[test]
fn generates_matches_for_struct_with_ignored_fields() -> AnyResult<()> {
    let event1 = MultisigExecuted::new("a", Default::default(), "b", Default::default(), Ok(()));
    let event2 = MultisigExecuted::new("a", Default::default(), "b", Default::default(), Ok(()));
    let event3 = MultisigExecuted::new("a", Default::default(), "a", Default::default(), Ok(()));
    let event4 = MultisigExecuted::new("a", Default::default(), "a", [1; 32], Ok(()));
    let event5 = MultisigExecuted::new(
        "a",
        Default::default(),
        "b",
        Default::default(),
        Err(String::new()),
    );
    let event6 = MultisigExecuted::new(
        "a",
        Timepoint {
            height: 1,
            index: 2,
        },
        "b",
        Default::default(),
        Ok(()),
    );

    assert!(event1.matches(&event2));
    assert!(event1.matches(&event2));
    assert!(event1.matches(&event5));
    assert!(event1.matches(&event6));

    assert!(!event1.matches(&event3));
    assert!(!event1.matches(&event4));

    Ok(())
}

#[test]
fn generates_constructor_for_unit_struct() -> AnyResult<()> {
    let _ = BatchCompleted::from_relevant_fields();
    Ok(())
}

#[test]
fn generates_constructor_for_normal_struct() -> AnyResult<()> {
    let event = Transfer::from_relevant_fields(
        AccountId(String::from("a")),
        AccountId(String::from("b")),
        10,
    );

    assert_eq!(event, Transfer::new("a", "b", 10));

    Ok(())
}

#[test]
fn generates_constructor_for_struct_with_ignored_fields() -> AnyResult<()> {
    let event = MultisigExecuted::from_relevant_fields(
        AccountId(String::from("a")),
        AccountId(String::from("b")),
        [1; 32],
    );

    assert_eq!(
        event,
        MultisigExecuted::new("a", Default::default(), "b", [1; 32], Ok(()))
    );

    Ok(())
}
