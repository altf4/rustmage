// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile the top two cards of your library. Until the end of your next turn, you may play those cards.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::ExileTopPlay {
                amount: 2,
                next_turn: true,
            }],
        },
        SpellMode {
            text: "Up to two target creatures each get +2/+1 until end of turn.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 2,
            }),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
