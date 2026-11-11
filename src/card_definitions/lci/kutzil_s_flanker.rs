pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on this creature for each creature that left the battlefield under your control this turn.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCountersScaled {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Stat(StatValue::ControlledCreatureDeparturesThisTurn),
                }],
            },
            SpellMode {
                text: "You gain 2 life and scry 2.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::GainLife(PlayerSet::Controller, 2),
                    Instruction::Scry(2),
                ],
            },
            SpellMode {
                text: "Exile target player's graveyard.",
                targets: one(TargetRule::Player {
                    opponent_only: false,
                }),
                program: &[Instruction::ExileGraveyard(PlayerSet::Targeted)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
