// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::DOUBLE_STRIKE,
        }],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Picnic Ruiner // Stolen Goodies"),
        adventure_spell: Some(Card::from_catalogue_id(5558)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
