// Full pinned Oracle and XMage: all Elf cards, independent of creature type.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 6),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::BorrowGraveyardActivations("Elf")],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::LegendarySubtype(
            "Elf",
        )),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
