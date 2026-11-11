//! Live counts, characteristic effects, and resolved grants from the authored cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, p, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn advance(game: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..16 {
        if stop(game) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("expected a resolution decision");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn sinuous_benthisaur_selects_two_privately_from_live_caves_in_both_zones() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Sinuous Benthisaur", Zone::Hand);
    put(&mut game, p, "Hidden Cataract", Zone::Battlefield);
    put(&mut game, p, "Hidden Volcano", Zone::Graveyard);
    let added = put(&mut game, p, "Hidden Necropolis", Zone::Hand);
    put(&mut game, p.other(), "Hidden Cataract", Zone::Battlefield);
    put(&mut game, p, "Forest", Zone::Graveyard);
    let a = put(&mut game, p, "Shock", Zone::Library);
    let b = put(&mut game, p, "Island", Zone::Library);
    let c = put(&mut game, p, "Bear Cub", Zone::Library);
    cast(&mut game, source, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    game.move_object(added, Zone::Graveyard);
    advance(&mut game, |g| g.observe(p).library_selection.is_some());
    let view = game.observe(p).library_selection.unwrap();
    assert_eq!(
        (view.minimum, view.maximum, view.remaining.len()),
        (2, 2, 3)
    );
    assert!(game.observe(p.other()).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: b });
    reject(&mut game, Action::FinishLibrarySelection);
    act(&mut game, Action::SelectLibrary { card: c });
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(game.object_view(b).zone, Zone::Hand);
    assert_eq!(game.object_view(c).zone, Zone::Hand);
    assert_eq!(game.state.players[p.index()].library.first(), Some(&a));
}

#[test]
fn song_of_stupefaction_enchants_a_vehicle_and_uses_live_graveyard_power() {
    let p = Player::P0;
    let mut game = scenario(p);
    let song = candidate(&mut game, p, "Song of Stupefaction", Zone::Hand);
    let vehicle = put(&mut game, p, "Magmatic Galleon", Zone::Battlefield);
    let generic_artifact = put(&mut game, p, "Candy Trail", Zone::Battlefield);
    let permanent = put(&mut game, p, "Forest", Zone::Library);
    let spell = put(&mut game, p, "Opt", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: song });
    reject(&mut game, target(generic_artifact));
    act(&mut game, target(vehicle));
    finish_targets(&mut game);
    pay(&mut game);
    advance(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(permanent).zone, Zone::Library);
    assert_eq!(
        decline.object_view(song).attached_to,
        Some(decline.object_ref(vehicle))
    );

    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let crewer = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    act(&mut game, activate(vehicle, 0));
    let object = game.object_ref(crewer);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    let power = stats(&game, vehicle);
    assert_eq!(power, (4, 5));
    game.move_object(permanent, Zone::Exile);
    assert_eq!(stats(&game, vehicle), (power.0 + 1, power.1));
}

#[test]
fn souls_of_the_lost_pays_either_cost_and_recounts_its_own_graveyard() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Souls of the Lost", Zone::Hand);
    let action = Action::AnnounceCast { card: source };
    reject(&mut game, action.clone());
    let discard = put(&mut game, p, "Opt", Zone::Hand);
    let sacrifice = put(&mut game, p, "Forest", Zone::Battlefield);
    for chosen in [discard, sacrifice] {
        let mut branch = game.clone();
        act(&mut branch, action.clone());
        assert!(matches!(
            branch.state.pending,
            Pending::AdditionalCost { .. }
        ));
        let object = branch.object_ref(chosen);
        act(&mut branch, Action::ChooseAdditionalCost { object });
        if matches!(branch.state.pending, Pending::CastCosts { .. }) {
            choose(&mut branch, |a| matches!(a, Action::ChooseCastCost { .. }));
        }
        pay(&mut branch);
        settle(&mut branch);
        let count = i128::from(chosen == sacrifice);
        assert_eq!(branch.object_view(chosen).zone, Zone::Graveyard);
        assert_eq!(stats(&branch, source), (count, count + 1));
        branch.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&branch, source), (count + 1, count + 2));
    }
}

#[test]
fn tendril_puts_seven_counters_then_preserves_the_animated_land() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Tendril of the Mycotyrant", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let foe = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    let nonland = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    act(&mut game, activate(source, 0));
    reject(&mut game, target(foe));
    reject(&mut game, target(nonland));
    act(&mut game, target(land));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, land), (7, 7));
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 7);
    assert!(game.has_subtype(land, "Forest") && game.has_subtype(land, "Fungus"));
    assert!(game.keywords(land).contains(Keywords::HASTE));
    assert_eq!(
        game.object_kinds(land, &game.state.objects[land as usize]) & 3,
        3
    );
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, land), (7, 7));
    assert!(game.has_subtype(land, "Fungus"));
}

#[test]
fn ancient_one_requires_eight_permanents_and_mills_by_exact_discard_value() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "The Ancient One", Zone::Battlefield);
    for _ in 0..7 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    put(&mut game, p, "Opt", Zone::Graveyard);
    let mut before = game.clone();
    before.state.step = Step::Attackers;
    before.state.pending = Pending::Attackers;
    reject(&mut before, Action::Attack { card: source });
    put(&mut before, p, "Short Bow", Zone::Graveyard);
    act(&mut before, Action::Attack { card: source });

    let discarded = put(&mut game, p, "Hill Giant", Zone::Library);
    let opponent_before = game.state.players[p.other().index()].library.len();
    act(&mut game, activate(source, 0));
    pay(&mut game);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(game.object_view(discarded).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: discarded });
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].library.len(),
        opponent_before - 4
    );
    assert_eq!(stats(&game, source), (8, 8));
}

#[test]
fn the_belligerent_crews_and_grants_private_normal_play_after_it_leaves() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "The Belligerent", Zone::Battlefield);
    let crewer = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let second_land = put(&mut game, p, "Island", Zone::Library);
    let spell = put(&mut game, p, "Shock", Zone::Library);
    let land = put(&mut game, p, "Mountain", Zone::Library);
    reject(&mut game, Action::PlayLand { card: land });
    act(&mut game, activate(source, 0));
    let object = game.object_ref(crewer);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert_eq!(stats(&game, source), (5, 5));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.observe(p).known_library_top[p.index()].unwrap().object,
        game.object_ref(land)
    );
    assert!(game.observe(p.other()).known_library_top[p.index()].is_none());
    assert_eq!(
        game.observe(p)
            .objects
            .iter()
            .filter(|o| o.card == Card(5302) && o.zone == Zone::Battlefield)
            .count(),
        1
    );
    game.state.step = Step::Main2;
    act(&mut game, Action::PlayLand { card: land });
    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    announce(&mut game, spell, &[Target::Player(p.other())]);
    settle(&mut game);
    reject(&mut game, Action::PlayLand { card: second_land });
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 2
    );
}

#[test]
fn zoetic_glyph_adds_golem_and_discovers_when_its_attachment_dies() {
    let p = Player::P0;
    let mut game = scenario(p);
    let artifact = put(&mut game, p, "Candy Trail", Zone::Battlefield);
    let spell = candidate(&mut game, p, "Zoetic Glyph", Zone::Hand);
    let hit = put(&mut game, p, "Bear Cub", Zone::Library);
    let land = put(&mut game, p, "Forest", Zone::Library);
    cast(&mut game, spell, Some(Target::Card(artifact)));
    settle(&mut game);
    assert_eq!(stats(&game, artifact), (5, 4));
    assert!(game.has_subtype(artifact, "Golem") && game.has_subtype(artifact, "Food"));
    game.add_counter(artifact, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, artifact), (6, 5));
    let destroy = put(&mut game, p, "Murder", Zone::Hand);
    cast(&mut game, destroy, Some(Target::Card(artifact)));
    advance(&mut game, |g| g.observe(p).cascade.is_some());
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let receipt = game.observe(p.other()).cascade.unwrap();
    assert_eq!(receipt.discovered, Some(game.object_ref(hit)));
    assert!(receipt.exiled.contains(&game.object_ref(land)));
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(hit).zone, Zone::Hand);
}
