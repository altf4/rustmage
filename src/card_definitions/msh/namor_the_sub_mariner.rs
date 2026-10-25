// Complete direct Oracle behavior; all-zone CDA and frozen chosen-spell symbols.
const POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Merfolk"),
            basic: false,
            other: false,
            cards_only: false,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, POWER)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::ManaSymbols {
                filter: &CastFilter::Noncreature,
                color: ChoiceColor::Blue,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card::MshBlueMerfolkToken,
            amount: AmountValue::CastSpellManaSymbols(ChoiceColor::Blue),
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
