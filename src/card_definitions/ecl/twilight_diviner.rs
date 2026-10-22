// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(
                &TriggerCondition::CreaturesFromGraveyardEnterYou { other: true },
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyEntryBatchCreature],
        },
    ],
    ..CardRules::EMPTY
};
