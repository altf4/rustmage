//! Opposing creature death links return only exact cards in the Treasure form.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const RETURNER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::OpposingNontokenCreature),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::ReturnDeathCard(ReturnKind::Treasure),
        ],
    }],
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ArtifactEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const PAYMENT: Action = Action::PayResolutionMana {
    payment: [0, 0, 0, 0, 0, 1],
};

fn start_return(game: &mut Game, dying: CardId) {
    let departure = game.move_object(dying, Zone::Graveyard);
    game.prepare_triggers();
    let item = game.state.stack.last().expect("opposing creature death");
    assert!(item.targets.is_empty());
    assert_eq!(item.return_link.as_ref().unwrap().object, departure.after);
}

fn optional_payment(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::ResolutionMana { .. }));
}

#[test]
fn the_return_enters_as_treasure_retains_printed_abilities_and_has_a_noncopiable_form() {
    let shock = Card::from_name("Shock").unwrap();
    let hill_giant = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &RETURNER),
        (hill_giant, &OBSERVER),
        (shock, &COPY),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let elf = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    start_return(&mut game, elf);
    game.move_object(source, Zone::Graveyard);
    optional_payment(&mut game);
    act(&mut game, PAYMENT);
    settle(&mut game);

    let returned = game.object_view(elf);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, player);
    assert_eq!(returned.owner, opponent);
    assert!(returned.tapped);
    assert_eq!(returned.kinds, 16);
    assert!(returned.has_subtype("Treasure"));
    assert!(!returned.has_subtype("Elf"));
    assert!(game.printed_abilities_available(elf));
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let copy = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, copy, Some(Target::Card(elf)));
    settle(&mut game);
    let copied = game.object_view((game.state.objects.len() - 1) as CardId);
    assert!(copied.is_token() && copied.is_creature());
    assert!(copied.has_subtype("Elf"));
    assert!(!copied.has_subtype("Treasure"));

    let treasure_mana = Action::TapGrantedMana {
        card: elf,
        grantor: returned.reference,
        production: WHITE,
    };
    reject(&mut game, treasure_mana.clone());
    game.state.objects[elf as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: hawk });
    act(&mut game, treasure_mana);
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(&mut game, Action::ChooseMana { production: WHITE });
    }
    let departed = game.object_view(elf);
    assert_eq!(departed.zone, Zone::Graveyard);
    assert_eq!(departed.owner, opponent);
    assert!(!departed.has_subtype("Treasure"));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(hawk).zone, Zone::Battlefield);
}

#[test]
fn death_uses_the_opposing_controller_and_stale_links_do_not_follow_a_return_trip() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RETURNER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let owned = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let token = (game.state.objects.len()) as CardId;
    game.create_tokens(opponent, Card::from_catalogue_id(5323), 1, false);
    game.move_object(token, Zone::Graveyard);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());

    let object = game.object_ref(owned);
    game.change_controller(object, opponent);
    start_return(&mut game, owned);
    let mut stale = game.clone();
    stale.move_object(owned, Zone::Exile);
    stale.move_object(owned, Zone::Graveyard);
    optional_payment(&mut stale);
    act(&mut stale, PAYMENT);
    assert_eq!(stale.object_view(owned).zone, Zone::Graveyard);

    optional_payment(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.object_view(owned).zone, Zone::Graveyard);
    act(&mut game, PAYMENT);
    assert_eq!(game.object_view(owned).controller, player);
    assert!(game.object_view(owned).has_subtype("Treasure"));
}

#[test]
fn projected_entry_has_the_same_treasure_type_and_mana_grant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let origin = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let recipient = put(&mut game, player.other(), "Llanowar Elves", Zone::Graveyard);
    let mut movement = game.entry_move(recipient, Zone::Battlefield, true);
    movement.controller = player;
    movement.return_modification = Some((ReturnKind::Treasure, game.object_ref(origin)));
    let goal = FundingGoal {
        actor: player,
        required: WHITE,
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    let mut projected = None;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        let view = characteristic_query::CharacteristicQuery::in_world(next).object_view(recipient);
        assert_eq!(view.kinds, 16);
        assert!(view.has_subtype("Treasure"));
        assert!(view.tapped);
        assert!(next.printed_abilities_available(recipient));
        assert!(
            next.granted_mana(recipient)
                .iter()
                .any(|ability| ability.sacrifice)
        );
        projected = Some(view);
        true
    });
    game.stage_entries(vec![movement], None);
    game.advance_entries();
    let actual = game.object_view(recipient);
    let projected = projected.expect("projected returned card");
    assert_eq!(actual.reference, projected.reference);
    assert_eq!(actual.granted_subtypes, projected.granted_subtypes);
    assert_eq!(actual.kinds, projected.kinds);
    assert_eq!(actual.controller, projected.controller);
}
