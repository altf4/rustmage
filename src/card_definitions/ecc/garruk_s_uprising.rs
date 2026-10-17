// Direct typed behavior; full Oracle and pinned XMage dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Threshold {
                    value: StepValue::GreatestCreaturePower,
                    minimum: Some(4),
                    maximum: None,
                },
                event: &TriggerCondition::SelfEnters,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::CreaturePowerAtLeast(4)),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(
            0,
            KeywordGrant {
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
                source_only: false,
                own_turn: false,
                keywords: Keywords::TRAMPLE,
            },
        )],
    }],
    extras: &CardExtras {
        keyword_grants: &[KeywordGrant {
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
            source_only: false,
            own_turn: false,
            keywords: Keywords::TRAMPLE,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
