use serde::{Deserialize, Deserializer};
use starknet_crypto::FieldElement;

pub fn montgomery_to_felt(montgomery_felt: FieldElement) -> FieldElement {
    let dd: Vec<u64> = montgomery_felt
        .to_bytes_be()
        .chunks(8)
        .map(|d| {
            let mut segment = [0u8; 8];
            segment.copy_from_slice(&d[..8]);
            segment
        })
        .map(u64::from_be_bytes)
        .rev()
        .collect();

    let mut bytes = [0u64; 4];
    bytes.copy_from_slice(&dd);
    FieldElement::from_mont(bytes)
}

pub fn deserialize_montgomery<'de, D>(de: D) -> Result<FieldElement, D::Error>
where
    D: Deserializer<'de>,
{
    let incorrectly_deserialized_felt =
        FieldElement::deserialize(de).map_err(serde::de::Error::custom)?;
    Ok(montgomery_to_felt(incorrectly_deserialized_felt))
}

pub fn deserialize_montgomery_vec<'de, D>(de: D) -> Result<Vec<FieldElement>, D::Error>
where
    D: Deserializer<'de>,
{
    let incorrectly_deserialized_felts =
        Vec::<FieldElement>::deserialize(de).map_err(serde::de::Error::custom)?;

    Ok(incorrectly_deserialized_felts
        .into_iter()
        .map(montgomery_to_felt)
        .collect())
}

#[test]
fn test() {
    let expected = "0x00f2e6af983ae40f9d409cbc81a3a9f70ce2ef9ccd2d2018aba74f3a77406193";
    let got = "0x004b372a6c0acf83dd330cdf701e5dc85726b19819d4b33158dcb57a33f704c7";

    let felt = montgomery_to_felt(FieldElement::from_hex_be(got).unwrap());
    assert_eq!(felt, FieldElement::from_hex_be(expected).unwrap());
}
