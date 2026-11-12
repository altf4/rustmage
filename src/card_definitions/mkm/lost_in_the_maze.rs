// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        program: &[
            Instruction::TargetsExactlyX,
            Instruction::Tap,
            Instruction::PutCountersOnTargetsByController {
                relation: CastRelation::Opponent,
                kind: CounterKind::Stun,
                amount: 1,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTappedKeywords {
            other: false,
            keywords: Keywords::HEXPROOF,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
