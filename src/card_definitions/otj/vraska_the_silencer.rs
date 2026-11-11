pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::OpposingNontokenCreature),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: generic(1),
                skip: 1,
            },
            Instruction::ReturnDeathCard(ReturnKind::Treasure),
        ],
    }],
    ..CardRules::EMPTY
};
