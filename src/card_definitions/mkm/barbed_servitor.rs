const OPPONENT: TargetPlan = one(TargetRule::Player {
    opponent_only: true,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::INDESTRUCTIBLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Suspect {
                subject: Subject::Source,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDamaged,
            targets: OPPONENT,
            program: &[Instruction::LoseLifeScaled(
                PlayerSet::Targeted,
                AmountValue::DamageReceivedAmount,
            )],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::INDESTRUCTIBLE,
        )],
    }],
    ..CardRules::EMPTY
};
