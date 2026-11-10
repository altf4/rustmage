pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Creatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                qualification: DamageQualification::Any,
            },
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::May { skip: 1 }, Instruction::DrawDamageAmount],
    }],
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 2,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
