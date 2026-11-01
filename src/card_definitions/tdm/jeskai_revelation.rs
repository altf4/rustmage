// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Union(&[
                TargetRule::Spell,
                TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
            ])),
            TargetSpec::one(TargetRule::Any),
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::ReturnSpellToHand,
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::Damage(4),
            Instruction::CreateTokens {
                token: Card(5486),
                amount: 2,
                tapped: false,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    ..CardRules::EMPTY
};
