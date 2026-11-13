// Direct typed behavior from the paired Oracle and pinned XMage sources.
const AFFINITY: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Ally"),
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
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 2,
            },
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[Instruction::GroupPowerDamage {
            source: 0,
            recipient: 1,
        }],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, AFFINITY)],
    }],
    extras: &CardExtras {
        cost_reductions: &[AFFINITY],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
