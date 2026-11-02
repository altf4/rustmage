// Layer-four subtype addition preserves other types; the live base effect uses layer seven.
const BASE: LiveStaticEffect = LiveStaticEffect {
    recipients: LiveStatRecipients::Creatures {
        relation: 0,
        other: false,
        enchanted_by_controller: false,
    },
    condition: None,
    power: Some(LiveStatValue::Fixed(6)),
    toughness: Some(LiveStatValue::Fixed(6)),
    bonus_power: 0,
    bonus_toughness: 0,
    keywords: Keywords::NONE,
    cannot_block_subtype: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Type,
    fragments: &[
        StaticAbilityFragment::ControlledCreatureSubtype {
            grant: CreatureSubtypeGrant::Named("Ooze"),
            required_subtype: None,
        },
        StaticAbilityFragment::LiveBase(0, BASE),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::CasterOffTurn {
                filter: &CastFilter::Any,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5301),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        live_static_effects: &[BASE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
