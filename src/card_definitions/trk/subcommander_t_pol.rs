// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Counter target activated or triggered ability.",
                targets: one(TargetRule::Ability),
                program: &[Instruction::Counter],
            },
            SpellMode {
                text: "Tap up to one target artifact or creature. Put a stun counter on it.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Union(&[
                        TargetRule::Creature,
                        TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 16,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                    ]),
                    min: 0,
                    max: 1,
                }),
                program: &[
                    Instruction::Tap,
                    Instruction::PutCounters {
                        subject: Subject::Targets,
                        kind: CounterKind::Stun,
                        amount: 1,
                    },
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
