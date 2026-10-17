// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::Hand,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Discard,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords(0),
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    ),
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
        omen_parent: Some(Card(4178)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
