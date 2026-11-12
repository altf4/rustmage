pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(9, 9),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPayCosts {
            subject: CounterSubject::Related,
            costs: &[ResolutionCost {
                objects: Some(ResolutionObjectCost {
                    cost: AdditionalCost {
                        kind: EffectChoiceKind::Sacrifice,
                        filter: SacrificeFilter {
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 0,
                            nonland: true,
                            other: false,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        },
                    },
                    count: 3,
                }),
                ..ResolutionCost::FREE
            }],
            exile: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayLinkedExileOnOwnTurnForLife],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileUncontrolledOpponentGraveyardCardsLinked),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
