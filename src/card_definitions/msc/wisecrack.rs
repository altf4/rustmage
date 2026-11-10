// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::PowerDamage {
                source: 0,
                recipient: 0,
            },
            Instruction::IfSubject {
                subject: Subject::Targets,
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                skip: 1,
            },
            Instruction::DamageTargetController(2),
        ],
    ),
    ..CardRules::EMPTY
};
