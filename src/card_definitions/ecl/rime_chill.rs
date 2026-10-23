// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
const VIVID: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Colors(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
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
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, VIVID)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 2,
        }),
        &[
            Instruction::Tap,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Stun,
                amount: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[VIVID],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
