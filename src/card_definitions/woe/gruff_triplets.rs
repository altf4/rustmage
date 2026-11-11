// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfNontokenEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyToken {
                subject: Subject::Source,
                amount: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutNamedPermanentCountersScaled {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                name: "Gruff Triplets",
                kind: CounterKind::PlusOne,
                amount: AmountValue::SourcePower,
            }],
        },
    ],
    ..CardRules::EMPTY
};
