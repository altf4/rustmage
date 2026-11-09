pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::GainControlUntilEndTurn,
                Instruction::Untap,
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::HASTE.union(Keywords::LIFELINK),
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::OpponentTakesPermanent,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
