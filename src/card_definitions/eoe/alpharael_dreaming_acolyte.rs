const OWN_TURN: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: true,
    own_turn: true,
    keywords: Keywords::DEATHTOUCH,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, OWN_TURN)],
    }],
    extras: &CardExtras {
        keyword_grants: &[OWN_TURN],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Discard,
                            filter: SacrificeFilter {
                                kinds: 16,
                                other: false,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
