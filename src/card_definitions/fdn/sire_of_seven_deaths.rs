// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::MayPayOpponentLife { amount: 7, skip: 2 },
            Instruction::LoseLife(PlayerSet::Opponent, 7),
            Instruction::Skip(1),
            Instruction::CounterRelated,
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
    keywords: Keywords(33610),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(514))],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32776))],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(320))],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(514))],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32776))],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(320))],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
