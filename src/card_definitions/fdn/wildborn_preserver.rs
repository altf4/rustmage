// Complete Oracle body authored from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH.union(Keywords::FLASH),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherFilteredEntersYou(
                EntryPredicate::CreatureWithoutSubtype("Human"),
            ),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::PayXReflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            }],
        },
    ],
    ..CardRules::EMPTY
};
