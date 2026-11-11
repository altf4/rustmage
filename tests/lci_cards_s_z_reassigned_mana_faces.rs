//! Tarrian's one-use permissions and The Myriad Pools' exact spent mana.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tap(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn resolve_tap_ability(game: &mut Game, source: CardId, ability: u16) {
    act(game, announce(source, ability));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

#[test]
fn tarrians_journal_sacrifices_another_permanent_then_discards_the_entire_hand_to_transform() {
    let p = Player::P0;
    let mut game = scenario(p);
    let card = named(&mut game, p, "Tarrian's Journal", Zone::Hand);
    let material = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let other = put(&mut game, p, "Shock", Zone::Hand);
    let drawn = put(&mut game, p, "Healer's Hawk", Zone::Library);
    cast(&mut game, card, None);
    settle(&mut game);
    game.state.step = Step::Upkeep;
    reject(&mut game, announce(card, 0));
    game.state.step = Step::Main1;
    act(&mut game, announce(card, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let cost = |game: &Game, card| Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    };
    let invalid = cost(&game, card);
    reject(&mut game, invalid);
    let invalid = cost(&game, enemy);
    reject(&mut game, invalid);
    let payment = cost(&game, material);
    act(&mut game, payment);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(material).zone, Zone::Graveyard);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
    let reference = game.object_ref(card);
    game.untap_objects(&[card], untap_rules::UntapCause::Effect);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    game.state.active = p.other();
    game.state.step = Step::Upkeep;
    let mut empty = game.clone();
    empty.move_object(drawn, Zone::Graveyard);
    empty.move_object(other, Zone::Graveyard);
    for branch in [&mut game, &mut empty] {
        resolve_tap_ability(branch, card, 1);
        assert_eq!(branch.object_view(card).name, "The Tomb of Aclazotz");
        assert_eq!(branch.object_ref(card), reference);
        assert!(branch.object_view(card).tapped);
        assert_eq!(branch.hand_count(p), 0);
        assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
    }
    assert_eq!(game.object_view(drawn).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
}

#[test]
fn the_tomb_of_aclazotz_grants_one_ordinary_paid_cast_with_vampire_and_finality() {
    let p = Player::P1;
    let mut game = scenario(p);
    let card = named(&mut game, p, "The Tomb of Aclazotz", Zone::Battlefield);
    let spell = put(&mut game, p, "Healer's Hawk", Zone::Graveyard);
    let another = put(&mut game, p, "Healer's Hawk", Zone::Graveyard);
    let foreign = put(&mut game, p.other(), "Healer's Hawk", Zone::Graveyard);
    let wrong = put(&mut game, p, "Shock", Zone::Graveyard);
    game.state.players[p.index()].mana = [0; 6];
    let mut mana = game.clone();
    act(&mut mana, tap(card, BLACK));
    assert_eq!(mana.state.players[p.index()].mana, BLACK);

    resolve_tap_ability(&mut game, card, 1);
    assert!(game.object_view(card).tapped);
    let permissions = game.observe(p).paid_graveyard_cast_permissions;
    assert_eq!(permissions.len(), 1);
    let permission = permissions[0].id;
    game.move_object(card, Zone::Graveyard);
    let cast_route = |game: &Game, card| Action::AnnounceGraveyardPaid {
        object: game.object_ref(card),
        permission,
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
    };
    let action = cast_route(&game, spell);
    reject(&mut game, action.clone());
    game.state.players[p.index()].mana = WHITE;
    for card in [foreign, wrong] {
        let invalid = cast_route(&game, card);
        reject(&mut game, invalid);
    }
    game.state.step = Step::Upkeep;
    reject(&mut game, action.clone());
    game.state.step = Step::Main1;
    act(&mut game, action);
    pay(&mut game);
    assert!(game.has_subtype(spell, "Vampire"));
    assert!(game.observe(p).paid_graveyard_cast_permissions.is_empty());
    settle(&mut game);
    assert!(game.has_subtype(spell, "Bird"));
    assert!(game.has_subtype(spell, "Vampire"));
    assert_eq!(game.counter_count(spell, CounterKind::Finality), 1);
    game.state.players[p.index()].mana = WHITE;
    let consumed = cast_route(&game, another);
    reject(&mut game, consumed);

    let damage = put(&mut game, p, "Shock", Zone::Hand);
    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut game, damage, Some(Target::Card(spell)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert!(!game.has_subtype(spell, "Vampire"));
}

#[test]
fn everflowing_well_mills_before_drawing_and_rechecks_descend_eight_on_upkeep() {
    let p = Player::P0;
    let mut game = scenario(p);
    let card = named(&mut game, p, "The Everflowing Well", Zone::Hand);
    for _ in 0..5 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    put(&mut game, p, "Opt", Zone::Graveyard);
    put(&mut game, p.other(), "Forest", Zone::Graveyard);
    let draw = [
        put(&mut game, p, "Opt", Zone::Library),
        put(&mut game, p, "Shock", Zone::Library),
    ];
    let mill = [
        put(&mut game, p, "Forest", Zone::Library),
        put(&mut game, p, "Island", Zone::Library),
    ];
    cast(&mut game, card, None);
    settle(&mut game);
    for id in draw {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
    for id in mill {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    let exact = game.object_ref(card);
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(card).name, "The Everflowing Well");

    let eighth = put(&mut game, p, "Forest", Zone::Library);
    game.move_object(eighth, Zone::Graveyard);
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    let mut depleted = game.clone();
    depleted.move_object(eighth, Zone::Exile);
    settle(&mut depleted);
    assert_eq!(depleted.object_view(card).name, "The Everflowing Well");
    settle(&mut game);
    assert_eq!(game.object_view(card).name, "The Myriad Pools");
    assert_eq!(game.object_ref(card), exact);
    assert_eq!(game.object_view(card).kinds, 2 | 16);
}

#[test]
fn myriad_pools_copies_the_exact_permanent_spell_its_mana_paid_for_even_after_countering() {
    let p = Player::P0;
    let opponent = p.other();
    let mut game = scenario(p);
    let pools = named(&mut game, p, "The Myriad Pools", Zone::Battlefield);
    let recipient = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let spell = put(&mut game, p, "Spyglass Siren", Zone::Hand);
    let instant = put(&mut game, p, "Opt", Zone::Hand);
    let counter = put(&mut game, opponent, "Essence Scatter", Zone::Hand);
    let exact = game.object_ref(recipient);
    game.state.players[p.index()].mana = [0; 6];

    let mut ordinary = game.clone();
    ordinary.state.players[p.index()].mana = BLUE;
    cast(&mut ordinary, spell, None);
    assert_eq!(ordinary.state.stack.len(), 1);
    assert_eq!(ordinary.state.pending, Pending::Priority);
    let mut nonpermanent = game.clone();
    act(&mut nonpermanent, tap(pools, BLUE));
    cast(&mut nonpermanent, instant, None);
    assert_eq!(nonpermanent.state.stack.len(), 1);
    assert_eq!(nonpermanent.state.pending, Pending::Priority);

    act(&mut game, tap(pools, BLUE));
    cast(&mut game, spell, None);
    reject(&mut game, target(pools));
    reject(&mut game, target(enemy));
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(recipient).name, "Bear Cub");

    act(&mut game, target(recipient));
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(spell)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(recipient).name, "Spyglass Siren");
    assert_eq!(game.object_ref(recipient), exact);
    assert!(game.keywords(recipient).contains(Keywords::FLYING));
    assert!(
        game.state
            .objects
            .iter()
            .all(|object| object.card != Card(5284))
    );
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.object_view(recipient).name, "Bear Cub");
    assert_eq!(stats(&game, recipient), (2, 2));
}
