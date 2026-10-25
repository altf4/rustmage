// Full frozen Oracle/direct Java, with CR115.3 allowing the same artifact in both groups.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Artifact,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
const ARTIFACT: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 16,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
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
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Groups(&[ARTIFACT, ARTIFACT]),
        program: &[Instruction::BecomeCopy(InPlaceCopySpec {
            recipient_group: Some(0),
            blueprint_group: 1,
            overrides: crate::engine::CopyOverrides {
                name: None,
                legendary: Some(false),
                kinds: None,
                types: None,
                keywords: Keywords::NONE,
            },
            add_kinds: 0,
            base: None,
            next_turn: false,
        })],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
