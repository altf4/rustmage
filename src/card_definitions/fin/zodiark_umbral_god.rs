pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::INDESTRUCTIBLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseHalf {
                kind: EffectChoiceKind::SacrificeCreaturesExceptSubtype("God"),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
