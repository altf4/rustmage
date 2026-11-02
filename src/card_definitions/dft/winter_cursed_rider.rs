// A distinct granted Ward template; declared nonmana X is shared by one signed group event.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ConditionalWard {
        condition: StatValue::Fixed(1),
        filter: CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 16,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        },
        ability: 1,
    }],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: GROUPS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(2)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 1, 1, 0, 0, 0],
            mana_component: true,
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyardRange { minimum: 0 },
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
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowFilteredScaled {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: -1,
                kind: 1,
                exclude: 16,
                subtype: None,
                basic: false,
                other: true,
                cards_only: false,
            },
            amount: AmountValue::PaidAdditionalCostSelectionCount,
            power: -1,
            toughness: -1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
