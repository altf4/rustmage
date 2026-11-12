pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTokenTapMana {
            kinds: 1,
            production: ManaProduction::Choice {
                mask: 31,
                amount: 1,
            },
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyardKinds(1),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5682),
                amount: 1,
                tapped: false,
            },
            Instruction::ChangeControlledPermanentGroupCounters {
                filters: &[ControlledPermanentFilter {
                    subtype: Some("Plant"),
                    ..ControlledPermanentFilter::ANY
                }],
                effect: PermanentGroupCounters::Add {
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            },
        ],
    }],
    ..CardRules::EMPTY
};
