// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5357),
                amount: 1,
                tapped: false,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Ninja"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(5),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::LoseHalfLifeRoundedUp(PlayerSet::DamagedPlayer),
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(
            0,
            KeywordGrant {
                condition: None,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: Some("Ninja"),
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                source_only: false,
                own_turn: false,
                keywords: Keywords::DEATHTOUCH,
            },
        )],
    }],
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [1, 0, 1, 0, 0, 0],
            generic: 0,
        }],
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::KeywordGrant(
                0,
                KeywordGrant {
                    condition: None,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: Some("Ninja"),
                        keyword: Keywords::NONE,
                        attacking: true,
                        tokens: None,
                        artifact: false,
                    },
                    source_only: false,
                    own_turn: false,
                    keywords: Keywords::DEATHTOUCH,
                },
            )],
        }],
        keyword_grants: &[KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some("Ninja"),
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: false,
            keywords: Keywords::DEATHTOUCH,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
