const HASTE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::HASTE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, HASTE)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::ControlledFaceDownCreatures,
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                destinations: &[crate::engine::Zone::Graveyard],
                face_down: true,
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CaptureEntryResult,
                Instruction::ReturnDepartedPermanentFaceDown,
                Instruction::TurnEntryResultFaceUp,
            ],
        },
    ],
    extras: &CardExtras {
        keyword_grants: &[HASTE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
