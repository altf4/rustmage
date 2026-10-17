// Capture mana value before the destruction; indestructibility does not prevent life loss.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1 | 32,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        &[Instruction::DestroyLoseManaValue],
    ),
    ..CardRules::EMPTY
};
