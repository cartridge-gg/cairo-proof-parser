use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use crate::{from_felts, from_felts_with_lengths, to_felts};

use super::error::Result;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Basic {
    a: Felt,
    b: Felt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Nested {
    a: Felt,
    b: Basic,
    c: Felt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct WithSequence {
    a: Vec<Felt>,
    b: Felt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct WithArray {
    a: [Felt; 2],
    b: Felt,
}

#[test]
fn test_deser_basic() -> Result<()> {
    let value = Basic {
        a: 1u64.into(),
        b: 2u64.into(),
    };
    let expected = vec![1u64.into(), 2u64.into()];

    assert_eq!(to_felts(&value).unwrap(), expected);
    assert_eq!(from_felts::<Basic>(&expected).unwrap(), value);
    Ok(())
}

#[test]
fn test_deser_nested() -> Result<()> {
    let value = Nested {
        a: 1u64.into(),
        b: Basic {
            a: 11u64.into(),
            b: 12u64.into(),
        },
        c: 2u64.into(),
    };
    let expected = vec![1u64.into(), 11u64.into(), 12u64.into(), 2u64.into()];

    assert_eq!(to_felts(&value).unwrap(), expected);
    assert_eq!(from_felts::<Nested>(&expected).unwrap(), value);
    Ok(())
}

#[test]
fn test_deser_seq() -> Result<()> {
    let value = WithSequence {
        a: vec![11u64.into(), 12u64.into()],
        b: 2u64.into(),
    };
    let expected = vec![2u64.into(), 11u64.into(), 12u64.into(), 2u64.into()];

    assert_eq!(to_felts(&value).unwrap(), expected);
    assert_eq!(from_felts::<WithSequence>(&expected).unwrap(), value);
    Ok(())
}

#[test]
fn test_deser_arr() -> Result<()> {
    let value = WithArray {
        a: [11u64.into(), 12u64.into()],
        b: 2u64.into(),
    };
    let expected = vec![11u64.into(), 12u64.into(), 2u64.into()];

    assert_eq!(to_felts(&value).unwrap(), expected);
    assert_eq!(from_felts::<WithArray>(&expected).unwrap(), value);
    Ok(())
}

#[test]
fn test_deser_seq_with_len() -> Result<()> {
    let len_override = ("a".to_string(), vec![2]);
    let de: WithSequence = from_felts_with_lengths(
        &vec![11u64.into(), 12u64.into(), 2u64.into()],
        vec![len_override].into_iter().collect(),
    )?;
    let expected = WithSequence {
        a: vec![11u64.into(), 12u64.into()],
        b: 2u64.into(),
    };

    assert_eq!(de, expected);
    Ok(())
}
