// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::SelfEntersIfCastByController,
                guard: StepGuard::Expression {
                    value: StatValue::CountUnion(&[
                        CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Dragon"),
                            basic: false,
                            other: false,
                            cards_only: true,
                        },
                        CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Lesson"),
                            basic: false,
                            other: false,
                            cards_only: true,
                        },
                    ]),
                    minimum: Some(3),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateNonlegendaryCopyTokens {
                subject: Subject::Source,
                amount: 1,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreatures {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some("Dragon"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            power: 2,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
