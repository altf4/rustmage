const GNOME: Card = Card::from_catalogue_id(5267);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: GNOME,
            amount: 2,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureKindBonus {
            kinds: 1,
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
