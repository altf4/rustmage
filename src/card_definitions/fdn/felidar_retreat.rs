// Directly authored from complete pinned Oracle and XMage executable behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Create a 2/2 white Cat Beast creature token.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5337),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Put a +1/+1 counter on each creature you control. Those creatures gain vigilance until end of turn.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::PutCreatureCounters {
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        kind: CounterKind::PlusOne,
                        amount: 1,
                    },
                    Instruction::GrantCreatures {
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords(0),
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        keywords: Keywords::VIGILANCE,
                    },
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
