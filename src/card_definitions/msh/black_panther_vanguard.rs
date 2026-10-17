// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: true,
            subtype: Some("Hero"),
            include_source: false,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Create a 1/1 white Soldier creature token.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::CreateTokens {
                    token: Card(5318),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Creatures you control get +1/+1 until end of turn.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    power: 1,
                    toughness: 1,
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
