pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Creature),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: -4,
                toughness: -4,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardsLeaveGraveyardKinds(1),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    ..CardRules::EMPTY
};
