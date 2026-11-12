use crate::engine::{RoomOperation, RoomSelection};

const ENCHANTMENT_RETURN: SacrificeFilter = SacrificeFilter {
    kinds: 0,
    keywords: Keywords::NONE,
    source_only: false,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfRoomDoorAvailable {
                    selection: RoomSelection::Controlled,
                    operation: RoomOperation::Unlock,
                    skip: 4,
                },
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ReturnGraveyardToHand { kinds: 32 },
                    players: PlayerSet::Controller,
                    filter: ENCHANTMENT_RETURN,
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Skip(3),
                Instruction::ChooseRoomDoor {
                    selection: RoomSelection::Controlled,
                    operation: RoomOperation::Unlock,
                },
                Instruction::Skip(1),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ReturnGraveyardToHand { kinds: 32 },
                    players: PlayerSet::Controller,
                    filter: ENCHANTMENT_RETURN,
                    amount: 1,
                    optional_skip: None,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Eerie,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5672), // White 3/1 flying Spirit Token
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
