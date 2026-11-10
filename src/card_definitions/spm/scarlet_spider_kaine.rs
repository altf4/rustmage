// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Discard,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        }),
        mayhem_cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
