// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[CreatureFilter {
                relation: 0,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            }],
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::ControllerTurn,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::BattlefieldDeparture(DepartureRule {
                relation: CastRelation::You,
                subject: DepartureSubject::Other,
                ..DepartureRule::ANY
            }),
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5272),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
