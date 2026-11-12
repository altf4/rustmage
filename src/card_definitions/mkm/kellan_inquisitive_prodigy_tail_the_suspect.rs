pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: object(16),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Destroy, Instruction::DrawIfTargetController],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Kellan, Inquisitive Prodigy // Tail the Suspect"),
        adventure_spell: Some(Card(5697)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
