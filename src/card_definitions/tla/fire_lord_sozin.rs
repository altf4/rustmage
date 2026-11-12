// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::MENACE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PayXReflexive { ability: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::DamagedPlayerObject {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                    },
                    constraint: TargetGroupConstraint::TotalManaValueAtMostX,
                },
                min: 0,
                max: usize::MAX,
            }),
            program: &[Instruction::ReturnToBattlefieldUnderControl],
        },
    ],
    ..CardRules::EMPTY
};
