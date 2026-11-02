// Direct typed behavior from complete Oracle and pinned XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::ControllerSpeed,
        minimum: Some(4),
        maximum: None,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
        },
    ],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
