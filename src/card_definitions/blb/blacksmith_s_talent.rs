// Direct typed behavior from complete pinned Oracle and XMage sources.
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
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5620),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::ClassLevelAtLeast(2),
            },
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Equipment"),
                }),
                TargetSpec {
                    rule: TargetRule::creature_you_control(),
                    min: 0,
                    max: 1,
                },
            ]),
            program: &[Instruction::AttachEquipmentByGroups {
                equipment: 0,
                recipient: 1,
            }],
        },
    ],
    activated: &[
        LEVEL,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            cost: ActivationCost {
                generic: 3,
                ..LEVEL.cost
            },
            program: &[Instruction::GainClassLevel(3)],
            ..LEVEL
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ConditionalEquippedCreatureKeywords {
            condition: StatValue::AtLeast {
                value: &StatValue::Sum(&[
                    StatValue::SourceClassLevelAtLeast(3),
                    StatValue::ControllerTurn,
                ]),
                minimum: 2,
            },
            keywords: Keywords::DOUBLE_STRIKE.union(Keywords::HASTE),
        }],
    }],
    ..CardRules::EMPTY
};
