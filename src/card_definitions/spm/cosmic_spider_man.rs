// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::GrantCreatures {
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Spider"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            keywords: Keywords::FLYING
                .union(Keywords::HASTE)
                .union(Keywords::LIFELINK)
                .union(Keywords::TRAMPLE)
                .union(Keywords::FIRST_STRIKE),
        }],
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
    keywords: Keywords::FLYING
        .union(Keywords::HASTE)
        .union(Keywords::LIFELINK)
        .union(Keywords::TRAMPLE)
        .union(Keywords::FIRST_STRIKE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING
                .union(Keywords::HASTE)
                .union(Keywords::LIFELINK)
                .union(Keywords::TRAMPLE)
                .union(Keywords::FIRST_STRIKE),
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING
                    .union(Keywords::HASTE)
                    .union(Keywords::LIFELINK)
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::FIRST_STRIKE),
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
