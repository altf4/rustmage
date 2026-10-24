// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::WithoutKinds(1),
        })),
        program: &[Instruction::ExileTargetsPlay { next_turn: true }],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [2, 0, 0, 1, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
