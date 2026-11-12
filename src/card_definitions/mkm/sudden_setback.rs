// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Spell,
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        ])),
        &[Instruction::OwnerLibrary],
    ),
    ..CardRules::EMPTY
};
