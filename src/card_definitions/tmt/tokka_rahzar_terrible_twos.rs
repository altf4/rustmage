// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpentBelowValue(&TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 3)],
    }],
    extras: &CardExtras {
        uncounterable: true,
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
