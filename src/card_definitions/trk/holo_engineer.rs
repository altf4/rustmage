const ILLUSION: Card = Card::from_catalogue_id(6011);
const REPLACEMENT: ReplacementSpec = ReplacementSpec::ReplaceTokens(ILLUSION);
const STATIC: StaticAbilityGroup = StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, REPLACEMENT)],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: ILLUSION,
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[STATIC],
    extras: &CardExtras {
        static_ability_groups: &[STATIC],
        replacement_effects: &[REPLACEMENT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
