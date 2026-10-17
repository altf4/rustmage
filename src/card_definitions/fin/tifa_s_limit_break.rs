// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Somersault — {0} — Target creature gets +2/+2 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            }],
        },
        SpellMode {
            text: "Meteor Strikes — {2} — Double target creature's power and toughness until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::MultiplyTargetStats { factor: 2 }],
        },
        SpellMode {
            text: "Final Heaven — {6}{G} — Triple target creature's power and toughness until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::MultiplyTargetStats { factor: 3 }],
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
    extras: &CardExtras {
        tiered_costs: Some(&[
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 6,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
