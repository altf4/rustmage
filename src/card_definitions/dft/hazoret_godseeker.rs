// Complete direct typed behavior from full pinned Oracle and XMage sources.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: true,
    unless: Some(StepGuard::Expression {
        value: StatValue::ControllerSpeed,
        minimum: Some(4),
        maximum: None,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::INDESTRUCTIBLE.union(Keywords::HASTE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    keywords: Keywords::INDESTRUCTIBLE.union(Keywords::HASTE),
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::PowerAtMost(2),
        })),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BE_BLOCKED,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[],

    extras: &CardExtras {
        static_ability_groups: GROUPS,
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
