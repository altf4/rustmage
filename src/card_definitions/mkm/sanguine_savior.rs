pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::LIFELINK,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
            alternatives: &[
                ManaCost {
                    colored: [1, 0, 1, 0, 0, 0],
                    generic: 0,
                },
                ManaCost {
                    colored: [0, 0, 2, 0, 0, 0],
                    generic: 0,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
