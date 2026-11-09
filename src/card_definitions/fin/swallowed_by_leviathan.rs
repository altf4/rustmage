// Direct typed behavior from the pinned Oracle and XMage sources.
const GRAVEYARD_CARDS: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[
            Instruction::Surveil(2),
            Instruction::CounterUnlessPayCosts {
                subject: CounterSubject::Targets,
                costs: &[ResolutionCost {
                    extra_generic: Some(AmountValue::Stat(StatValue::Count(GRAVEYARD_CARDS))),
                    ..ResolutionCost::FREE
                }],
                exile: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
