// Direct typed behavior from the paired Oracle and pinned XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Colors {
        filter: &CastFilter::Any,
        mask: 4,
    },
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const STATIC: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: STATIC,
    extras: &CardExtras {
        static_ability_groups: STATIC,
        cost_reductions: &[REDUCTION],
        zone_replacement: Some(
            ZoneReplacementRule::ExileOpposingNontokenCreatureDeathLinked { life: 2 },
        ),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 4,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::LinkedExileCreature {
            ability_origin: None,
        }),
        program: &[Instruction::ReturnExiledTargetTappedWithCounters(&[(
            CounterKind::PlusOne,
            2,
        )])],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
