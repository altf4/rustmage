// Direct typed rules from the pinned Oracle and XMage sources.
const DISCOUNT: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::GreatestManaValue(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        minimum: Some(4),
        maximum: None,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 1,
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
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
                keywords: Keywords::NONE,
            },
            Instruction::GroupPowerDamage {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, DISCOUNT)],
    }],
    extras: &CardExtras {
        cost_reductions: &[DISCOUNT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
