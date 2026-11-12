pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrTurnsFaceUp,
        targets: one(TargetRule::Creature),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 4,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 0, 1, 0, 1, 0],
                    generic: 4,
                },
                ManaCost {
                    colored: [0, 0, 0, 0, 2, 0],
                    generic: 4,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
