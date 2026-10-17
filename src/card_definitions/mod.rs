//! Directly authored, compile-time card behavior. No rules text is interpreted here.
use super::*;

pub(super) struct CardRules {
    pub effect: Effect,
    pub triggers: &'static [TriggeredAbility],
    pub activated: &'static [ActivatedAbility],
    pub keywords: Keywords,
    pub enters: Counters,
    pub entry: EntryTapped,
    /// Fixed additional mana for each predeclared modal combination.
    pub mode_costs: Option<&'static [ManaCost]>,
}
impl CardRules {
    pub const EMPTY: Self = Self {
        effect: Effect::Unsupported,
        triggers: &[],
        activated: &[],
        keywords: Keywords(0),
        enters: Counters {
            plus_one: 0,
            minus_one: 0,
            stun: 0,
            loyalty: 0,
            lore: 0,
        },
        entry: EntryTapped::Never,
        mode_costs: None,
    };
    pub const fn checked_effect(&self) -> Effect {
        assert!(
            !matches!(self.effect, Effect::Unsupported),
            "complete authored card lacks behavior"
        );
        self.effect
    }
}
const fn generic(amount: u32) -> ManaCost {
    ManaCost {
        colored: [0; 6],
        generic: amount,
    }
}
const fn one(rule: TargetRule) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(rule))
}
const fn object(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
include!("../generated/authored_cards.rs");
