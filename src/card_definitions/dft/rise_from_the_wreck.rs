// Each "target" starts an independent optional group; overlapping legal identities may repeat.
const OWN_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWN_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: OWN_CREATURE,
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &OWN_GRAVE,
                    predicate: &TargetPredicate::Subtype("Mount"),
                },
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &OWN_GRAVE,
                    predicate: &TargetPredicate::Subtype("Vehicle"),
                },
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &OWN_CREATURE,
                    predicate: &TargetPredicate::NoAbilities,
                },
                min: 0,
                max: 1,
            },
        ]),
        &[Instruction::ReturnAllTargetsToHand],
    ),
    ..CardRules::EMPTY
};
