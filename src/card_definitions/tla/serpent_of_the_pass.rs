// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::AsThoughFlashWithGraveyardSubtype {
            subtype: "Lesson",
            minimum: 3,
        }),
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 1 | 2,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
