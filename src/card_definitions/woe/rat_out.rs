// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: -1,
                toughness: -1,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5323),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
