// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 2,
        }),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: 2,
            toughness: 2,
        }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/spiritcall_enthusiast_scrollboost"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
