//! Reserved generic activation identities; ordinary authored slots are below these.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const INTRINSIC_FALLBACK: u16 = u16::MAX;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const INTRINSIC_COLORLESS: u16 = u16::MAX - 6;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const GRANTED_TAP_MANA: u16 = u16::MAX - 7;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const GRANTED_SACRIFICE_MANA: u16 = u16::MAX - 8;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const EMBALM: u16 = u16::MAX - 9;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const FIRST_RESERVED: u16 = EMBALM;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) const fn intrinsic(color: usize) -> u16 {
    assert!(color < 6);
    u16::MAX - 1 - color as u16
}
const _: () = {
    assert!(EMBALM < GRANTED_SACRIFICE_MANA);
    assert!(GRANTED_SACRIFICE_MANA < GRANTED_TAP_MANA);
    assert!(GRANTED_TAP_MANA < INTRINSIC_COLORLESS);
    assert!(intrinsic(5) == INTRINSIC_COLORLESS);
    assert!(intrinsic(0) < INTRINSIC_FALLBACK);
};
