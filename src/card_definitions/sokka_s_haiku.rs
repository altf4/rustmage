// Complete printed behavior. Base characteristics come from the pinned facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Spell),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::Counter,
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::SelectGroup(1),
            Instruction::Untap,
        ],
    ),
    ..CardRules::EMPTY
};
