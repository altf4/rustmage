// Direct typed behavior from complete Oracle and pinned XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::AtLeast {
        value: &StatValue::ControllerSpeed,
        minimum: 4,
    },
    stats: Stats {
        power: 1,
        toughness: 2,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
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
