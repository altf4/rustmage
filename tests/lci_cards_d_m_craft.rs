//! LCI Craft faces, granted attacks, and equipment behavior through actual costs.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

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

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn equip(game: &mut Game, source: CardId, bearer: CardId, amount: u32) {
    let p = game.actor().unwrap();
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, amount];
    act(game, announce(source, 0));
    act(game, target(bearer));
    pay(game);
    settle(game);
    assert_eq!(
        game.object_view(source).attached_to,
        Some(game.object_ref(bearer))
    );
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

fn craft(
    game: &mut Game,
    source: CardId,
    ability: u16,
    payment: Mana,
    material: CardId,
    invalid: CardId,
) {
    let p = game.actor().unwrap();
    let before = game.object_ref(source);
    game.state.players[p.index()].mana = payment;
    act(game, announce(source, ability));
    act(game, Action::BeginOrderedCosts);
    let index = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
    let invalid = Action::ChooseAdditionalCost {
        object: game.object_ref(invalid),
    };
    reject(game, invalid);
    let object = game.object_ref(material);
    act(game, Action::ChooseAdditionalCost { object });
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            break;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::ChooseCastCost { .. } | Action::PayCosts { .. }
            )
        });
    }
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_ne!(game.object_ref(source), before);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn dire_flail_crafts_into_a_stronger_equipment_with_an_exact_sacrifice_trigger() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Dire Flail", Zone::Hand);
    let bearer = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let material = put(&mut game, p, "Short Bow", Zone::Graveyard);
    let sacrifice = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    equip(&mut game, source, bearer, 1);
    assert_eq!(stats(&game, bearer), (4, 2));
    let original = game.object_ref(source);
    craft(&mut game, source, 1, [0, 0, 0, 2, 0, 3], material, source);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Dire Blunderbuss");
    assert_ne!(game.object_ref(source), original);
    assert_eq!(game.object_view(source).attached_to, None);
    equip(&mut game, source, bearer, 1);
    assert_eq!(stats(&game, bearer), (5, 2));
    attack(&mut game, bearer);
    pass_pair(&mut game);
    let excluded = Action::Sacrifice {
        object: game.object_ref(source),
    };
    reject(&mut game, excluded);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(sacrifice).zone, Zone::Battlefield);

    let payment = Action::Sacrifice {
        object: game.object_ref(sacrifice),
    };
    act(&mut game, payment);
    act(&mut game, target(victim));
    game.move_object(source, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(game.object_view(victim).damage, 2);
}

#[test]
fn glowcap_lantern_reveals_only_to_the_bearers_controller_and_explores_on_attack() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Glowcap Lantern", Zone::Hand);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let top = put(&mut game, p, "Island", Zone::Library);
    let their_top = put(&mut game, p.other(), "Opt", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.observe(p).known_library_top[p.index()].is_none());
    equip(&mut game, source, creature, 2);
    assert_eq!(
        game.observe(p).known_library_top[p.index()].unwrap().object,
        game.object_ref(top)
    );
    assert!(game.observe(p.other()).known_library_top[p.index()].is_none());
    let mut stolen = game.clone();
    stolen.record_control(stolen.object_ref(creature), p.other(), None);
    assert!(stolen.observe(p).known_library_top[p.index()].is_none());
    assert_eq!(
        stolen.observe(p.other()).known_library_top[p.other().index()]
            .unwrap()
            .object,
        stolen.object_ref(their_top)
    );

    attack(&mut game, creature);
    game.move_object(source, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(
        game.observe(p).last_explore.unwrap().explorer.object,
        game.object_ref(creature)
    );
}

#[test]
fn hunters_blowgun_switches_between_deathtouch_and_reach_on_its_controllers_turn() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Hunter's Blowgun", Zone::Hand);
    let bearer = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    equip(&mut game, source, bearer, 2);
    assert_eq!(stats(&game, bearer), (3, 3));
    assert!(game.keywords(bearer).contains(Keywords::DEATHTOUCH));
    assert!(!game.keywords(bearer).contains(Keywords::REACH));
    game.record_control(game.object_ref(source), p.other(), None);
    assert!(game.keywords(bearer).contains(Keywords::REACH));
    assert!(!game.keywords(bearer).contains(Keywords::DEATHTOUCH));
    game.move_object(source, Zone::Hand);
    assert_eq!(stats(&game, bearer), (2, 2));
}

#[test]
fn idol_of_the_deep_king_flashes_in_and_its_crafted_face_attaches_and_equips() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Idol of the Deep King", Zone::Hand);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let material = put(&mut game, p, "Short Bow", Zone::Graveyard);
    game.state.active = p.other();
    game.state.step = Step::Upkeep;
    cast(&mut game, source, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    settle(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 2
    );

    game.state.active = p;
    game.state.priority = p;
    game.state.step = Step::Main1;
    craft(&mut game, source, 0, [0, 0, 0, 1, 0, 2], material, first);
    pass_pair(&mut game);
    assert_eq!(game.object_view(source).name, "Sovereign's Macuahuitl");
    reject(&mut game, target(enemy));
    act(&mut game, target(first));
    settle(&mut game);
    assert_eq!(
        game.object_view(source).attached_to,
        Some(game.object_ref(first))
    );
    assert_eq!(stats(&game, first), (4, 2));
    equip(&mut game, source, second, 2);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(stats(&game, second), (4, 2));
}

#[test]
fn inverted_iceberg_mills_before_drawing_and_iceberg_titan_toggles_its_attack_target() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Inverted Iceberg", Zone::Hand);
    let draw = put(&mut game, p, "Island", Zone::Library);
    let mill = put(&mut game, p, "Shock", Zone::Library);
    let material = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let untap = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    game.state.objects[untap as usize].tapped = true;
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
    craft(&mut game, source, 0, [0, 2, 0, 0, 0, 4], material, source);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Iceberg Titan");
    assert_eq!(stats(&game, source), (6, 6));
    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, source);
    act(&mut game, target(untap));
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert!(declined.object_view(untap).tapped);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(!game.object_view(untap).tapped);
}

#[test]
fn jade_seedstones_divides_three_counters_and_jadeheart_uses_exiled_mana_value() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Jade Seedstones", Zone::Hand);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let material = put(&mut game, p, "Hill Giant", Zone::Graveyard);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(opponent));
    act(&mut game, target(first));
    act(&mut game, target(second));
    act(&mut game, Action::FinishTargets);
    reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
    for amount in [2, 1] {
        act(&mut game, Action::AllocateTargetAmount { amount });
    }
    let mut stale = game.clone();
    stale.move_object(first, Zone::Hand);
    settle(&mut stale);
    assert_eq!(stale.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(stale.counter_count(second, CounterKind::PlusOne), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);

    craft(&mut game, source, 0, [0, 0, 0, 0, 2, 5], material, opponent);
    pass_pair(&mut game);
    let mut removed = game.clone();
    removed.move_object(material, Zone::Graveyard);
    removed.move_object(material, Zone::Exile);
    settle(&mut removed);
    assert_eq!(removed.state.players[p.index()].life, STARTING_LIFE);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Jadeheart Attendant");
    assert_eq!(stats(&game, source), (7, 7));
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 4);
}

#[test]
fn kaslems_stonetree_puts_a_private_land_onto_the_battlefield_and_crafts_with_a_cave() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Kaslem's Stonetree", Zone::Hand);
    let mut revealed = Vec::new();
    for name in [
        "Opt",
        "Shock",
        "Bear Cub",
        "Forest",
        "Grizzly Bears",
        "Shock",
    ] {
        revealed.push(put(&mut game, p, name, Zone::Library));
    }
    let material = put(&mut game, p, "Captivating Cave", Zone::Graveyard);
    let invalid = put(&mut game, p, "Forest", Zone::Graveyard);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(
        game.observe(p).library_selection.unwrap().remaining.len(),
        6
    );
    assert!(game.observe(p.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: revealed[0] });
    act(&mut game, Action::SelectLibrary { card: revealed[3] });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert_eq!(game.object_view(revealed[3]).zone, Zone::Battlefield);
    assert!(game.object_view(revealed[3]).tapped);
    assert!(
        revealed
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 3)
            .all(|(_, id)| game.state.players[p.index()].library[..5].contains(id))
    );
    craft(&mut game, source, 0, [0, 0, 0, 0, 1, 5], material, invalid);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Kaslem's Strider");
    assert_eq!(stats(&game, source), (5, 5));
}

#[test]
fn lodestone_needle_stuns_an_already_tapped_creature_and_guidestone_compass_explores() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Lodestone Needle", Zone::Hand);
    let opponent = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[opponent as usize].tapped = true;
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let material = put(&mut game, p, "Short Bow", Zone::Graveyard);
    let top = put(&mut game, p, "Island", Zone::Library);
    game.state.step = Step::Upkeep;
    cast(&mut game, source, None);
    pass_pair(&mut game);
    let mut skipped = game.clone();
    act(&mut skipped, Action::FinishTargets);
    settle(&mut skipped);
    assert_eq!(skipped.counter_count(opponent, CounterKind::Stun), 0);
    act(&mut game, target(opponent));
    settle(&mut game);
    assert!(game.object_view(opponent).tapped);
    assert_eq!(game.counter_count(opponent, CounterKind::Stun), 2);

    game.state.step = Step::Main1;
    craft(&mut game, source, 0, [0, 1, 0, 0, 0, 2], material, source);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Guidestone Compass");
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    game.state.step = Step::Upkeep;
    reject(&mut game, announce(source, 0));
    game.state.step = Step::Main1;
    act(&mut game, announce(source, 0));
    reject(&mut game, target(opponent));
    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(
        game.observe(p).last_explore.unwrap().explorer.object,
        game.object_ref(creature)
    );
}
