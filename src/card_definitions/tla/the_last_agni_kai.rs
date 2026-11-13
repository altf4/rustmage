// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::FightForExcess {
                source: 0,
                recipient: 1,
            },
            Instruction::AddExcessDamageMana(ChoiceColor::Red),
            Instruction::RetainManaColorThisTurn(ChoiceColor::Red),
        ],
    ),
    ..CardRules::EMPTY
};
