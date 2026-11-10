const JUMP: KeywordGrant = KeywordGrant {
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
    keywords: Keywords::FLYING,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, JUMP)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainControl {
                subject: Subject::Source,
                controller: PlayerSet::DamagedPlayer,
            },
            Instruction::IfSubjectControlledBy {
                subject: Subject::Source,
                controller: PlayerSet::DamagedPlayer,
                skip: 3,
            },
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
            Instruction::CreateTokensScaled {
                token: Card(5302),
                amount: AmountValue::DamageEventAmount,
                tapped: true,
            },
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
        ],
    }],
    extras: &CardExtras {
        keyword_grants: &[JUMP],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
