use rustmage_core::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            keywords: Keywords::FLYING,
            relation: CastRelation::You,
            subject: DepartureSubject::SelfOrOtherMatching,
            destinations: &[Zone::Graveyard],
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::ManaValueLessThanEvent { other: true },
        })),
        program: &[Instruction::ReturnToBattlefield],
    }],
    ..CardRules::EMPTY
};
