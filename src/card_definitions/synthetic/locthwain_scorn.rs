// Adventure spell of Virtue of Persistence, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: -3,
                toughness: -3,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(4908)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
