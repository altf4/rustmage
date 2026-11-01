//! Single version source shared with the Python binding and current-contract tests.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) const VERSION: u32 = parse(include_str!("../../../data/observation-schema.txt"));

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const fn parse(text: &str) -> u32 {
    let bytes = text.as_bytes();
    let end = if !bytes.is_empty() && bytes[bytes.len() - 1] == b'\n' {
        bytes.len() - 1
    } else {
        bytes.len()
    };
    assert!(end > 0, "observation schema version is empty");
    let mut value = 0u32;
    let mut i = 0;
    while i < end {
        assert!(
            bytes[i] >= b'0' && bytes[i] <= b'9',
            "invalid observation schema version"
        );
        value = value
            .checked_mul(10)
            .expect("observation schema version overflow");
        value = value
            .checked_add((bytes[i] - b'0') as u32)
            .expect("observation schema version overflow");
        i += 1;
    }
    assert!(value > 0, "observation schema version must be positive");
    value
}
