// Only artifact/creature departures trigger life; recovery is limited to this mill batch.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CardsLeaveGraveyardKinds(17),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
                Instruction::SelectMilled(MilledSelectionSpec {
                    filter: MilledFilter::Kinds(19),
                    count: MilledCount::Range {
                        minimum: 0,
                        maximum: 1,
                    },
                    destination: MilledDestination::Hand,
                }),
            ],
        },
    ],
    ..CardRules::EMPTY
};
