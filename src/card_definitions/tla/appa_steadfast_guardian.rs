// Direct typed behavior from Oracle and pinned XMage sources; admission awaits tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object { zone: TargetZone::Battlefield, kinds: 0, relation: 0, other: true, nonland: true, tapped: false, min_mana: 0, max_mana: u32::MAX },
                min: 0, max: usize::MAX,
            }),
            program: &[Instruction::AirbendTargets],
        },
        TriggeredAbility {
            condition: TriggerCondition::CastFromExile(&TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::Any, ordinal: None }),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens { token: Card(5272), amount: 1, tapped: false }],
        },
    ],
    ..CardRules::EMPTY
};
