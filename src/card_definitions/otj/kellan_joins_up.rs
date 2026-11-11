pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::HandPlot { maximum: 3 },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 0,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(0),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::LegendaryCreature),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
