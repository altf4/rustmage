const fn hybrid_cost(white: u32, blue: u32, black: u32, red: u32) -> ManaCost {
    ManaCost {
        colored: [white, blue, black, red, 4 - white - blue - black - red, 0],
        generic: 0,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Color,
            fragments: &[StaticAbilityFragment::ControlledNonlandAllColors],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ControlledLandsAllBasicTypes],
        },
    ],
    extras: &CardExtras {
        opening_hand_battlefield: true,
        // The catalogue's base payment is {G}{G}{G}{G}.
        cost_alternatives: &[
            hybrid_cost(0, 0, 0, 1),
            hybrid_cost(0, 0, 1, 0),
            hybrid_cost(0, 0, 1, 1),
            hybrid_cost(0, 1, 0, 0),
            hybrid_cost(0, 1, 0, 1),
            hybrid_cost(0, 1, 1, 0),
            hybrid_cost(0, 1, 1, 1),
            hybrid_cost(1, 0, 0, 0),
            hybrid_cost(1, 0, 0, 1),
            hybrid_cost(1, 0, 1, 0),
            hybrid_cost(1, 0, 1, 1),
            hybrid_cost(1, 1, 0, 0),
            hybrid_cost(1, 1, 0, 1),
            hybrid_cost(1, 1, 1, 0),
            hybrid_cost(1, 1, 1, 1),
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
