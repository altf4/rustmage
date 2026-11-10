// Oracle and pinned XMage evidence are recorded in the paired provenance.
const fn animal(subtype: &'static str) -> CreatureFilter {
    CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some(subtype),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[
                animal("Spider"),
                animal("Boar"),
                animal("Bat"),
                animal("Bear"),
                animal("Bird"),
                animal("Cat"),
                animal("Dog"),
                animal("Frog"),
                animal("Jackal"),
                animal("Lizard"),
                animal("Mouse"),
                animal("Otter"),
                animal("Rabbit"),
                animal("Raccoon"),
                animal("Rat"),
                animal("Squirrel"),
                animal("Turtle"),
                animal("Wolf"),
            ],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
