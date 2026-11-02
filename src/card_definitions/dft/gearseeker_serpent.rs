// Direct typed behavior from complete pinned Oracle and XMage sources.
const AFFINITY: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 16,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 6),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, AFFINITY)],
    }],
    extras: &CardExtras {
        cost_reductions: &[AFFINITY],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::CANT_BE_BLOCKED,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
