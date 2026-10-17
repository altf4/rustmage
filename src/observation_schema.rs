//! Single version source shared with the Python binding and current-contract tests.
pub(super) const VERSION: u32 = parse(include_str!("../data/observation-schema.txt"));

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

#[cfg(test)]
mod tests {
    #[test]
    fn observation_wire_version_matches_shared_source() {
        let game = super::super::tests::scenario();
        assert_eq!(game.observe(crate::Player::P0).schema, super::VERSION);
        assert_eq!(
            super::VERSION.to_string(),
            include_str!("../data/observation-schema.txt").trim()
        );
    }
}
