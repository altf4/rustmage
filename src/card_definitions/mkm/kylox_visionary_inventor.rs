pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE.union(Keywords::HASTE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MaySacrificeAnyNumberForPower(SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                }),
                Instruction::ExileTopCastFree {
                    amount: AmountValue::SacrificedPower,
                    kinds: 4 | 8,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
