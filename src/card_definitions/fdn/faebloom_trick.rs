// Directly authored from complete pinned Oracle and XMage executable behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensReflexive {
            token: Card(5286),
            amount: 2,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::Tap],
    }],
    ..CardRules::EMPTY
};
