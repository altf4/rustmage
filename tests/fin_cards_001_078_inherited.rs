//! Named native outcomes for the previously admitted cards in FIN rows 1–78.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn reach(game: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..48 {
        if game.legal_actions().iter().any(&wanted) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario choice did not arrive");
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn activate(game: &mut Game, card: CardId, ability: u16, recipient: Option<CardId>) {
    act(game, activation(card, ability));
    if let Some(card) = recipient {
        target(game, card);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn mana(card: CardId, ability: u16, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment,
        counter_payment: Counters::default(),
        production,
    }
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    reach(game, |a| *a == Action::Attack { card: attackers[0] });
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn choose_search(game: &mut Game, card: CardId) {
    let find = Action::ChooseSearch { card };
    reach(game, |a| *a == find);
    act(game, find);
    act(game, Action::FinishSearch);
    settle(game);
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<CardId> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.card == card)
        .map(|o| o.id)
        .collect()
}

fn job(game: &mut Game, player: Player, name: &str) -> (CardId, CardId) {
    let equipment = put(game, player, name, Zone::Hand);
    cast(game, equipment, None);
    settle(game);
    let hero = game.object_view(equipment).attached_to.unwrap().card;
    assert_eq!(game.object_view(hero).card, Card(5328));
    (equipment, hero)
}

#[test]
fn adelbert_counts_only_owned_equipment_and_gains_life_from_damage() {
    let p = Player::P0;
    let mut g = scenario(p);
    let adelbert = put(&mut g, p, "Adelbert Steiner", Zone::Battlefield);
    let first = put(&mut g, p, "Short Bow", Zone::Battlefield);
    put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&g, adelbert), (4, 3));
    g.move_object(first, Zone::Graveyard);
    assert_eq!(stats(&g, adelbert), (3, 2));
    declare(&mut g, &[adelbert]);
    pass_pair(&mut g);
    act(&mut g, Action::FinishBlockers);
    pass_pair(&mut g);
    assert_eq!(g.state.players[p.index()].life, 23);
    assert_eq!(g.state.players[p.other().index()].life, 17);
}

#[test]
fn adventurers_airship_crews_for_two_and_loots_only_on_attack() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Adventurer's Airship", Zone::Battlefield);
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let loot = put(&mut g, p, "Shock", Zone::Hand);
    activate(&mut g, ship, 0, None);
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (3, 2));
    assert!(g.keywords(ship).contains(Keywords::FLYING));
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    declare(&mut g, &[ship]);
    let discard = Action::Discard { card: loot };
    reach(&mut g, |a| *a == discard);
    act(&mut g, discard);
    settle(&mut g);
    assert_eq!(g.object_view(loot).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn aettir_and_priwen_uses_live_life_and_adventurers_inn_adds_two_and_taps_for_colorless() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Aettir and Priwen", Zone::Hand);
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let inn = put(&mut g, p, "Adventurer's Inn", Zone::Hand);
    g.state.objects[bearer as usize].counters.plus_one = 1;
    cast(&mut g, source, None);
    settle(&mut g);
    activate(&mut g, source, 0, Some(bearer));
    settle(&mut g);
    assert_eq!(stats(&g, bearer), (21, 21));
    act(&mut g, Action::PlayLand { card: inn });
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 22);
    assert_eq!(stats(&g, bearer), (23, 23));
    g.state.players[p.index()].mana = [0; 6];
    let colorless = [0, 0, 0, 0, 0, 1];
    act(&mut g, mana(inn, 0, [0; 6], colorless));
    assert_eq!(g.state.players[p.index()].mana, colorless);
}

#[test]
fn ahriman_pays_with_another_creature_or_artifact() {
    let p = Player::P0;
    for name in ["Grizzly Bears", "Short Bow"] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ahriman", Zone::Battlefield);
        let paid = put(&mut g, p, name, Zone::Battlefield);
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
        );
        act(&mut g, activation(source, 0));
        let invalid = g.object_ref(source);
        reject(&mut g, Action::ChooseAdditionalCost { object: invalid });
        let object = g.object_ref(paid);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(paid).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn airship_crash_destroys_each_printed_kind_and_cycles_from_hand() {
    let p = Player::P0;
    for victim in ["Short Bow", "Summon: Fat Chocobo", "Healer's Hawk"] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Airship Crash", Zone::Hand);
        let target = put(&mut g, p.other(), victim, Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Airship Crash", Zone::Hand);
    let plain = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    reject(&mut g, Action::AnnounceCast { card: spell });
    activate(&mut g, spell, 0, None);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.object_view(plain).zone, Zone::Battlefield);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn ashe_privately_looks_five_and_reveals_only_the_chosen_artifact() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ashe = put(&mut g, p, "Ashe, Princess of Dalmasca", Zone::Battlefield);
    let artifact = put(&mut g, p, "Short Bow", Zone::Library);
    let instant = put(&mut g, p, "Shock", Zone::Library);
    declare(&mut g, &[ashe]);
    let select = Action::SelectLibrary { card: artifact };
    reach(&mut g, |a| *a == select);
    assert_eq!(g.observe(p).library_selection.unwrap().remaining.len(), 5);
    assert!(g.observe(p.other()).library_selection.is_none());
    reject(&mut g, Action::SelectLibrary { card: instant });
    act(&mut g, select);
    act(&mut g, Action::FinishLibrarySelection);
    settle(&mut g);
    assert!(
        g.observe(p.other())
            .known_hand
            .contains(&g.object_ref(artifact))
    );
    assert_eq!(g.object_view(instant).zone, Zone::Library);
}

#[test]
fn astrologians_planisphere_grants_the_cast_and_third_draw_triggers_to_its_bearer() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let (planisphere, hero) = job(&mut g, p, "Astrologian's Planisphere");
    assert!(g.has_subtype(hero, "Wizard"));
    g.state.players[p.index()].cards_drawn_this_turn = 2;
    let draw = put(&mut g, p, "Think Twice", Zone::Hand);
    cast(&mut g, draw, None);
    settle(&mut g);
    assert_eq!(g.object_view(hero).counters.plus_one, 2);

    activate(&mut g, planisphere, 0, Some(bearer));
    settle(&mut g);
    assert!(!g.has_subtype(hero, "Wizard"));
    assert!(g.has_subtype(bearer, "Wizard"));
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    settle(&mut g);
    assert_eq!(g.object_view(hero).counters.plus_one, 2);
    assert_eq!(g.object_view(bearer).counters.plus_one, 1);
}

#[test]
fn aurons_inspiration_flashback_can_boost_an_opponents_attackers() {
    let p = Player::P0;
    let mut g = scenario(p.other());
    let spell = put(&mut g, p, "Auron's Inspiration", Zone::Graveyard);
    let attacker = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let idle = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    declare(&mut g, &[attacker]);
    act(&mut g, Action::Pass);
    g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
    act(&mut g, Action::AnnounceFlashback { card: spell });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(stats(&g, attacker), (4, 2));
    assert_eq!(stats(&g, idle), (3, 3));
}

#[test]
fn balamb_t_rexaur_gains_life_on_entry_and_forestcycles_for_a_dual_land() {
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Balamb T-Rexaur", Zone::Hand);
    let second = put(&mut g, p, "Balamb T-Rexaur", Zone::Hand);
    let dual = put(&mut g, p, "Stomping Ground", Zone::Library);
    cast(&mut g, first, None);
    settle(&mut g);
    assert!(g.keywords(first).contains(Keywords::TRAMPLE));
    assert_eq!(g.state.players[p.index()].life, 23);
    activate(&mut g, second, 0, None);
    choose_search(&mut g, dual);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
    assert_eq!(g.object_view(dual).zone, Zone::Hand);
    assert_eq!(g.state.players[p.index()].life, 23);
}

#[test]
fn baron_enters_tapped_and_supplies_only_blue_or_red() {
    let p = Player::P0;
    let mut g = scenario(p);
    let town = put(&mut g, p, "Baron, Airship Kingdom", Zone::Hand);
    act(&mut g, Action::PlayLand { card: town });
    assert!(g.object_view(town).tapped);
    g.state.objects[town as usize].tapped = false;
    g.state.players[p.index()].mana = [0; 6];
    for color in [[0, 1, 0, 0, 0, 0], [0, 0, 0, 1, 0, 0]] {
        let mut branch = g.clone();
        act(&mut branch, mana(town, 0, [0; 6], color));
        assert_eq!(branch.state.players[p.index()].mana, color);
    }
    reject(&mut g, mana(town, 0, [0; 6], [0, 0, 0, 0, 0, 1]));
}

#[test]
fn battle_menus_modes_create_a_knight_boost_toughness_destroy_or_gain_life() {
    let p = Player::P0;
    for mode in 0..4 {
        let mut g = scenario(p);
        let menu = put(&mut g, p, "Battle Menu", Zone::Hand);
        let small = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let large = put(
            &mut g,
            p.other(),
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        act(&mut g, Action::AnnounceCast { card: menu });
        act(&mut g, Action::ChooseMode { mode });
        if mode == 1 {
            target(&mut g, small);
        } else if mode == 2 {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(small),
                },
            );
            target(&mut g, large);
        }
        pay(&mut g);
        settle(&mut g);
        match mode {
            0 => {
                let knights = tokens(&g, p, Card(5236));
                assert_eq!(knights.len(), 1);
                assert_eq!(stats(&g, knights[0]), (2, 2));
            }
            1 => assert_eq!(stats(&g, small), (2, 6)),
            2 => assert_eq!(g.object_view(large).zone, Zone::Graveyard),
            3 => assert_eq!(g.state.players[p.index()].life, 24),
            _ => unreachable!(),
        }
    }
}

#[test]
fn black_mages_rod_deals_damage_from_its_equipped_wizard_and_black_waltz_from_itself() {
    let p = Player::P0;
    let mut g = scenario(p);
    let waltz = put(&mut g, p, "Black Waltz No. 3", Zone::Battlefield);
    let (rod, hero) = job(&mut g, p, "Black Mage's Rod");
    assert!(
        g.keywords(waltz)
            .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
    );
    assert_eq!(stats(&g, hero), (2, 1));
    assert!(g.has_subtype(hero, "Wizard"));
    assert_eq!(g.state.players[p.other().index()].life, 18);
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[bearer as usize].gained_keywords = Keywords::LIFELINK;
    activate(&mut g, rod, 0, Some(bearer));
    settle(&mut g);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    for _ in 0..12 {
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    assert_eq!(g.state.players[p.other().index()].life, 13);
    assert_eq!(g.state.players[p.index()].life, 21);
    assert!(!g.has_subtype(hero, "Wizard"));
    assert!(g.has_subtype(bearer, "Wizard"));
}

#[test]
fn blazing_bomb_uses_mana_spent_then_deals_last_known_power_on_sacrifice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bomb = put(&mut g, p, "Blazing Bomb", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    settle(&mut g);
    assert_eq!(g.object_view(bomb).counters.plus_one, 0);
    let first = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let second = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let costly = put(&mut g, p, "Deadly Embrace", Zone::Hand);
    cast(&mut g, costly, Some(Target::Card(first)));
    settle(&mut g);
    assert_eq!(g.object_view(bomb).counters.plus_one, 1);
    activate(&mut g, bomb, 0, Some(second));
    assert_eq!(g.object_view(bomb).zone, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
}

#[test]
fn blitzball_shot_boosts_and_grants_trample_only_until_cleanup() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Blitzball Shot", Zone::Hand);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    cast(&mut g, spell, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(stats(&g, bear), (5, 5));
    assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
    g.state.step = Step::End;
    pass_pair(&mut g);
    assert_eq!(stats(&g, bear), (2, 2));
    assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
}

#[test]
fn call_the_mountain_chocobo_finds_a_mountain_and_creates_birds_even_on_a_failed_flashback_search()
{
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Call the Mountain Chocobo", Zone::Hand);
    let found = put(&mut g, p, "Sacred Foundry", Zone::Library);
    cast(&mut g, spell, None);
    choose_search(&mut g, found);
    assert!(
        g.observe(p.other())
            .known_hand
            .contains(&g.object_ref(found))
    );
    let bird = tokens(&g, p, Card(5269))[0];
    assert_eq!(stats(&g, bird), (2, 2));
    let land = put(&mut g, p, "Forest", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    settle(&mut g);
    assert_eq!(stats(&g, bird), (3, 2));

    act(&mut g, Action::AnnounceFlashback { card: spell });
    pay(&mut g);
    reach(&mut g, |a| *a == Action::FinishSearch);
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(tokens(&g, p, Card(5269)).len(), 2);
}

#[test]
fn capital_city_can_make_colorless_filter_mana_or_cycle() {
    let p = Player::P0;
    let mut g = scenario(p);
    let city = put(&mut g, p, "Capital City", Zone::Hand);
    let spare = put(&mut g, p, "Capital City", Zone::Hand);
    act(&mut g, Action::PlayLand { card: city });
    assert!(!g.object_view(city).tapped);
    let colorless = [0, 0, 0, 0, 0, 1];
    let blue = [0, 1, 0, 0, 0, 0];
    let mut tap = g.clone();
    tap.state.players[p.index()].mana = [0; 6];
    reject(&mut tap, mana(city, 1, colorless, blue));
    act(&mut tap, mana(city, 0, [0; 6], colorless));
    assert_eq!(tap.state.players[p.index()].mana, colorless);
    g.state.players[p.index()].mana = colorless;
    act(&mut g, mana(city, 1, colorless, blue));
    assert_eq!(g.state.players[p.index()].mana, blue);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut g, spare, 2, None);
    settle(&mut g);
    assert_eq!(g.object_view(spare).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}
