// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::RegisterEarthbendReturn,
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::CounterKeyword(
            0,
            CounterKind::PlusOne,
            Keywords::TRAMPLE,
        )],
    }],
    extras: &CardExtras {
        counter_keyword_grants: &[(CounterKind::PlusOne, Keywords::TRAMPLE)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
