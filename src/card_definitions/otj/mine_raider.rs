// Complete typed behavior from the pinned Oracle and XMage implementation.
const fn other_outlaw(subtype: &'static str) -> CountFilter {
    CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some(subtype),
        basic: false,
        other: true,
        cards_only: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::CountUnion(&[
                    other_outlaw("Assassin"),
                    other_outlaw("Mercenary"),
                    other_outlaw("Pirate"),
                    other_outlaw("Rogue"),
                    other_outlaw("Warlock"),
                ]),
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
