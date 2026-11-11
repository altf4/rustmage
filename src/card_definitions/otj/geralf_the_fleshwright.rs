const ZOMBIE_ROGUE: Card = Card::from_catalogue_id(5305);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::YourTurn,
                event: &TriggerCondition::When {
                    guard: StepGuard::Expression {
                        value: StatValue::SpellsCastThisTurn,
                        minimum: Some(2),
                        maximum: None,
                    },
                    event: &TriggerCondition::SpellCast {
                        relation: CastRelation::You,
                        filter: CastFilter::Any,
                        ordinal: None,
                    },
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: ZOMBIE_ROGUE,
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SubtypeAny(&["Zombie"])),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
                amount: AmountValue::OtherZombieEntriesThisTurn,
            }],
        },
    ],
    ..CardRules::EMPTY
};
