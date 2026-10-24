// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::CounterKeyword(
            0,
            CounterKind::PlusOne,
            Keywords::TRAMPLE,
        )],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensWithCounters {
            token: Card::SosFractalToken,
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Stat(StatValue::DistinctNames(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        counter_keyword_grants: &[(CounterKind::PlusOne, Keywords::TRAMPLE)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
