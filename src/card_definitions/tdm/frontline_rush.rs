// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create two 1/1 red Goblin creature tokens.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5324),
                amount: 2,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Target creature gets +X/+X until end of turn, where X is the number of creatures you control.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                power: 1,
                toughness: 1,
            }],
        },
    ]),
    triggers: &[],
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
