// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::RevealHandDiscard { excluded_kinds: 2 },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Faerie"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: None,
                    maximum: Some(0),
                },
                skip: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileFromHand,
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
                optional_skip: None,
            },
        ],
    ),
    ..CardRules::EMPTY
};
