// Whole Oracle: one received-damage batch, actual counter completion, then later targets.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 7),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutSourceCounterReflexive {
                kind: CounterKind::PlusOne,
                amount: 1,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Union(&[
                TargetRule::Player {
                    opponent_only: false,
                },
                TargetRule::Filtered {
                    base: &TargetRule::Any,
                    predicate: &TargetPredicate::OtherThanSource,
                },
            ])),
            program: &[Instruction::DamageScaled(
                AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
            )],
        },
    ],
    ..CardRules::EMPTY
};
