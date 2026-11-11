//! Ordered mixed-zone Craft with live colors and exact material re-entry.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn start_craft(game: &mut Game, source: CardId) {
    act(game, activate(source, 1));
    act(game, Action::BeginOrderedCosts);
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

fn select(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn finish_craft(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishCraftMaterials) {
        act(game, Action::FinishCraftMaterials);
    }
    for _ in 0..6 {
        match game.state.pending {
            Pending::CastCosts { .. } => {
                choose(game, |a| matches!(a, Action::ChooseCastCost { .. }))
            }
            Pending::PayCosts { .. } => pay(game),
            Pending::Priority => {
                settle(game);
                return;
            }
            _ => panic!("unexpected Craft choice {:?}", game.state.pending),
        }
    }
    panic!("Craft payment did not complete");
}

fn mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn sunbird_effigy_counts_each_live_material_color_once_and_makes_each_color() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = spawn(&mut game, p, named("Sunbird Standard"), Zone::Battlefield);
    let green = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let blue = put(&mut game, p, "Opt", Zone::Graveyard);
    let extra_blue = put(&mut game, p, "Sage of Days", Zone::Graveyard);
    let red = put(&mut game, p, "Shock", Zone::Graveyard);
    let colorless = put(&mut game, p, "Forest", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Forest", Zone::Graveyard);
    act(&mut game, mana(source, [1, 0, 0, 0, 0, 0]));
    assert!(game.object_view(source).tapped);

    start_craft(&mut game, source);
    reject(&mut game, Action::FinishCraftMaterials);
    for invalid in [source, foreign] {
        let action = select(&game, invalid);
        reject(&mut game, action);
    }
    for id in [colorless, green, blue, extra_blue, red] {
        let action = select(&game, id);
        act(&mut game, action);
    }
    finish_craft(&mut game);
    assert_eq!(game.object_view(source).card, named("Sunbird Effigy"));
    assert_eq!(stats(&game, source), (3, 3));
    assert!(
        game.keywords(source).contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE)
        )
    );
    let all = [0, 1, 0, 1, 1, 0];
    let mut full = game.clone();
    act(&mut full, mana(source, all));
    game.move_object(blue, Zone::Hand);
    assert_eq!(stats(&game, source), (3, 3));
    game.move_object(extra_blue, Zone::Hand);
    assert_eq!(stats(&game, source), (2, 2));
    reject(&mut game, mana(source, all));
    act(&mut game, mana(source, [0, 0, 0, 1, 1, 0]));
}

#[test]
fn the_grim_captain_uses_four_distinct_materials_and_returns_one_tapped_attacking() {
    let p = Player::P1;
    let opponent = p.other();
    let mut game = scenario(p);
    let source = spawn(
        &mut game,
        p,
        named("Throne of the Grim Captain"),
        Zone::Battlefield,
    );
    let dinosaur = put(
        &mut game,
        p,
        "Itzquinth, Firstborn of Gishath",
        Zone::Battlefield,
    );
    let merfolk = put(&mut game, p, "Jadelight Spelunker", Zone::Graveyard);
    let pirate = put(&mut game, p, "Shipwreck Sentry", Zone::Graveyard);
    let vampire = put(&mut game, p, "Acolyte of Aclazotz", Zone::Battlefield);
    let invalid = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let unlinked = put(&mut game, p, "Shipwreck Sentry", Zone::Exile);
    let victim = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    let top = game.top_library_card(p).unwrap();
    let library = game.state.players[p.index()].library.len();
    act(&mut game, activate(source, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].library.len(), library - 2);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    start_craft(&mut game, source);
    let wrong = select(&game, invalid);
    reject(&mut game, wrong);
    for id in [dinosaur, merfolk, pirate, vampire] {
        let action = select(&game, id);
        act(&mut game, action);
    }
    finish_craft(&mut game);
    assert_eq!(game.object_view(source).card, named("The Grim Captain"));
    assert_eq!(stats(&game, source), (7, 7));
    assert!(
        game.keywords(source).contains(
            Keywords::MENACE
                .union(Keywords::TRAMPLE)
                .union(Keywords::LIFELINK)
                .union(Keywords::HEXPROOF)
        )
    );
    game.state.objects[source as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    let object = game.object_ref(victim);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    let object = game.object_ref(unlinked);
    reject(&mut game, Action::ChooseEffectObject { object });
    let object = game.object_ref(pirate);
    act(&mut game, Action::ChooseEffectObject { object });
    act(
        &mut game,
        Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Player(opponent),
        },
    );
    settle(&mut game);
    assert!(game.object_view(pirate).tapped);
    assert_eq!(game.object_view(pirate).zone, Zone::Battlefield);
    assert!(game.state.attackers.iter().any(|row| row.card == pirate));
    assert_eq!(game.object_view(dinosaur).zone, Zone::Exile);
}
