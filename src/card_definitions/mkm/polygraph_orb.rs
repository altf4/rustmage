pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 4,
                    count: LibrarySelectionCount::Fixed(2),
                    filter: LibrarySelectionFilter::Any,
                    optional: false,
                    reveal: false,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Fixed(4),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::LoseLife(PlayerSet::Controller, 2),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::CollectEvidence(3),
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
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Opponent, 3),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
