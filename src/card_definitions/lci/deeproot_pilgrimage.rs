const MERFOLK: Card = Card::from_catalogue_id(5576);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledPermanentsOfSubtypeTapped {
            subtype: "Merfolk",
            nontoken: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: MERFOLK,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
