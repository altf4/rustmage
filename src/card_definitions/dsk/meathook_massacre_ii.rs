const RETURN_WITH_FINALITY: Instruction = Instruction::ReturnDeathCardWithCounters {
    under_controller: true,
    tapped: false,
    counter: CounterKind::Finality,
    amount: 1,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeScaled {
                players: PlayerSet::Each,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: AmountValue::ChosenX,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::Creature(
                CastRelation::You,
            )),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayControllerLife { amount: 3, skip: 2 },
                Instruction::LoseLife(PlayerSet::Controller, 3),
                RETURN_WITH_FINALITY,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::Creature(
                CastRelation::Opponent,
            )),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayEventPlayerLife { amount: 3, skip: 1 },
                Instruction::Skip(1),
                RETURN_WITH_FINALITY,
            ],
        },
    ],
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
