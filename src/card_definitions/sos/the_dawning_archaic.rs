// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 12,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::CastTargetedGraveyardFree],
    }],
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 12,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
