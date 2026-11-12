pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::TokensEnterYou),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::BecomesClassLevel(2),
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
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
                predicate: &TargetPredicate::Token,
            })),
            program: &[Instruction::CreateCopyToken {
                subject: Subject::Targets,
                amount: 1,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ConditionalCreatureModifier {
            condition: StatValue::SourceClassLevelAtLeast(3),
            modifier: StaticModifier {
                own_turn: false,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: Some(true),
                    artifact: false,
                },
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
            },
        }],
    }],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(1),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 0,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(2)],
        },
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
        },
    ],
    ..CardRules::EMPTY
};
