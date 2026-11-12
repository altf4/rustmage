pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5686),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceDeclaredAttackThisCombat,
                event: &TriggerCondition::ControlledSubtypeAttacks {
                    subtype: "Wolf",
                    exclude: false,
                    one_or_more: false,
                },
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::RequireSubjectCombat {
                subject: Subject::Targets,
                requirement: CombatRequirement::BlockEventAttacker,
                duration: CombatRequirementDuration::ThisCombat,
            }],
        },
    ],
    ..CardRules::EMPTY
};
