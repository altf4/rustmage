// Generated module links only; behavior lives in independent Rust files.
pub(super) mod explosive_derailment {
    use super::*;
    include!("../card_definitions/explosive_derailment.rs");
}
pub(super) mod jailbreak_scheme {
    use super::*;
    include!("../card_definitions/jailbreak_scheme.rs");
}
pub(super) mod requisition_raid {
    use super::*;
    include!("../card_definitions/requisition_raid.rs");
}
pub(super) mod rustler_rampage {
    use super::*;
    include!("../card_definitions/rustler_rampage.rs");
}
pub(super) fn mode_costs(card:Card)->Option<&'static [ManaCost]> {
    match card.0 {
        1406 => explosive_derailment::RULES.mode_costs,
        2258 => jailbreak_scheme::RULES.mode_costs,
        3471 => requisition_raid::RULES.mode_costs,
        3630 => rustler_rampage::RULES.mode_costs,
        _=>None,
    }
}
