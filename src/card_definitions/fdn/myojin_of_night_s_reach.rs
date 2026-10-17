// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceCounterKeywords {
            kind: CounterKind::Divinity,
            minimum: 1,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    extras: &CardExtras {
        hand_cast_entry_counters: &[(CounterKind::Divinity, 1)],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: Some((Some(CounterKind::Divinity), 1)),
            loyalty: None,
        },
        program: &[Instruction::DiscardHand(PlayerSet::Opponent)],
        sorcery: false,
    }],
    ..CardRules::EMPTY
};
