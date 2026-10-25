// Flash entry creates an exact-source resolution effect with its printed duration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        program: &[
            Instruction::Tap,
            Instruction::PreventTargetUntapWhileSourceControlled,
        ],
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
    keywords: Keywords::FLASH,
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
