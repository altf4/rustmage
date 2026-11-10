// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacksAlone,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantKeywords {
                subject: Subject::Entering,
                keywords: Keywords::DOUBLE_STRIKE,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            })),
            program: &[Instruction::CastTargetedGraveyardFree],
        },
    ],
    ..CardRules::EMPTY
};
