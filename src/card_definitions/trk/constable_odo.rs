// Pinned Oracle and complete XMage class: stun or two Clue tokens.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Tap target creature an opponent controls. Put two stun counters on it.",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                program: &[
                    Instruction::Tap,
                    Instruction::PutCounters {
                        subject: Subject::Targets,
                        kind: CounterKind::Stun,
                        amount: 2,
                    },
                ],
            },
            SpellMode {
                text: "Investigate twice.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5296),
                    amount: 2,
                    tapped: false,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
