// Direct typed behavior from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Creature,
                ordinal: None,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::CastSpellManaValue,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5414),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
