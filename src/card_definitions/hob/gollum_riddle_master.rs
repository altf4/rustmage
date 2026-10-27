// Complete Oracle text; pinned XMage inventory records absence of a card class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::SourceChosenManaValueParity,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Put a +1/+1 counter on Gollum.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Each opponent loses 2 life and you gain 2 life.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::LoseLife(PlayerSet::Opponent, 2),
                    Instruction::GainLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Draw a card.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
        ])],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::ManaValueParity),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
