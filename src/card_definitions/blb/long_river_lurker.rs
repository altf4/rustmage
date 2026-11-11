const WARD: &[Instruction] = &[Instruction::CounterUnlessPay {
    cost: ManaCost {
        colored: [0; 6],
        generic: 1,
    },
    subject: CounterSubject::Related,
    exile: false,
    unpaid_skip: 0,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ConditionalWard {
            condition: StatValue::Fixed(1),
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Frog"),
                basic: false,
                other: true,
                cards_only: false,
            },
            ability: 1,
        }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: WARD,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: WARD,
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::CANT_BE_BLOCKED,
                },
                Instruction::DelayTargetEvent {
                    event: DelayedTargetEvent::CombatDamage,
                    ability: 3,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayIfCombatDamageDealerOnBattlefield { skip: 1 },
                Instruction::BlinkCombatDamageDealer,
            ],
        },
    ],
    ..CardRules::EMPTY
};
