// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Each player sacrifices a creature token of their choice.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Each,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords(0),
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: Some(true),
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: None,
                }],
            },
            SpellMode {
                text: "Each player sacrifices a nontoken creature of their choice.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Each,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords(0),
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: Some(false),
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: None,
                }],
            },
            SpellMode {
                text: "Each player sacrifices an enchantment of their choice.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Each,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords(0),
                        source_only: false,
                        kinds: 32,
                        nonland: false,
                        other: false,
                        tokens: None,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: None,
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
