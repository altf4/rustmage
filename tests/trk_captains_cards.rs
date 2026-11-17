//! Actual typed Star Trek Captain slice, with legal actions and owned branches.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn until(game: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if done(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Captain action budget at {:?}", game.state.pending);
}

fn complete(game: &mut Game) {
    until(game, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}

fn token_count(game: &Game, player: Player, token: Card) -> usize {
    game.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.controller == player && o.card == token && o.is_token()
        })
        .count()
}

#[test]
fn kirk_captures_one_or_three_modes_at_trigger_and_resolves_all_three_in_order() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let kirk = owned(&mut game, player, "Captain James T. Kirk", Zone::Hand);
    cast(&mut game, kirk, None);
    until(&mut game, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseMode { mode: 6 })
    );
    let saved = game.snapshot();
    let mut simple = game.clone();
    act(&mut simple, Action::ChooseMode { mode: 1 });
    complete(&mut simple);
    assert_eq!(token_count(&simple, player, Card(6008)), 1);
    assert_eq!(stats(&simple, bear), (2, 2));
    game.restore(&saved).unwrap();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::ChooseMode { mode: 6 })
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, Action::ChooseMode { mode: 6 });
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6008)), 1);
    let officer = game
        .state
        .objects
        .iter()
        .position(|o| o.card == Card(6008) && o.zone == Zone::Battlefield)
        .unwrap() as CardId;
    assert_eq!(stats(&game, officer), (2, 1));
    assert_eq!(stats(&game, bear), (3, 2));
    assert_eq!(stats(&game, kirk), (3, 2));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.state.objects[kirk as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: kirk });
    act(&mut game, Action::FinishAttackers);
    until(&mut game, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    reject(&mut game, Action::ChooseMode { mode: 6 });
    act(&mut game, Action::ChooseMode { mode: 2 });
    complete(&mut game);
    assert_eq!(stats(&game, bear), (4, 2));

    let mut hand = scenario(player);
    let cast_kirk = owned(&mut hand, player, "Captain James T. Kirk", Zone::Hand);
    let discard_card = put(&mut hand, player, "Forest", Zone::Hand);
    cast(&mut hand, cast_kirk, None);
    until(&mut hand, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    reject(&mut hand, Action::ChooseMode { mode: 6 });
    act(&mut hand, Action::ChooseMode { mode: 0 });
    until(&mut hand, |g| {
        g.legal_actions()
            .contains(&Action::Discard { card: discard_card })
    });
    act(&mut hand, Action::Discard { card: discard_card });
    complete(&mut hand);
    assert_eq!(hand.object_view(discard_card).zone, Zone::Graveyard);
    assert_eq!(hand.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn archer_creates_porthos_and_federation_targets_only_owned_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let archer = owned(&mut game, player, "Captain Jonathan Archer", Zone::Hand);
    cast(&mut game, archer, None);
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6013)), 1);
    game.state.objects[archer as usize].summoning_sick = false;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: archer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    complete(&mut game);
    assert_eq!(stats(&game, bear), (6, 6));
    assert!(game.object_view(archer).tapped);
}

#[test]
fn janeway_explores_herself_and_another_entrant_and_grants_a_second_land_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    let janeway = owned(&mut game, player, "Captain Kathryn Janeway", Zone::Hand);
    let first = put(&mut game, player, "Forest", Zone::Hand);
    let second = put(&mut game, player, "Island", Zone::Hand);
    let nonland = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, janeway, None);
    until(&mut game, |g| g.observe(player).explore_choice.is_some());
    let reveal = game.observe(player).explore_choice.unwrap();
    assert_eq!(reveal.explorer.object, game.object_ref(janeway));
    assert_eq!(reveal.revealed.object, game.object_ref(nonland));
    assert!(reveal.counter_added);
    act(
        &mut game,
        Action::ChooseExplore {
            object: reveal.revealed.object,
            graveyard: true,
        },
    );
    complete(&mut game);
    assert_eq!(game.counter_count(janeway, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
    act(&mut game, Action::PlayLand { card: first });
    act(&mut game, Action::PlayLand { card: second });
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    let entrant = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    cast(&mut game, entrant, None);
    until(&mut game, |g| {
        g.observe(player)
            .last_explore
            .as_ref()
            .is_some_and(|e| e.sequence == 2)
    });
    complete(&mut game);
    assert_eq!(
        game.observe(player).last_explore.unwrap().explorer.object,
        game.object_ref(entrant)
    );
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 0);
}

#[test]
fn burnham_counts_resolutions_and_pike_exposes_only_its_controllers_library_top() {
    let player = Player::P0;
    let mut game = scenario(player);
    let burnham = owned(
        &mut game,
        player,
        "Captain Michael Burnham",
        Zone::Battlefield,
    );
    let pike = owned(
        &mut game,
        player,
        "Captain Pike, Fullness of Time",
        Zone::Battlefield,
    );
    let base = put(&mut game, player, "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, pike), (1, 1));
    assert!(game.keywords(burnham).contains(Keywords::TRAMPLE));
    let first = put(&mut game, player, "Island", Zone::Hand);
    let second = put(&mut game, player, "Mountain", Zone::Hand);
    let top = put(&mut game, player, "Plains", Zone::Library);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object,
        game.object_ref(top)
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    act(&mut game, Action::PlayLand { card: first });
    complete(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    assert_eq!(game.counter_count(burnham, CounterKind::PlusOne), 0);
    game.state.land_played = false; // A second legal land entry in this turn.
    act(&mut game, Action::PlayLand { card: second });
    complete(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    assert_eq!(game.counter_count(burnham, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, pike), (3, 3));
    game.state.land_played = false;
    act(&mut game, Action::PlayLand { card: top });
    complete(&mut game);
    assert_eq!(stats(&game, pike), (4, 4));
    assert_eq!(game.counter_count(burnham, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(base).zone, Zone::Battlefield);
    game.move_object(pike, Zone::Hand);
    assert_eq!(stats(&game, pike), (4, 4));
}

#[test]
fn shuttle_stations_from_another_creature_and_still_has_its_mana_ability() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = owned(&mut game, player, "Class-F Shuttlecraft", Zone::Battlefield);
    let donor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    assert!(!game.is_creature(ship));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: ship,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let opposing_ref = game.object_ref(opposing);
    reject(
        &mut game,
        Action::ChooseAdditionalCost {
            object: opposing_ref,
        },
    );
    let saved = game.snapshot();
    let donor_ref = game.object_ref(donor);
    let chosen = Action::ChooseAdditionalCost { object: donor_ref };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == chosen)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, chosen);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    pay(&mut game);
    complete(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    assert!(!game.is_creature(ship));
    game.restore(&saved).unwrap();
    act(
        &mut game,
        Action::ChooseAdditionalCost { object: donor_ref },
    );
    pay(&mut game);
    complete(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    game.state.objects[donor as usize].tapped = false;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: ship,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseAdditionalCost { object: donor_ref },
    );
    pay(&mut game);
    complete(&mut game);
    assert!(game.is_creature(ship));
    assert_eq!(stats(&game, ship), (2, 4));
    assert!(
        game.keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.state.players[player.index()].mana = [0; 6];
    let production = [0, 0, 0, 1, 0, 0];
    act(
        &mut game,
        Action::ActivateMana {
            card: ship,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
    assert!(game.object_view(ship).tapped);
    assert_eq!(game.state.players[player.index()].mana, production);
}

#[test]
fn cloak_and_phaser_reduces_generic_cost_and_cold_blooded_crew_counters_any_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Island", Zone::Battlefield);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let crew = owned(&mut game, player, "Cold-Blooded Crew", Zone::Hand);
    cast(&mut game, crew, None);
    until(&mut game, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(victim),
        })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    complete(&mut game);
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 2);
    assert!(
        game.keywords(crew)
            .contains(Keywords::NO_POWER_LE_TWO_BLOCKERS)
    );
    let cloak = owned(&mut game, player, "Cloak and Phaser", Zone::Hand);
    game.state.players[player.index()].mana = [6, 1, 0, 0, 0, 0];
    let drawn = put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, cloak, None);
    until(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Discard { .. }))
    });
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard);
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6012)), 2);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_ne!(game.object_view(drawn).zone, Zone::Library);
}

#[test]
fn cold_blooded_crew_basic_landcycles_from_hand_and_restricts_small_blockers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let crew = owned(&mut game, player, "Cold-Blooded Crew", Zone::Hand);
    let basic = put(&mut game, player, "Island", Zone::Library);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: crew,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    until(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    assert!(game.observe(player.other()).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: crew });
    let saved = game.snapshot();
    let choice = Action::ChooseSearch { card: basic };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choice)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, choice);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(&mut game, Action::FinishSearch);
    complete(&mut game);
    assert_eq!(game.object_view(crew).zone, Zone::Graveyard);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::ChooseSearch { card: basic });
    act(&mut indexed, Action::FinishSearch);
    complete(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let mut combat = scenario(player);
    let attacker = owned(&mut combat, player, "Cold-Blooded Crew", Zone::Battlefield);
    let small = put(&mut combat, player.other(), "Bear Cub", Zone::Battlefield);
    let big = put(&mut combat, player.other(), "Hill Giant", Zone::Battlefield);
    combat.state.step = Step::Attackers;
    combat.state.pending = Pending::Attackers;
    act(&mut combat, Action::Attack { card: attacker });
    act(&mut combat, Action::FinishAttackers);
    act(&mut combat, Action::Pass);
    act(&mut combat, Action::Pass);
    assert_eq!(combat.state.pending, Pending::Blockers);
    reject(
        &mut combat,
        Action::Block {
            blocker: small,
            attacker,
        },
    );
    act(
        &mut combat,
        Action::Block {
            blocker: big,
            attacker,
        },
    );
}

const DILEMMA: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Choose no modes",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Gain one life",
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
    ]),
    mode_counts: Some(&[0, 1]),
    ..CardRules::EMPTY
};

#[test]
fn sisko_counts_a_real_dilemma_and_casts_only_its_exact_exiled_spells() {
    let _helper = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DILEMMA)]);
    let player = Player::P0;
    let mut none = scenario(player);
    owned(
        &mut none,
        player,
        "Captain Benjamin Sisko",
        Zone::Battlefield,
    );
    none.begin_step(Step::End);
    none.prepare_triggers();
    complete(&mut none);
    assert_eq!(token_count(&none, player, Card(6008)), 0);
    let mut game = scenario(player);
    owned(
        &mut game,
        player,
        "Captain Benjamin Sisko",
        Zone::Battlefield,
    );
    let modal = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: modal });
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    complete(&mut game);
    assert_eq!(
        game.state.players[player.index()].dilemmas_faced_this_turn,
        1
    );
    game.begin_step(Step::End);
    game.prepare_triggers();
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6008)), 1);

    let mut casts = scenario(player);
    let sisko = owned(
        &mut casts,
        player,
        "Captain Benjamin Sisko",
        Zone::Battlefield,
    );
    let officer = put(&mut casts, player, "Tactical Officer", Zone::Battlefield);
    let unrelated = put(&mut casts, player, "Grizzly Bears", Zone::Exile);
    let land = put(&mut casts, player, "Forest", Zone::Library);
    let giant = put(&mut casts, player, "Hill Giant", Zone::Library);
    let bear = put(&mut casts, player, "Grizzly Bears", Zone::Library);
    let first = put(&mut casts, player, "Forest", Zone::Library);
    let second = put(&mut casts, player, "Forest", Zone::Library);
    act(
        &mut casts,
        Action::AnnounceActivate {
            card: sisko,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut casts);
    until(&mut casts, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert!(casts.object_view(sisko).tapped);
    assert_eq!(casts.object_view(land).zone, Zone::Exile);
    assert_eq!(casts.object_view(first).zone, Zone::Exile);
    assert_eq!(casts.object_view(second).zone, Zone::Exile);
    let giant_ref = casts.object_ref(giant);
    let unrelated_ref = casts.object_ref(unrelated);
    reject(
        &mut casts,
        Action::CastExiledDuringResolution {
            object: giant_ref,
            face: CastFace::Normal,
        },
    );
    reject(
        &mut casts,
        Action::CastExiledDuringResolution {
            object: unrelated_ref,
            face: CastFace::Normal,
        },
    );
    let free = Action::CastExiledDuringResolution {
        object: casts.object_ref(bear),
        face: CastFace::Normal,
    };
    let saved = casts.snapshot();
    let index = casts
        .legal_actions()
        .iter()
        .position(|a| *a == free)
        .unwrap();
    let mut indexed = casts.clone();
    act(&mut casts, free);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(casts.snapshot(), indexed.snapshot());
    if matches!(casts.state.pending, Pending::PayCosts { .. }) {
        pay(&mut casts);
    }
    act(&mut casts, Action::FinishResolutionCasts);
    complete(&mut casts);
    assert_eq!(casts.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(casts.object_view(giant).zone, Zone::Exile);
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(casts.object_view(officer).zone, Zone::Battlefield);
}

#[test]
fn picard_buffs_other_creatures_observes_creature_and_spacecraft_casts_and_searches() {
    let player = Player::P0;
    let mut game = scenario(player);
    let picard = owned(
        &mut game,
        player,
        "Captain Jean-Luc Picard",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&game, picard), (2, 3));
    assert_eq!(stats(&game, bear), (3, 3));
    let second_bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, second_bear, None);
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6008)), 1);
    let ship_spell = owned(&mut game, player, "Class-F Shuttlecraft", Zone::Hand);
    cast(&mut game, ship_spell, None);
    complete(&mut game);
    assert_eq!(token_count(&game, player, Card(6008)), 2);

    let mut search = scenario(player);
    let picard = owned(
        &mut search,
        player,
        "Captain Jean-Luc Picard",
        Zone::Battlefield,
    );
    let mut officers: Vec<_> = (0..5)
        .map(|_| put(&mut search, player, "Tactical Officer", Zone::Battlefield))
        .collect();
    let opposing = put(
        &mut search,
        player.other(),
        "Tactical Officer",
        Zone::Battlefield,
    );
    put(&mut search, player, "Doubling Season", Zone::Battlefield);
    let ship = owned(&mut search, player, "Class-F Shuttlecraft", Zone::Library);
    admit(
        &search,
        Action::AnnounceActivate {
            card: picard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    officers.push(put(
        &mut search,
        player,
        "Tactical Officer",
        Zone::Battlefield,
    ));
    act(
        &mut search,
        Action::AnnounceActivate {
            card: picard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let opposing_ref = search.object_ref(opposing);
    reject(
        &mut search,
        Action::ChooseAdditionalCost {
            object: opposing_ref,
        },
    );
    for id in std::iter::once(picard).chain(officers.iter().copied()) {
        let object = search.object_ref(id);
        act(&mut search, Action::ChooseAdditionalCost { object });
    }
    pay(&mut search);
    until(&mut search, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    assert!(search.observe(player.other()).search_choice.is_none());
    act(&mut search, Action::ChooseSearch { card: ship });
    act(&mut search, Action::FinishSearch);
    complete(&mut search);
    assert_eq!(search.object_view(ship).zone, Zone::Battlefield);
    assert_eq!(search.counter_count(ship, CounterKind::Charge), 14);
    assert!(search.is_creature(ship));
    assert_eq!(stats(&search, ship), (3, 5));
    assert!(
        search
            .keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
}

#[test]
fn combadge_cast_copy_equips_two_creatures_and_grants_attack_evasion() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first_bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second_bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let original = owned(&mut game, player, "Combadge", Zone::Hand);
    cast(&mut game, original, None);
    let mut removed_original = game.clone();
    removed_original.move_object(original, Zone::Graveyard);
    complete(&mut removed_original);
    assert_eq!(
        token_count(
            &removed_original,
            player,
            Card::from_name("Combadge").unwrap()
        ),
        1
    );
    complete(&mut game);
    let badges: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == Card::from_name("Combadge").unwrap())
                .then_some(id as CardId)
        })
        .collect();
    assert_eq!(badges.len(), 2);
    assert_eq!(
        badges
            .iter()
            .filter(|&&id| game.state.objects[id as usize].is_token())
            .count(),
        1
    );
    for (badge, creature) in [(badges[0], first_bear), (badges[1], second_bear)] {
        act(
            &mut game,
            Action::AnnounceActivate {
                card: badge,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        complete(&mut game);
        assert_eq!(
            game.state.objects[badge as usize].attached_to,
            Some(game.object_ref(creature))
        );
    }
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: first_bear });
    act(&mut game, Action::FinishAttackers);
    complete(&mut game);
    assert!(
        game.keywords(first_bear)
            .contains(Keywords::CANT_BE_BLOCKED)
    );
}
