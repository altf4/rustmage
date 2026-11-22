use super::{activation, activation_at, begin_attack, enter_step, life, pass_pair, target, tokens};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);
const DETECTIVE: Card = Card::from_catalogue_id(5329);

#[test]
fn caught_red_handed_ignores_ward_untaps_and_suspects_before_control_reverts() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Caught Red-Handed", Zone::Hand);
    let creature = put(&mut game, opponent, "Vein Ripper", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let exact = game.object_ref(creature);
    game.state.objects[creature as usize].tapped = true;
    cast(&mut game, spell, Some(Target::Card(creature)));
    pass_pair(&mut game);
    if game.legal_actions().contains(&Action::DeclineEffectPayment) {
        act(&mut game, Action::DeclineEffectPayment);
    }
    settle(&mut game);
    assert!(game.on_battlefield(ally));
    assert_eq!(game.object_view(creature).controller, player);
    assert!(!game.object_view(creature).tapped);
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    assert_eq!(game.observe(player).suspected, [exact]);

    enter_step(&mut game, Step::End);
    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).controller, opponent);
    assert!(!game.keywords(creature).contains(Keywords::HASTE));
    assert_eq!(game.observe(opponent).suspected, [exact]);
}

#[test]
fn cornered_crook_chooses_its_damage_target_only_after_a_paid_artifact_sacrifice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let crook = put(&mut game, player, "Cornered Crook", Zone::Hand);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let nonartifact = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    cast(&mut game, crook, None);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert!(declined.on_battlefield(artifact));
    assert_eq!(life(&declined, opponent), 20);

    let object = game.object_ref(nonartifact);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let object = game.object_ref(artifact);
    act(&mut game, Action::ChooseAdditionalCost { object });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game.actor() != Some(opponent) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, removal, Some(Target::Card(crook)));
    settle(&mut game);
    assert_eq!(game.object_view(crook).zone, Zone::Graveyard);
    assert_eq!(life(&game, opponent), 17);
}

#[test]
fn felonious_rage_tracks_the_exact_creature_through_a_control_change_until_cleanup() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Felonious Rage", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut game, opponent, "Murder", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    target(&mut game, own);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, own), (4, 2));
    assert!(game.keywords(own).contains(Keywords::HASTE));

    let mut expired = game.clone();
    enter_step(&mut expired, Step::End);
    pass_pair(&mut expired);
    expired.state.players[opponent.index()].mana = [0, 0, 2, 0, 0, 1];
    cast(&mut expired, murder, Some(Target::Card(own)));
    settle(&mut expired);
    assert!(tokens(&expired, player, DETECTIVE).is_empty());

    game.record_control(game.object_ref(own), opponent, None);
    act(&mut game, Action::Pass);
    cast(&mut game, murder, Some(Target::Card(own)));
    settle(&mut game);
    let [token] = tokens(&game, player, DETECTIVE).try_into().unwrap();
    assert!(tokens(&game, opponent, DETECTIVE).is_empty());
    assert_eq!(stats(&game, token), (2, 2));
    assert_eq!(game.object_view(token).colors, 3);
    assert!(game.has_subtype(token, "Detective"));
}

#[test]
fn galvanize_counts_cards_drawn_in_response_when_choosing_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Galvanize", Zone::Hand);
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    let creature = put(
        &mut game,
        player.other(),
        "Earthshaker Dreadmaw",
        Zone::Battlefield,
    );
    cast(&mut game, spell, Some(Target::Card(creature)));
    let mut three = game.clone();
    settle(&mut three);
    assert_eq!(three.object_view(creature).damage, 3);

    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(creature).damage, 5);
}

#[test]
fn innocent_bystander_investigates_for_three_damage_even_when_it_is_lethal() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Innocent Bystander", Zone::Battlefield);
    let small = put(&mut game, player, "Shock", Zone::Hand);
    let large = put(&mut game, player, "Lightning Strike", Zone::Hand);
    let mut short = game.clone();
    cast(&mut short, small, Some(Target::Card(source)));
    settle(&mut short);
    assert_eq!(short.object_view(source).zone, Zone::Graveyard);
    assert!(tokens(&short, player, CLUE).is_empty());

    cast(&mut game, large, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let [clue] = tokens(&game, player, CLUE).try_into().unwrap();
    assert!(game.has_subtype(clue, "Clue"));
}

#[test]
fn knife_uses_the_equipment_controllers_turn_and_can_be_sacrificed_to_draw() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let knife = put(&mut game, player, "Knife", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, knife, None);
    settle(&mut game);
    act(&mut game, activation_at(knife, 1));
    target(&mut game, creature);
    pay(&mut game);
    settle(&mut game);
    assert!(game.has_subtype(knife, "Clue"));
    assert!(game.has_subtype(knife, "Equipment"));
    assert_eq!(stats(&game, creature), (3, 2));
    assert!(game.keywords(creature).contains(Keywords::FIRST_STRIKE));

    game.state.active = opponent;
    game.state.priority = opponent;
    assert_eq!(stats(&game, creature), (2, 2));
    assert!(!game.keywords(creature).contains(Keywords::FIRST_STRIKE));
    game.record_control(game.object_ref(knife), opponent, None);
    assert_eq!(stats(&game, creature), (3, 2));
    assert!(game.keywords(creature).contains(Keywords::FIRST_STRIKE));
    act(&mut game, activation(knife));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(knife).zone, Zone::Graveyard);
    assert_eq!(stats(&game, creature), (2, 2));
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn reckless_detective_draws_and_grows_for_either_optional_attack_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Reckless Detective", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let hand = put(&mut game, player, "Mountain", Zone::Hand);
    begin_attack(&mut game, source);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(stats(&declined, source), (0, 3));
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        0
    );

    let mut discarded = game.clone();
    act(&mut discarded, Action::Discard { card: hand });
    assert_eq!(discarded.object_view(hand).zone, Zone::Graveyard);
    assert!(discarded.on_battlefield(artifact));
    assert_eq!(
        discarded.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    assert_eq!(stats(&discarded, source), (2, 3));

    let object = game.object_ref(artifact);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, source), (2, 3));
}

#[test]
fn red_herring_must_attack_as_a_hasty_creature_and_can_be_sacrificed_to_draw() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Red Herring", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::HASTE));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    reject(&mut game, Action::FinishAttackers);
    begin_attack(&mut game, source);
    assert!(game.object_view(source).attacking);
    act(&mut game, activation(source));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn rubblebelt_braggart_offers_suspect_only_when_its_attack_meets_the_condition() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Rubblebelt Braggart", Zone::Battlefield);
    let exact = game.object_ref(source);
    let mut already = game.clone();
    already.set_suspected(exact, true);
    begin_attack(&mut already, source);
    assert!(already.state.stack.is_empty());

    begin_attack(&mut game, source);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert!(declined.observe(player).suspected.is_empty());
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.observe(player).suspected, [exact]);
    assert!(game.keywords(source).contains(Keywords::MENACE));
}

#[test]
fn torch_the_witness_doubles_x_and_investigates_only_for_positive_excess() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Torch the Witness", Zone::Hand);
    let target_card = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
    for (x, zone, clues) in [
        (0, Zone::Battlefield, 0),
        (1, Zone::Graveyard, 0),
        (2, Zone::Graveyard, 1),
    ] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        reject(&mut branch, Action::ChooseX { value: 3 });
        act(&mut branch, Action::ChooseX { value: x });
        target(&mut branch, target_card);
        act(
            &mut branch,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, x],
            },
        );
        settle(&mut branch);
        assert_eq!(branch.object_view(target_card).zone, zone);
        assert_eq!(tokens(&branch, player, CLUE).len(), clues);
    }
}
