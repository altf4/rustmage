const fn subtype_count(subtype: &'static str) -> AmountValue {
    AmountValue::Stat(StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some(subtype),
        basic: false,
        other: false,
        cards_only: false,
    }))
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::VIGILANCE.union(Keywords::TRAMPLE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(3),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutPermanentCountersScaled {
                    filters: &[BattlefieldFilter {
                        kinds: 1,
                        relation: 0,
                        nonland: false,
                        tapped: None,
                        without: Keywords::NONE,
                        subtype: None,
                        max_mana: u32::MAX,
                    }],
                    kind: CounterKind::PlusOne,
                    amount: subtype_count("Elf"),
                },
                Instruction::DrawScaled(PlayerSet::Controller, subtype_count("Wolf")),
            ],
        },
    ],
    ..CardRules::EMPTY
};
