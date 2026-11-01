// Direct typed behavior from complete pinned Oracle and XMage sources.
const fn selection(count: u16) -> LibrarySelectionEffect {
    LibrarySelectionEffect {
        spec: LibrarySelectionSpec {
            look: 0,
            count: LibrarySelectionCount::Fixed(count),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::BottomRandom,
        },
        look: LibraryLook::Amount(AmountValue::Stat(StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 2,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }))),
        destination: LibraryPickDestination::Hand,
        public: false,
        maximum_mana_value: None,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfKicked { skip: 2 },
            Instruction::SelectLibraryTo(selection(2)),
            Instruction::Skip(1),
            Instruction::SelectLibraryTo(selection(1)),
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
