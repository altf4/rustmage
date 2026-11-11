// Direct typed behavior from complete pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Noncreature,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::ClassLevelAtLeast(2),
};
const DAMAGE: ReplacementSpec = ReplacementSpec::ConditionalNoncombatDamageToOpponentPlus {
    condition: StatValue::SourceClassLevelAtLeast(3),
    amount: 2,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DAMAGE)],
    },
];
const LEVEL: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::ClassLevelExactly(1),
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 0, 0, 1, 0, 0],
        generic: 2,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainClassLevel(2)],
    sorcery: true,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Discard,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[
        LEVEL,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            program: &[Instruction::GainClassLevel(3)],
            ..LEVEL
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        replacement_effects: &[DAMAGE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
