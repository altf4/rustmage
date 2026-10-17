// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::ManaValueMaximum {
                filter: &CastFilter::Any,
                maximum: 3,
            },
            ordinal: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 1)],
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
    extras: &CardExtras {
        adventure_identity: Some("Scalding Viper // Steam Clean"),
        adventure_spell: Some(Card(5186)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
