// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[
            Instruction::DamageRememberPlayers(2),
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::DiscardDamagedPlayers(1),
        ],
    }],
    ..CardRules::EMPTY
};
