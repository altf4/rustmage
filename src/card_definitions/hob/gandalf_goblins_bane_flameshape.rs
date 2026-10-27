// Both faces authored from full pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            },
            Instruction::DamagePlayers(PlayerSet::Opponent, 1),
        ],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Gandalf, Goblins' Bane // Flameshape"),
        adventure_spell: Some(Card(5417)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
