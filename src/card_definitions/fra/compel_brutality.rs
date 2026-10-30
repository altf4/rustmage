// Direct typed rules from complete pinned Oracle and XMage sources; gated pending validation.
const ENEMY: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1 | 64,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWN_WALKER: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 64,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature you control deals damage equal to its power to target creature or planeswalker an opponent controls.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(ENEMY),
            ]),
            program: &[Instruction::PowerDamage {
                source: 0,
                recipient: 1,
            }],
        },
        SpellMode {
            text: "Target planeswalker you control deals damage equal to its loyalty to target creature or planeswalker an opponent controls.",
            targets: TargetPlan::Groups(&[TargetSpec::one(OWN_WALKER), TargetSpec::one(ENEMY)]),
            program: &[Instruction::LoyaltyDamage {
                source: 0,
                recipient: 1,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
