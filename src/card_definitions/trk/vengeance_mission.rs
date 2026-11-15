// Oracle and pinned XMage reviewed. The threshold is checked after the quest counter is put on.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::Any,
            other: false,
            nontoken: false,
            subtype: None,
        },
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Quest)),
                minimum: 7,
                skip: 2,
            },
            Instruction::LoseLife(PlayerSet::Targeted, 2),
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Targeted, 1),
        ],
    }],
    ..CardRules::EMPTY
};
