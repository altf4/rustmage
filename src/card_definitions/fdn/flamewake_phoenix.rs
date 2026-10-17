// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
const STATIC_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttacksEachCombat],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Threshold {
                    value: StepValue::GreatestCreaturePower,
                    minimum: Some(4),
                    maximum: None,
                },
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::ReturnSourceToBattlefield { tapped: false },
        ],
    }],
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        static_ability_groups: STATIC_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
