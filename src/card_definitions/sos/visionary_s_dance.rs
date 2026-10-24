// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card::SosElementalToken,
            amount: 2,
            tapped: false,
        }],
    ),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::Fixed(2),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
