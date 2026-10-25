// Complete frozen Oracle: target announcement follows successful optional payment.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0; 6],
                        generic: 1,
                    },
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Keyword(Keywords::HASTE),
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::ONLY_HASTE_BLOCKERS,
            }],
        },
    ],
    ..CardRules::EMPTY
};
