// Generated module links only; behavior lives in independent Rust files.
pub(super) mod enchanted_rivers_grasp {
    use super::*;
    include!("../card_definitions/enchanted_rivers_grasp.rs");
}
pub(super) mod explosive_derailment {
    use super::*;
    include!("../card_definitions/explosive_derailment.rs");
}
pub(super) mod fresh_start {
    use super::*;
    include!("../card_definitions/fresh_start.rs");
}
pub(super) mod frozen_in_ice {
    use super::*;
    include!("../card_definitions/frozen_in_ice.rs");
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
pub(super) mod stop_cold {
    use super::*;
    include!("../card_definitions/stop_cold.rs");
}
pub(super) fn mode_costs(card:Card)->Option<&'static [ManaCost]> {
    match card.0 {
        1325 => enchanted_rivers_grasp::RULES.mode_costs,
        1406 => explosive_derailment::RULES.mode_costs,
        1622 => fresh_start::RULES.mode_costs,
        1640 => frozen_in_ice::RULES.mode_costs,
        2258 => jailbreak_scheme::RULES.mode_costs,
        3471 => requisition_raid::RULES.mode_costs,
        3630 => rustler_rampage::RULES.mode_costs,
        4167 => stop_cold::RULES.mode_costs,
        _=>None,
    }
}
pub(super) fn rules(card:Card)->Option<&'static CardRules> {
    match card.0 {
        1325 => Some(&enchanted_rivers_grasp::RULES),
        1406 => Some(&explosive_derailment::RULES),
        1622 => Some(&fresh_start::RULES),
        1640 => Some(&frozen_in_ice::RULES),
        2258 => Some(&jailbreak_scheme::RULES),
        3471 => Some(&requisition_raid::RULES),
        3630 => Some(&rustler_rampage::RULES),
        4167 => Some(&stop_cold::RULES),
        _=>None,
    }
}
