// Direct typed Oracle behavior; source and dependencies pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::Wish, 3)],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OwnTurn,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: Some((Some(CounterKind::Wish), 1)),
            loyalty: None,
        },
        program: &[
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: false,
            }),
            Instruction::GainControl {
                subject: Subject::Source,
                controller: PlayerSet::Opponent,
            },
        ],
    }],
    ..CardRules::EMPTY
};
