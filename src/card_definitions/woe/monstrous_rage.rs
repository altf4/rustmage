// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::CreateRole {
                token: Card::from_catalogue_id(5330),
                subject: Subject::Targets,
            },
        ],
    ),
    ..CardRules::EMPTY
};
