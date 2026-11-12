// Direct typed behavior from complete pinned Oracle and XMage sources.
use rustmage_core::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            subject: DepartureSubject::SelfOrOtherMatching,
            destinations: &[Zone::Hand, Zone::Library, Zone::Exile, Zone::Command],
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
