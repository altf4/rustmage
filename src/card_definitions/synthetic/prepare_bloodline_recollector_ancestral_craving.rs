// Ancestral Craving prepared face; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Player { opponent_only: false }), &[
        Instruction::Draw(PlayerSet::Targeted, 3),
        Instruction::LoseLife(PlayerSet::Targeted, 3),
    ]),
    extras: &CardExtras { prepared_identity: Some("@prepare/bloodline_recollector_ancestral_craving"), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
