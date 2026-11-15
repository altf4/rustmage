pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::Damage(6),
            Instruction::SelectGroup(1),
            Instruction::Destroy,
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(6009),
                amount: 1,
                tapped: true,
            },
        ],
    ),
    ..CardRules::EMPTY
};
