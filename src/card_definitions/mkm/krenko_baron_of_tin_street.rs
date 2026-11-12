pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
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
        program: &[Instruction::PutFilteredCounters {
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Goblin"),
                basic: false,
                other: false,
                cards_only: false,
            },
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentToGraveyard {
            kinds: 16,
            relation: CastRelation::Any,
            exclude_source_kinds: 0,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::CreateTemporaryTokens {
                token: Card(5324),
                amount: AmountValue::Fixed(1),
                keywords: Keywords::HASTE,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    }],
    ..CardRules::EMPTY
};
