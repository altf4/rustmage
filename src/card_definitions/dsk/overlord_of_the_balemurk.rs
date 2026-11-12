const RECOVER: SearchFilter = SearchFilter::AnyOf(&[
    SearchFilter::Kinds(64),
    SearchFilter::AllOf(&[SearchFilter::Creature, SearchFilter::NonSubtype("Avatar")]),
]);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ReturnGraveyardToHandMatching(RECOVER),
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
                    optional_skip: Some(0),
                },
            ],
        },
        ImpendingSpec::END_STEP,
    ],
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 1,
            },
            time_counters: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
