// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/tam_observant_sequencer_deep_sight"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
