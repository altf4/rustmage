// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 1,
                kind: 64,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }),
            keywords: Keywords::TRAMPLE
                .union(Keywords::HEXPROOF)
                .union(Keywords::HASTE),
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentToGraveyard {
            kinds: 16,
            relation: CastRelation::You,
            exclude_source_kinds: 16,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Scry(1),
        ],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::SourcePower,
        )],
    }],
    ..CardRules::EMPTY
};
