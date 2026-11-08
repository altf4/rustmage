// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentLeaves(DepartureFilter {
            kinds: 1,
            relation: CastRelation::You,
            other: false,
            graveyard_only: false,
            counters: DepartureCounters::Unrestricted,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Opponent, 1)],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::ClassLevelExactly(1),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 1,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(2)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::ClassLevelExactly(2),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ConditionalCreatureModifier {
                condition: StatValue::SourceClassLevelAtLeast(2),
                modifier: StaticModifier {
                    own_turn: false,
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
                    stats: Stats {
                        power: 1,
                        toughness: 0,
                    },
                },
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: Some(StatValue::SourceClassLevelAtLeast(2)),
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
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords::MENACE,
                },
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::GrantGraveyardCreatureSneak {
                    minimum_class_level: 3,
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 3,
                    },
                },
                StaticAbilityFragment::CastGraveyardCreaturesUsingSneak {
                    minimum_class_level: 3,
                },
            ],
        },
    ],
    extras: &CardExtras {
        keyword_grants: &[KeywordGrant {
            condition: Some(StatValue::SourceClassLevelAtLeast(2)),
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
            source_only: false,
            own_turn: false,
            keywords: Keywords::MENACE,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
