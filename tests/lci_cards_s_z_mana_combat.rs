//! Actual tap costs, scaled damage, restricted mana and the Calendar state trigger.
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

fn tap_cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn finish_costs(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn until_choice(game: &mut Game) {
    for _ in 0..12 {
        if game.state.pending != Pending::Priority || game.state.stack.is_empty() {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a resolution choice");
}

#[test]
fn sunshot_militia_accepts_itself_and_a_sick_artifact_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = candidate(&mut game, p, "Sunshot Militia", Zone::Battlefield);
    let ally = put(&mut game, p, "Gingerbrute", Zone::Battlefield);
    let opponent = put(&mut game, p.other(), "Gingerbrute", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    for id in [source, ally] {
        game.state.objects[id as usize].summoning_sick = true;
    }
    let mut early = game.clone();
    early.state.step = Step::Upkeep;
    reject(&mut early, activate(source, 0));

    act(&mut game, activate(source, 0));
    for id in [opponent, land] {
        let action = tap_cost(&game, id);
        reject(&mut game, action);
    }
    let first = tap_cost(&game, source);
    act(&mut game, first.clone());
    reject(&mut game, first);
    let second = tap_cost(&game, ally);
    act(&mut game, second);
    finish_costs(&mut game);
    assert!(game.object_view(source).tapped && game.object_view(ally).tapped);
    settle(&mut game);
    assert_eq!(stats(&game, source), (1, 3));
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 1
    );
    reject(&mut game, activate(source, 0));
}

#[test]
fn warden_counts_counter_types_and_scries_only_after_paying_three_taps() {
    let p = Player::P1;
    let mut game = scenario(p);
    let warden = candidate(&mut game, p, "Warden of the Inner Sky", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p, "Candy Trail", Zone::Battlefield);
    let top = put(&mut game, p, "Opt", Zone::Library);
    let keywords = Keywords::FLYING.union(Keywords::VIGILANCE);
    game.add_counter(warden, CounterKind::Charge, 2);
    assert!(!game.keywords(warden).contains(keywords));
    act(&mut game, activate(warden, 0));
    for id in [artifact, warden, creature] {
        let action = tap_cost(&game, id);
        act(&mut game, action);
    }
    finish_costs(&mut game);
    assert!(
        [artifact, warden, creature]
            .iter()
            .all(|id| game.object_view(*id).tapped)
    );
    until_choice(&mut game);
    assert_eq!(game.counter_count(warden, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, warden), (2, 3));
    assert!(game.keywords(warden).contains(keywords));
    let private = game.observe(p).library_choice.expect("private scry");
    assert_eq!(private.remaining.len(), 1);
    assert_eq!(private.remaining[0].reference, game.object_ref(top));
    assert!(game.observe(p.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].library.first(), Some(&top));
}

#[test]
fn triumphant_chomp_samples_only_own_dinosaurs_when_it_resolves() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = candidate(&mut game, p, "Triumphant Chomp", Zone::Hand);
    let target = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Candy Trail", Zone::Battlefield);
    game.add_counter(target, CounterKind::PlusOne, 9);
    game.create_tokens(p.other(), Card(5327), 1, false);
    let invalid = Action::Cast {
        card: spell,
        target: Some(Target::Card(artifact)),
        payment: [0, 0, 0, 1, 0, 0],
    };
    reject(&mut game, invalid);
    cast(&mut game, spell, Some(Target::Card(target)));
    let mut above = game.clone();
    above.create_tokens(p, Card(5327), 1, false);
    let dinosaur = above.state.objects.len() as CardId - 1;
    above.add_counter(dinosaur, CounterKind::PlusOne, 2);
    settle(&mut game);
    settle(&mut above);
    assert_eq!(game.object_view(target).damage, 2);
    assert_eq!(above.object_view(target).damage, 5);
}

#[test]
fn sunken_citadel_exposes_only_its_color_and_funds_a_land_activation() {
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let citadel = candidate(&mut game, p, "Sunken Citadel", Zone::Hand);
    let vein = put(&mut game, p, "Promising Vein", Zone::Battlefield);
    let trail = put(&mut game, p, "Candy Trail", Zone::Battlefield);
    let spell = put(&mut game, p, "Opt", Zone::Hand);
    let land = put(&mut game, p, "Island", Zone::Library);
    act(&mut game, Action::PlayLand { card: citadel });
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    let mana = |ability, production| Action::ActivateMana {
        card: citadel,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    let blue = [0, 1, 0, 0, 0, 0];
    let two_blue = [0, 2, 0, 0, 0, 0];
    assert!(game.object_view(citadel).tapped);
    reject(&mut game, mana(0, blue));
    game.untap_objects(&[citadel], untap_rules::UntapCause::Effect);
    reject(&mut game, mana(1, [2, 0, 0, 0, 0, 0]));
    let mut one = game.clone();
    act(&mut one, mana(0, blue));
    assert_eq!(one.state.players[p.index()].mana, blue);
    assert!(
        one.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );

    act(&mut game, mana(1, two_blue));
    assert_eq!(
        game.observe(p).mana_lots[0].restriction,
        ManaRestrictionView::LandActivation
    );
    reject(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, activate(trail, 0));
    act(&mut game, activate(vein, 1));
    pay(&mut game);
    assert_eq!(game.object_view(vein).zone, Zone::Graveyard);
    until_choice(&mut game);
    act(&mut game, Action::ChooseSearch { card: land });
    if matches!(game.state.pending, Pending::EffectSearch { .. }) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.state.players[p.index()].mana, blue);
}

#[test]
fn the_millennium_calendar_counts_own_untaps_and_doubles_only_time() {
    let p = Player::P0;
    let mut game = scenario(p.other());
    let calendar = candidate(&mut game, p, "The Millennium Calendar", Zone::Battlefield);
    let own = put(&mut game, p, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    for id in [calendar, own, enemy] {
        game.state.objects[id as usize].tapped = true;
    }
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::Upkeep);
    if let Pending::OrderTriggers { .. } = game.state.pending {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.counter_count(calendar, CounterKind::Time), 2);
    assert!(game.object_view(enemy).tapped);
    game.add_counter(calendar, CounterKind::Time, 498);
    game.add_counter(calendar, CounterKind::Charge, 7);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    game.state.players[p.other().index()].life = 2000;
    act(&mut game, activate(calendar, 0));
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.counter_count(calendar, CounterKind::Time), 1000);
    assert_eq!(game.counter_count(calendar, CounterKind::Charge), 7);
    if let Pending::OrderTriggers { .. } = game.state.pending {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.object_view(calendar).zone, Zone::Graveyard);
    assert_eq!(game.state.players[p.other().index()].life, 1000);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE);
}
