const DINOSAUR: Card = Card::from_catalogue_id(5570);
const TREASURE: Card = Card::from_catalogue_id(5302);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::FIRST_STRIKE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(2),
            },
            Instruction::GrantExiledResultPlay { next_turn: false },
            Instruction::IfExiledLibraryCard {
                land: true,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: DINOSAUR,
                amount: 1,
                tapped: false,
            },
            Instruction::IfExiledLibraryCard {
                land: false,
                skip: 1,
            },
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
