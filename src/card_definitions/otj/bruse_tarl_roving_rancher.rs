pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledSubtypeKeywords {
            subtype: "Ox",
            keywords: Keywords::DOUBLE_STRIKE,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(1),
            },
            Instruction::IfExiledLibraryCard {
                land: true,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5479),
                amount: 1,
                tapped: false,
            },
            Instruction::IfExiledLibraryCard {
                land: false,
                skip: 1,
            },
            Instruction::GrantExiledResultCastUntilEndOfNextTurn,
        ],
    }],
    ..CardRules::EMPTY
};
