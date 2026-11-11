pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::TRAMPLE,
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Subtype("Desert"),
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::FlashWhileControlSubtype("Desert")),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
