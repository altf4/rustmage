// Pinned Oracle and complete XMage class: either optional graveyard exile and life,
// or one Clue and life loss.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Exile up to one target card from a graveyard. You gain 2 life.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    min: 0,
                    max: 1,
                }),
                program: &[
                    Instruction::Exile,
                    Instruction::GainLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Investigate. You lose 2 life.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::CreateTokens {
                        token: Card(5296),
                        amount: 1,
                        tapped: false,
                    },
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
