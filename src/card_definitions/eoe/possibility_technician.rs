// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: false,
            subtype: Some("Kavu"),
            include_source: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlayWhileControlSubtype("Kavu")],
    }],
    extras: &CardExtras {
        warp_cost: Some(ManaCost {
            generic: 1,
            colored: [0, 0, 0, 1, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
