// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Nonbasic,
        })),
        program: &[
            Instruction::Destroy,
            Instruction::SearchTargetControllerWithEntryCounters {
                spec: SearchSpec {
                    filter: SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped: true,
                    reveal: false,
                },
                amounts: &[(CounterKind::Stun, 1)],
            },
        ],
    }],
    ..CardRules::EMPTY
};
