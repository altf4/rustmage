// Complete Oracle/XMage: all-zone characteristic power and a target chosen before optional payment.
const DYNAMIC: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::GreatestManaValue(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 16,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, DYNAMIC)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::ReturnToBattlefield,
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(DYNAMIC),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
