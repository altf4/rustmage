// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 2,
                    toughness: 0,
                },
                Instruction::GrantUntilEndTurn { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::SelfDies),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::ReturnGrantedSource {
                counter: None,
                role: None,
            }],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        warp_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
