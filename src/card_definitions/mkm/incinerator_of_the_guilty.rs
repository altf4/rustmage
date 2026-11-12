const COMBAT_KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::TRAMPLE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: COMBAT_KEYWORDS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 2 },
                Instruction::CollectEvidenceX,
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayerPermanents {
                players: PlayerSet::DamagedPlayer,
                kinds: 1 | 64,
                amount: AmountValue::ChosenX,
            }],
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
        },
    ],
    ..CardRules::EMPTY
};
