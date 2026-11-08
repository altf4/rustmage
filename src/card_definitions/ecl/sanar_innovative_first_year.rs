// Direct typed behavior from complete Oracle and pinned XMage sources.
const COLORS: AmountValue = AmountValue::Stat(StatValue::Colors(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::ControlledColors,
                filter: LibrarySelectionFilter::Any,
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::Shuffle,
            },
            look: LibraryLook::UntilNonlandCount(COLORS),
            destination: LibraryPickDestination::ExileCastThisTurn,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
