// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DelayNextCreatureSpell { ability: 3 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseCardName { nonland: false },
                Instruction::ScheduleNamedCombatDamageTrigger { ability: 4 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedSpellCastCreature,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpellModified {
                retarget: false,
                modification: CapturedSpellModification::Nonlegendary,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
