// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5278),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureCounterRule {
            kind: None,
            keywords: Keywords::MAX_ONE_BLOCKER,
        }],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledCreatureCounterRule {
                kind: None,
                keywords: Keywords::MAX_ONE_BLOCKER,
            }],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
