// Directly authored from the paired Oracle and pinned XMage sources.
const ANY_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHand { kinds: 1 | 2 },
                players: PlayerSet::Controller,
                filter: ANY_CARD,
                amount: 1,
                optional_skip: Some(0),
            },
        ],
    ),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileSourceAndNamed {
                    name: "Say Its Name",
                    amount: 2,
                },
                filter: ANY_CARD,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchGraveyardHandAndOrLibrary(
            SearchFilter::Name("Altanak, the Thrice-Called"),
        )],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
