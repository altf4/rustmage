// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::CreaturePowerDamageBatch {
                source: 0,
                recipients: CreaturePowerDamageRecipients::OtherCreatures,
            },
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::DiscardHand(PlayerSet::Controller),
            Instruction::Draw(PlayerSet::Controller, 4),
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 2, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
