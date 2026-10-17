// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Player {
                opponent_only: false,
            },
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 64,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 4294967295,
            },
        ]))),
        program: &[Instruction::Damage(2)],
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
