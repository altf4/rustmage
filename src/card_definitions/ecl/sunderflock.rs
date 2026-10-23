// Direct typed behavior from complete pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::GreatestManaValue(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Elemental"),
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
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnCreaturesWithoutSubtype("Elemental")],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
