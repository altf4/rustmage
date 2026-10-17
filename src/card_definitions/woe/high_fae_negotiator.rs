// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBargained,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 3),
            Instruction::GainLife(PlayerSet::Controller, 3),
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
    keywords: Keywords(1),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
        }],
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
