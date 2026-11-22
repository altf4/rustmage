use super::{activation, activation_at, begin_attack, enter_step, life, pass_pair, target};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn target_player(game: &mut Game, player: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
}

fn surveil_to_graveyard(game: &mut Game, top: CardId) {
    act(
        game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn barbed_servitor_suspects_itself_and_has_two_distinct_damage_triggers() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Barbed Servitor", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.observe(opponent).suspected, [game.object_ref(source)]);
    assert!(
        game.keywords(source)
            .contains(Keywords::INDESTRUCTIBLE.union(Keywords::MENACE))
    );

    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(source)));
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target_player(&mut game, opponent);
    settle(&mut game);
    assert!(game.on_battlefield(source));
    assert_eq!(life(&game, opponent), 18);

    game.state.objects[source as usize].summoning_sick = false;
    begin_attack(&mut game, source);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    settle(&mut game);
    assert_eq!(life(&game, opponent), 17);
    assert_eq!(life(&game, player), 19);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn cerebral_confiscation_lets_the_opponent_discard_or_reveals_for_a_nonland_choice() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Cerebral Confiscation", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let instant = put(&mut game, opponent, "Opt", Zone::Hand);
    let keep = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut private = game.clone();
    act(&mut private, Action::ChooseMode { mode: 0 });
    target_player(&mut private, opponent);
    pay(&mut private);
    pass_pair(&mut private);
    assert_eq!(private.actor(), Some(opponent));
    assert!(private.observe(player).known_hand.is_empty());
    for card in [land, instant] {
        act(&mut private, Action::Discard { card });
    }
    assert_eq!(private.object_view(land).zone, Zone::Graveyard);
    assert_eq!(private.object_view(instant).zone, Zone::Graveyard);
    assert_eq!(private.object_view(keep).zone, Zone::Hand);

    act(&mut game, Action::ChooseMode { mode: 1 });
    target_player(&mut game, opponent);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    let object = game.object_ref(land);
    reject(&mut game, Action::ChooseRevealedDiscard { object });
    let object = game.object_ref(keep);
    act(&mut game, Action::ChooseRevealedDiscard { object });
    settle(&mut game);
    assert_eq!(game.object_view(keep).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Hand);
}

#[test]
fn clandestine_meddler_marks_another_creature_and_surveils_once_per_attack_declaration() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Clandestine Meddler", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let menace = put(
        &mut game,
        player,
        "Prowler, Clawed Thief",
        Zone::Battlefield,
    );
    let top = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    target(&mut game, ally);
    settle(&mut game);
    assert_eq!(game.observe(player).suspected, [game.object_ref(ally)]);

    let mut natural_menace = game.clone();
    begin_attack(&mut natural_menace, menace);
    assert!(natural_menace.state.stack.is_empty());
    game.set_suspected(game.object_ref(second), true);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [ally, second] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert!(game.observe(player).library_choice.unwrap().surveil);
    surveil_to_graveyard(&mut game, top);
}

#[test]
fn lead_pipe_boosts_the_equipped_creature_observes_its_death_and_can_be_sacrificed_to_draw() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Lead Pipe", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.has_subtype(source, "Clue") && game.has_subtype(source, "Equipment"));
    act(&mut game, activation_at(source, 1));
    target(&mut game, ally);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 2));
    assert_eq!(
        game.object_view(source).attached_to,
        Some(game.object_ref(ally))
    );

    cast(&mut game, murder, Some(Target::Card(ally)));
    settle(&mut game);
    assert_eq!(life(&game, player.other()), 19);
    assert_eq!(game.object_view(source).attached_to, None);
    act(&mut game, activation(source));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn massacre_girl_grants_wither_and_draws_for_zero_toughness_rather_than_zero_power() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Massacre Girl, Known Killer", Zone::Hand);
    let dealer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let zero_power = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[zero_power as usize].growth.power = -2;
    let indestructible = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[indestructible as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let bite = put(&mut game, player, "Bite Down", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(
        game.keywords(source)
            .contains(Keywords::MENACE.union(Keywords::WITHER))
    );
    assert!(game.keywords(dealer).contains(Keywords::WITHER));
    assert!(!game.keywords(indestructible).contains(Keywords::WITHER));
    cast(&mut game, murder, Some(Target::Card(zero_power)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    announce(
        &mut game,
        bite,
        &[Target::Card(dealer), Target::Card(indestructible)],
    );
    settle(&mut game);
    assert_eq!(game.object_view(indestructible).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn repeat_offender_samples_its_designation_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Repeat Offender", Zone::Battlefield);
    for _ in 0..2 {
        act(&mut game, activation(source));
        pay(&mut game);
    }
    assert!(game.observe(player).suspected.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.observe(player).suspected, [game.object_ref(source)]);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, source), (3, 2));
}

#[test]
fn soul_enervation_and_rot_farm_mortipede_each_observe_one_creature_card_departure_batch() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let soul = put(&mut game, player, "Soul Enervation", Zone::Hand);
    let mortipede = put(&mut game, player, "Rot Farm Mortipede", Zone::Battlefield);
    let orb = put(&mut game, player, "Polygraph Orb", Zone::Battlefield);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let fuel = [
        put(&mut game, player, "Bear Cub", Zone::Graveyard),
        put(&mut game, player, "Grizzly Bears", Zone::Graveyard),
    ];
    act(&mut game, Action::Pass);
    cast(&mut game, soul, None);
    pass_pair(&mut game);
    target(&mut game, creature);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(life(&game, player), 20);
    assert_eq!(stats(&game, mortipede), (3, 4));
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, activation(orb));
    for card in fuel {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseEvidence { object });
    }
    act(&mut game, Action::FinishEvidence);
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    assert_eq!(game.state.stack.len(), 3);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(life(&game, player), 21);
    assert_eq!(life(&game, opponent), 19);
    assert_eq!(stats(&game, mortipede), (4, 4));
    let granted = Keywords::MENACE.union(Keywords::LIFELINK);
    assert!(game.keywords(mortipede).contains(granted));
    pass_pair(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    enter_step(&mut game, Step::End);
    pass_pair(&mut game);
    assert_eq!(stats(&game, mortipede), (3, 4));
    assert!(!game.keywords(mortipede).contains(granted));
}

#[test]
fn slimy_dualleech_rechecks_target_power_and_control() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Slimy Dualleech", Zone::Battlefield);
    let small = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let growth = put(&mut game, opponent, "Giant Growth", Zone::Hand);
    enter_step(&mut game, Step::BeginCombat);
    for card in [large, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target(&mut game, small);
    let mut changed = game.clone();
    act(&mut changed, Action::Pass);
    cast(&mut changed, growth, Some(Target::Card(small)));
    settle(&mut changed);
    assert_eq!(stats(&changed, small), (5, 5));
    assert!(!changed.keywords(small).contains(Keywords::DEATHTOUCH));

    settle(&mut game);
    assert_eq!(stats(&game, small), (3, 2));
    assert!(game.keywords(small).contains(Keywords::DEATHTOUCH));
}

#[test]
fn undercity_eliminator_can_sacrifice_itself_before_choosing_the_exile_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Undercity Eliminator", Zone::Hand);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert!(declined.on_battlefield(source) && declined.on_battlefield(enemy));
    let object = game.object_ref(land);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let object = game.object_ref(source);
    act(&mut game, Action::ChooseAdditionalCost { object });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    target(&mut game, enemy);
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
}

#[test]
fn unscrupulous_agent_lets_the_opponent_privately_choose_a_land_to_exile() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Unscrupulous Agent", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let keep = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    target_player(&mut game, opponent);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).hand_choice.is_none());
    reject(&mut game, Action::ChooseHandCard { object: None });
    let object = Some(game.object_ref(land));
    act(&mut game, Action::ChooseHandCard { object });
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(keep).zone, Zone::Hand);
    assert!(game.observe(player).known_hand.is_empty());
}
