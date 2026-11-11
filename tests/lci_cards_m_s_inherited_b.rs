//! Focused outcomes for the inherited Panicked Altisaur through Seeker of Sunlight.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn mana(card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn reach(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing decision {:?}", game.state.pending);
}

fn reach_action(game: &mut Game, action: &Action) {
    reach(game, |game| game.legal_actions().contains(action));
}

fn attack(game: &mut Game, card: CardId) {
    reach(game, |game| game.state.pending == Pending::Attackers);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

#[test]
fn altisaur_taps_to_deal_two_damage_only_to_the_opponent() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let card = put(&mut game, player, "Panicked Altisaur", Zone::Battlefield);
        assert!(game.keywords(card).contains(Keywords::REACH));
        let mut sick = game.clone();
        sick.state.objects[card as usize].summoning_sick = true;
        reject(&mut sick, activate(card, 0));
        act(&mut game, activate(card, 0));
        pay(&mut game);
        assert!(game.object_view(card).tapped);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE);
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE - 2
        );
        reject(&mut game, activate(card, 0));
    }
}

#[test]
fn pirate_hat_has_two_equip_costs_and_its_bearer_draws_then_discards_on_attack() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let hat = put(&mut game, player, "Pirate Hat", Zone::Battlefield);
        let pirate = put(&mut game, player, "Plundering Pirate", Zone::Battlefield);
        let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(
            &mut game,
            player.other(),
            "Plundering Pirate",
            Zone::Battlefield,
        );
        let draw = put(&mut game, player, "Shock", Zone::Library);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut game, activate(hat, 1));
        let mut nonpirate = game.clone();
        nonpirate.state.players[player.index()].mana[5] = 2;
        act(&mut nonpirate, activate(hat, 1));
        act(&mut nonpirate, target(other));
        pay(&mut nonpirate);
        settle(&mut nonpirate);
        assert_eq!(stats(&nonpirate, other), (3, 3));

        act(&mut game, activate(hat, 0));
        reject(&mut game, target(other));
        reject(&mut game, target(foe));
        act(&mut game, target(pirate));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, pirate), (4, 3));
        attack(&mut game, pirate);
        let discard = Action::Discard { card: draw };
        reach_action(&mut game, &discard);
        assert_eq!(game.object_view(draw).zone, Zone::Hand);
        act(&mut game, discard);
        settle(&mut game);
        assert_eq!(game.object_view(draw).zone, Zone::Graveyard);
    }
}

#[test]
fn poison_dart_frog_produces_each_color_and_can_gain_deathtouch_while_tapped() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let frog = put(&mut game, player, "Poison Dart Frog", Zone::Battlefield);
        assert!(game.keywords(frog).contains(Keywords::REACH));
        reject(&mut game, mana(frog, 5));
        for color in 0..5 {
            let mut branch = game.clone();
            let before = branch.state.players[player.index()].mana[color];
            act(&mut branch, mana(frog, color));
            assert_eq!(branch.state.players[player.index()].mana[color], before + 1);
            assert!(branch.object_view(frog).tapped);
        }
        act(&mut game, mana(frog, 4));
        act(&mut game, activate(frog, 1));
        pay(&mut game);
        settle(&mut game);
        assert!(game.keywords(frog).contains(Keywords::DEATHTOUCH));
        game.begin_step(Step::End);
        reach(&mut game, |game| game.state.active == player.other());
        assert!(!game.keywords(frog).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn promising_vein_taps_for_colorless_or_sacrifices_to_find_a_tapped_basic() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vein = put(&mut game, player, "Promising Vein", Zone::Hand);
    let basic = put(&mut game, player, "Plains", Zone::Library);
    let nonbasic = put(&mut game, player, "Breeding Pool", Zone::Library);
    act(&mut game, Action::PlayLand { card: vein });
    assert!(!game.object_view(vein).tapped);
    let mut tap = game.clone();
    let before = tap.state.players[player.index()].mana[5];
    act(&mut tap, mana(vein, 5));
    assert_eq!(tap.state.players[player.index()].mana[5], before + 1);
    reject(&mut tap, activate(vein, 1));

    act(&mut game, activate(vein, 1));
    pay(&mut game);
    assert_eq!(game.object_view(vein).zone, Zone::Graveyard);
    let choose_basic = Action::ChooseSearch { card: basic };
    reach_action(&mut game, &choose_basic);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, choose_basic);
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
}

#[test]
fn plains_and_mountain_are_untapped_basic_sources_of_the_printed_mana() {
    for (name, color) in [("Plains", 0), ("Mountain", 3)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).card.is_basic_land());
        assert!(game.has_subtype(land, name));
        assert!(!game.object_view(land).tapped);
        let before = game.state.players[player.index()].mana[color];
        act(&mut game, Action::TapMana { card: land });
        assert_eq!(game.state.players[player.index()].mana[color], before + 1);
    }
}

#[test]
fn quicksand_locks_the_tapped_discount_but_exiles_an_untapped_creature() {
    for player in [Player::P0, Player::P1] {
        for tapped in [true, false] {
            let mut game = scenario(player);
            let target_creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut game, player, "Quicksand Whirlpool", Zone::Hand);
            game.state.objects[target_creature as usize].tapped = tapped;
            act(&mut game, Action::AnnounceCast { card: spell });
            act(&mut game, target(target_creature));
            let cost = if tapped { 2 } else { 5 };
            assert_eq!(game.observe(player).payment_request.unwrap().generic, cost);
            act(
                &mut game,
                Action::PayCosts {
                    payment: [1, 0, 0, 0, 0, cost],
                },
            );
            game.untap_objects(&[target_creature], untap_rules::UntapCause::Effect);
            settle(&mut game);
            assert_eq!(game.object_view(target_creature).zone, Zone::Exile);
        }
    }
}

#[test]
fn quintorius_creates_a_spirit_discovers_and_exiles_only_its_own_graveyard_targets() {
    let player = Player::P0;
    let mut root = scenario(player);
    let walker = put(&mut root, player, "Quintorius Kand", Zone::Hand);
    let discovered = put(&mut root, player, "Shock", Zone::Library);
    let too_large = put(&mut root, player, "Malamet Veteran", Zone::Library);
    let spell = put(&mut root, player, "Shock", Zone::Graveyard);
    let land = put(&mut root, player, "Mountain", Zone::Graveyard);
    let foreign = put(&mut root, player.other(), "Forest", Zone::Graveyard);
    cast(&mut root, walker, None);
    settle(&mut root);
    assert_eq!(root.counter_count(walker, CounterKind::Loyalty), 4);

    let mut create = root.clone();
    act(&mut create, activate(walker, 0));
    pay(&mut create);
    settle(&mut create);
    assert_eq!(create.counter_count(walker, CounterKind::Loyalty), 5);
    let spirit = create
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Spirit")
        })
        .unwrap();
    assert_eq!((spirit.power, spirit.toughness), (3, 2));
    assert_eq!(spirit.colors, 1 | 8);
    reject(&mut create, activate(walker, 1));

    let mut discover = root.clone();
    act(&mut discover, activate(walker, 1));
    pay(&mut discover);
    reach_action(&mut discover, &Action::FinishResolutionCasts);
    let cascade = discover.observe(player.other()).cascade.unwrap();
    // The public cascade view uses an exclusive upper bound.
    assert_eq!(cascade.maximum_mana_value, 5);
    assert!(cascade.exiled.iter().any(|object| object.card == too_large));
    assert_eq!(cascade.discovered.unwrap().card, discovered);
    let mut decline = discover.clone();
    act(&mut decline, Action::FinishResolutionCasts);
    settle(&mut decline);
    assert_eq!(decline.object_view(discovered).zone, Zone::Hand);
    assert_eq!(decline.object_view(too_large).zone, Zone::Library);
    assert_eq!(decline.state.players[player.index()].life, STARTING_LIFE);

    choose(&mut discover, |action| {
        matches!(
            action, Action::CastExiledDuringResolution { object, .. } if object.card == discovered
        )
    });
    act(
        &mut discover,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    if matches!(discover.state.pending, Pending::PayCosts { .. }) {
        pay(&mut discover);
    }
    if matches!(discover.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut discover, Action::FinishResolutionCasts);
    }
    settle(&mut discover);
    assert_eq!(
        discover.state.players[player.index()].life,
        STARTING_LIFE + 2
    );
    assert_eq!(
        discover.state.players[player.other().index()].life,
        STARTING_LIFE - 4
    );

    root.add_counter(walker, CounterKind::Loyalty, 2);
    let before = root.state.players[player.index()].mana[3];
    act(&mut root, activate(walker, 2));
    reject(&mut root, target(foreign));
    act(&mut root, target(spell));
    act(&mut root, target(land));
    act(&mut root, Action::FinishTargets);
    pay(&mut root);
    settle(&mut root);
    assert_eq!(root.state.players[player.index()].mana[3], before + 2);
    for card in [spell, land] {
        assert_eq!(root.object_view(card).zone, Zone::Exile);
    }
    act(&mut root, Action::PlayLand { card: land });
    assert_eq!(root.object_view(land).zone, Zone::Battlefield);
    act(&mut root, Action::AnnounceExile { card: spell });
    act(
        &mut root,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut root);
    settle(&mut root);
    assert_eq!(
        root.state.players[player.other().index()].life,
        STARTING_LIFE - 2
    );
}

#[test]
fn ceratops_requires_three_blockers_at_the_commit_boundary() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let ceratops = put(&mut game, player, "Rampaging Ceratops", Zone::Battlefield);
        let blockers =
            [0; 3].map(|_| put(&mut game, player.other(), "Bear Cub", Zone::Battlefield));
        attack(&mut game, ceratops);
        reach(&mut game, |game| game.state.pending == Pending::Blockers);
        for (count, blocker) in blockers.into_iter().enumerate() {
            act(
                &mut game,
                Action::Block {
                    blocker,
                    attacker: ceratops,
                },
            );
            if count < 2 {
                reject(&mut game, Action::FinishBlockers);
            }
        }
        act(&mut game, Action::FinishBlockers);
        assert_ne!(game.state.pending, Pending::Blockers);
    }
}

#[test]
fn spiketail_grants_power_and_indestructible_to_a_controlled_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spiketail = put(&mut game, player, "Rampaging Spiketail", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, spiketail, None);
    reach_action(&mut game, &target(ally));
    reject(&mut game, target(foe));
    act(&mut game, target(ally));
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 2));
    assert!(game.keywords(ally).contains(Keywords::INDESTRUCTIBLE));
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(ally)));
    settle(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
}

#[test]
fn ray_can_exile_an_uncrewed_vehicle_or_nonbasic_land_then_scry() {
    let player = Player::P1;
    let mut root = scenario(player);
    let ray = put(&mut root, player, "Ray of Ruin", Zone::Hand);
    let vehicle = put(
        &mut root,
        player.other(),
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let creature = put(&mut root, player.other(), "Bear Cub", Zone::Battlefield);
    let nonbasic = put(
        &mut root,
        player.other(),
        "Promising Vein",
        Zone::Battlefield,
    );
    let basic = put(&mut root, player.other(), "Forest", Zone::Battlefield);
    let top = put(&mut root, player, "Shock", Zone::Library);
    act(&mut root, Action::AnnounceCast { card: ray });
    reject(&mut root, target(basic));
    for card in [vehicle, creature, nonbasic] {
        let mut game = root.clone();
        act(&mut game, target(card));
        pay(&mut game);
        let scry = Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        };
        reach_action(&mut game, &scry);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        act(&mut game, scry);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].library.first(),
            Some(&top)
        );
    }
}

#[test]
fn rumbling_rockslide_reads_the_current_land_count_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Rumbling Rockslide", Zone::Hand);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    announce(&mut game, spell, &[Target::Card(creature)]);
    let mut smaller = game.clone();
    smaller.move_object(land, Zone::Graveyard);
    settle(&mut smaller);
    assert_eq!(smaller.object_view(creature).damage, 2);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn runaway_boulder_flashes_in_for_six_damage_or_cycles_for_one_card() {
    let player = Player::P0;
    let mut game = scenario(player.other());
    let boulder = put(&mut game, player, "Runaway Boulder", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let draw = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, Action::Pass);
    let mut cycle = game.clone();
    cycle.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut cycle, activate(boulder, 0));
    pay(&mut cycle);
    assert_eq!(cycle.object_view(boulder).zone, Zone::Graveyard);
    settle(&mut cycle);
    assert_eq!(cycle.object_view(draw).zone, Zone::Hand);

    cast(&mut game, boulder, None);
    reach_action(&mut game, &target(foe));
    reject(&mut game, target(ally));
    act(&mut game, target(foe));
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ally).damage, 0);
}

#[test]
fn screaming_phantom_mills_the_actors_top_card_on_declaration() {
    let player = Player::P1;
    let mut game = scenario(player);
    let phantom = put(&mut game, player, "Screaming Phantom", Zone::Battlefield);
    let top = put(&mut game, player, "Forest", Zone::Library);
    let enemy_top = put(&mut game, player.other(), "Forest", Zone::Library);
    assert!(game.keywords(phantom).contains(Keywords::FLYING));
    attack(&mut game, phantom);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enemy_top).zone, Zone::Library);
}

#[test]
fn seeker_pays_two_and_green_to_explore_at_sorcery_speed() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let seeker = put(&mut game, player, "Seeker of Sunlight", Zone::Battlefield);
        let nonland = put(&mut game, player, "Shock", Zone::Library);
        let mut upkeep = game.clone();
        upkeep.begin_step(Step::Upkeep);
        reject(&mut upkeep, activate(seeker, 0));
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut game, activate(seeker, 0));
        game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
        act(&mut game, activate(seeker, 0));
        pay(&mut game);
        let explore = Action::ChooseExplore {
            object: game.object_ref(nonland),
            graveyard: true,
        };
        reach_action(&mut game, &explore);
        act(&mut game, explore);
        settle(&mut game);
        assert_eq!(stats(&game, seeker), (2, 2));
        assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
        assert!(!game.object_view(seeker).tapped);
    }
}
