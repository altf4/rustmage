pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: true,
                        tokens: None,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::PutCountersScaled {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::SacrificedPower,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::EnsureGoblinArmy,
                Instruction::AmassGoblinsScaled {
                    player: PlayerSet::Controller,
                    amount: AmountValue::SourceLastKnownPower,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
