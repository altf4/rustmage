// Official ruling requires animation after token entry; it is not a copiable exception.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 16,
            nontoken: true,
            subtype: None,
            include_source: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                skip: 1,
            },
            Instruction::CopyEnteringPermanentThenAnimate(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Robot", "Villain"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
