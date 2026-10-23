// Direct typed behavior from complete pinned Oracle and XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    enters: Counters {
        plus_one: 0,
        minus_one: 6,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    keywords: Keywords::MENACE,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::SourceCounters {
                kind: CounterKind::MinusOne,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::PermanentCardsToGraveyard {
                relation: CastRelation::You,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::RemoveSourceCounterIfAny {
            kind: CounterKind::MinusOne,
            skip: 0,
        }],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
