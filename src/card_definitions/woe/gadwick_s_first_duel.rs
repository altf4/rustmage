// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::CreateRole {
                token: Card(5263),
                subject: Subject::Targets,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Scry(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyNextInstantSorcery { ability: 3 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedInstantSorceryCastManaValueAtMost { maximum: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpell {
                count: SpellCopyCount::One,
                retarget: true,
            }],
        },
    ],
    ..CardRules::EMPTY
};
