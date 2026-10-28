// Authored from the pinned Oracle and complete XMage implementation; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::Destroy,
            Instruction::DrawTargetControllerIfNotAttacking,
        ],
    ),
    ..CardRules::EMPTY
};
