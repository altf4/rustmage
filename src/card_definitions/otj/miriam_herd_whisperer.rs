// Complete typed behavior from the pinned Oracle and XMage implementation.
const fn hexproof_on_your_turn(subtype: &'static str) -> KeywordGrant {
    KeywordGrant {
        condition: None,
        requires_life_gain: false,
        filter: CreatureFilter {
            relation: 0,
            other: false,
            subtype: Some(subtype),
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
        source_only: false,
        own_turn: true,
        keywords: Keywords::HEXPROOF,
    }
}
const MOUNTS: KeywordGrant = hexproof_on_your_turn("Mount");
const VEHICLES: KeywordGrant = hexproof_on_your_turn("Vehicle");

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeAttacksAny(&["Mount", "Vehicle"]),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, MOUNTS),
            StaticAbilityFragment::KeywordGrant(1, VEHICLES),
        ],
    }],
    extras: &CardExtras {
        keyword_grants: &[MOUNTS, VEHICLES],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
