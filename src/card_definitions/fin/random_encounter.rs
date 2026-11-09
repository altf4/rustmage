// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ShuffleYourLibrary,
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1),
                count: MilledCount::All,
                destination: MilledDestination::BattlefieldWithHasteAndReturn { ability: 0 },
            }),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDelayedBattlefieldObjectsToHand],
    }],
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
