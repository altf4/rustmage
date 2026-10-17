// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Create a 1/1 black Rat creature token with \"This token can't block.\"",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::CreateTokens {
                    token: Card(5323),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "You may sacrifice another creature. If you do, scry 2, then draw a card.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[
                    Instruction::ChooseObjects {
                        kind: EffectChoiceKind::Sacrifice,
                        players: PlayerSet::Controller,
                        filter: SacrificeFilter {
                            artifact_enchantment_or_token: false,
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 1,
                            nonland: false,
                            other: true,
                            tokens: None,
                            subtype: None,
                        },
                        amount: 1,
                        optional_skip: Some(2),
                    },
                    Instruction::Scry(2),
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
            SpellMode {
                text: "Creatures you control gain menace until end of turn.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::GrantCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    keywords: Keywords::MENACE,
                }],
            },
        ])],
    }],
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
    ..CardRules::EMPTY
};
