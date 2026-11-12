use crate::engine::{RoomOperation, RoomSelection};

const SEARCH: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        tap: true,
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::Search(SearchSpec {
        filter: SearchFilter::BasicLand,
        maximum: 1,
        destination: SearchDestination::Hand,
        tapped: false,
        reveal: true,
    })],
    sorcery: false,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        SEARCH,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                ..SEARCH.cost
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Room"),
            }),
            program: &[Instruction::ChooseRoomDoor {
                selection: RoomSelection::Target,
                operation: RoomOperation::Either,
            }],
            sorcery: true,
            ..SEARCH
        },
    ],
    ..CardRules::EMPTY
};
