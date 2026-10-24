// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            },
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/scathing_shadelock_venomous_words"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
