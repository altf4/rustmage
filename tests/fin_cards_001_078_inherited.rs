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

#[test]
fn bards_bow_and_dragoons_lance_move_their_stats_types_and_keywords_with_equip() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bard = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let knight = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let (bow, bow_hero) = job(&mut g, p, "Bard's Bow");
    let (lance, lance_hero) = job(&mut g, p, "Dragoon's Lance");
    assert_eq!(stats(&g, bow_hero), (3, 3));
    assert!(g.has_subtype(bow_hero, "Bard"));
    assert!(g.keywords(bow_hero).contains(Keywords::REACH));
    assert_eq!(stats(&g, lance_hero), (2, 1));
    assert!(g.has_subtype(lance_hero, "Knight"));
    assert!(g.keywords(lance_hero).contains(Keywords::FLYING));

    activate(&mut g, bow, 0, Some(bard));
    settle(&mut g);
    activate(&mut g, lance, 0, Some(knight));
    settle(&mut g);
    assert_eq!(stats(&g, bow_hero), (1, 1));
    assert_eq!(stats(&g, lance_hero), (1, 1));
    assert!(!g.has_subtype(bow_hero, "Bard"));
    assert!(!g.has_subtype(lance_hero, "Knight"));
    assert_eq!(stats(&g, bard), (4, 4));
    assert_eq!(stats(&g, knight), (4, 3));
    assert!(g.has_subtype(bard, "Bard"));
    assert!(g.has_subtype(knight, "Knight"));
    assert!(g.keywords(knight).contains(Keywords::FLYING));
    g.state.active = p.other();
    assert!(!g.keywords(knight).contains(Keywords::FLYING));
    assert!(g.keywords(bard).contains(Keywords::REACH));
}

#[test]
fn choco_comet_deals_its_paid_x_and_the_racetrack_triggers_on_each_own_land() {
    let p = Player::P0;
    let mut g = scenario(p);
    let comet = put(&mut g, p, "Choco-Comet", Zone::Hand);
    let racetrack = put(&mut g, p, "Chocobo Racetrack", Zone::Battlefield);
    assert_eq!(g.object_view(racetrack).zone, Zone::Battlefield);
    act(&mut g, Action::AnnounceCast { card: comet });
    act(&mut g, Action::ChooseX { value: 3 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 17);
    let first = tokens(&g, p, Card(5269))[0];
    let opposing_land = put(&mut g, p.other(), "Island", Zone::Hand);
    g.move_object(opposing_land, Zone::Battlefield);
    assert_eq!(tokens(&g, p, Card(5269)).len(), 1);

    let land = put(&mut g, p, "Forest", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    for _ in 0..12 {
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    let birds = tokens(&g, p, Card(5269));
    assert_eq!(birds.len(), 2);
    assert_eq!(stats(&g, first), (3, 2));
    assert_eq!(
        stats(&g, *birds.iter().find(|&&id| id != first).unwrap()),
        (2, 2)
    );
}

#[test]
fn cloudbound_moogle_can_counter_an_opposing_creature_or_plainscycle() {
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Cloudbound Moogle", Zone::Hand);
    let second = put(&mut g, p, "Cloudbound Moogle", Zone::Hand);
    let recipient = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut g, p, "Sacred Foundry", Zone::Library);
    cast(&mut g, first, None);
    reach(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(recipient),
        }
    });
    target(&mut g, recipient);
    settle(&mut g);
    assert!(g.keywords(first).contains(Keywords::FLYING));
    assert_eq!(g.object_view(recipient).counters.plus_one, 1);
    activate(&mut g, second, 0, None);
    choose_search(&mut g, land);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
    assert!(
        g.observe(p.other())
            .known_hand
            .contains(&g.object_ref(land))
    );
}

#[test]
fn coeurl_can_tap_an_artifact_creature_and_cannot_target_an_enchantment_creature() {
    let p = Player::P0;
    let mut g = scenario(p);
    let cat = put(&mut g, p, "Coeurl", Zone::Battlefield);
    let artifact = put(&mut g, p.other(), "Magitek Infantry", Zone::Battlefield);
    let enchantment = put(&mut g, p.other(), "Summon: Fat Chocobo", Zone::Battlefield);
    act(&mut g, activation(cat, 0));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enchantment),
        },
    );
    target(&mut g, artifact);
    pay(&mut g);
    assert!(g.object_view(cat).tapped);
    settle(&mut g);
    assert!(g.object_view(artifact).tapped);
    assert!(!g.object_view(enchantment).tapped);
}

#[test]
fn coliseum_behemoth_can_destroy_an_artifact_or_draw_on_entry() {
    let p = Player::P0;
    for mode in 0..2 {
        let mut g = scenario(p);
        let behemoth = put(&mut g, p, "Coliseum Behemoth", Zone::Hand);
        let artifact = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
        cast(&mut g, behemoth, None);
        let choice = Action::ChooseMode { mode };
        reach(&mut g, |a| *a == choice);
        act(&mut g, choice);
        if mode == 0 {
            target(&mut g, artifact);
        }
        settle(&mut g);
        assert_eq!(stats(&g, behemoth), (7, 7));
        assert!(g.keywords(behemoth).contains(Keywords::TRAMPLE));
        assert_eq!(
            g.object_view(artifact).zone,
            if mode == 0 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            u64::from(mode)
        );
    }
}

#[test]
fn combat_tutorial_can_give_the_draws_to_an_opponent_and_the_counter_to_yours() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Combat Tutorial", Zone::Hand);
    let ours = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(theirs),
        },
    );
    target(&mut g, ours);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    assert_eq!(g.object_view(ours).counters.plus_one, 1);
}

#[test]
fn commune_reveals_one_qualified_card_and_places_the_others_in_the_chosen_order() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Commune with Beavers", Zone::Hand);
    let instant = put(&mut g, p, "Shock", Zone::Library);
    let land = put(&mut g, p, "Island", Zone::Library);
    let artifact = put(&mut g, p, "Short Bow", Zone::Library);
    cast(&mut g, spell, None);
    let find = Action::SelectLibrary { card: artifact };
    reach(&mut g, |a| *a == find);
    assert!(g.observe(p.other()).library_selection.is_none());
    reject(&mut g, Action::SelectLibrary { card: instant });
    act(&mut g, find);
    act(&mut g, Action::FinishLibrarySelection);
    act(&mut g, Action::OrderLibraryBottom { card: land });
    act(&mut g, Action::OrderLibraryBottom { card: instant });
    settle(&mut g);
    assert_eq!(&g.state.players[p.index()].library[..2], &[instant, land]);
    assert_eq!(g.object_view(artifact).zone, Zone::Hand);
    assert!(
        g.observe(p.other())
            .known_hand
            .contains(&g.object_ref(artifact))
    );
}

#[test]
fn coral_sword_flashes_in_and_its_first_strike_outlasts_the_attachment() {
    let p = Player::P0;
    let mut g = scenario(p.other());
    let sword = put(&mut g, p, "Coral Sword", Zone::Hand);
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let new_bearer = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    act(&mut g, Action::Pass);
    cast(&mut g, sword, None);
    reach(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(bearer),
        }
    });
    target(&mut g, bearer);
    settle(&mut g);
    assert_eq!(stats(&g, bearer), (3, 2));
    assert!(g.keywords(bearer).contains(Keywords::FIRST_STRIKE));
    g.state.active = p;
    g.state.priority = p;
    activate(&mut g, sword, 0, Some(new_bearer));
    settle(&mut g);
    assert_eq!(stats(&g, bearer), (2, 2));
    assert_eq!(stats(&g, new_bearer), (4, 3));
    assert!(g.keywords(bearer).contains(Keywords::FIRST_STRIKE));
    assert!(!g.keywords(new_bearer).contains(Keywords::FIRST_STRIKE));
}

#[test]
fn dark_knights_greatsword_re_equips_once_per_turn_by_paying_three_life() {
    let p = Player::P0;
    let mut g = scenario(p);
    let (blade, hero) = job(&mut g, p, "Dark Knight's Greatsword");
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&g, hero), (4, 1));
    assert!(g.has_subtype(hero, "Knight"));
    activate(&mut g, blade, 0, Some(bearer));
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 17);
    assert_eq!(stats(&g, bearer), (5, 2));
    assert!(g.has_subtype(bearer, "Knight"));
    assert!(!g.has_subtype(hero, "Knight"));
    reject(&mut g, activation(blade, 0));
}

#[test]
fn dragoons_wyvern_and_dwarven_castle_guard_create_heroes_at_entry_and_death() {
    let p = Player::P0;
    let mut g = scenario(p);
    let wyvern = put(&mut g, p, "Dragoon's Wyvern", Zone::Hand);
    let guard = put(&mut g, p, "Dwarven Castle Guard", Zone::Hand);
    cast(&mut g, wyvern, None);
    settle(&mut g);
    assert!(g.keywords(wyvern).contains(Keywords::FLYING));
    assert_eq!(tokens(&g, p, Card(5328)).len(), 1);
    cast(&mut g, guard, None);
    settle(&mut g);
    assert_eq!(tokens(&g, p, Card(5328)).len(), 1);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Card(guard)));
    settle(&mut g);
    assert_eq!(g.object_view(guard).zone, Zone::Graveyard);
    assert_eq!(tokens(&g, p, Card(5328)).len(), 2);
}

#[test]
fn dreams_of_laguna_surveys_before_drawing_and_exiles_its_flashback() {
    let p = Player::P0;
    let mut g = scenario(p);
    let dreams = put(&mut g, p, "Dreams of Laguna", Zone::Graveyard);
    let drawn = put(&mut g, p, "Island", Zone::Library);
    let rejected = put(&mut g, p, "Shock", Zone::Library);
    act(&mut g, Action::AnnounceFlashback { card: dreams });
    pay(&mut g);
    pass_pair(&mut g);
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: rejected,
            destination: LibraryDestination::Graveyard,
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(dreams).zone, Zone::Exile);
    assert_eq!(g.object_view(rejected).zone, Zone::Graveyard);
    assert_eq!(g.object_view(drawn).zone, Zone::Hand);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn eject_cant_be_countered_and_returns_a_stolen_permanent_to_its_owner() {
    let p = Player::P0;
    let opponent = p.other();
    let mut g = scenario(p);
    let eject = put(&mut g, p, "Eject", Zone::Hand);
    let negate = put(&mut g, opponent, "Negate", Zone::Hand);
    let stolen = put(&mut g, opponent, "Grizzly Bears", Zone::Battlefield);
    g.record_control(g.object_ref(stolen), p, None);
    cast(&mut g, eject, Some(Target::Card(stolen)));
    act(&mut g, Action::Pass);
    cast(&mut g, negate, Some(Target::Spell(eject)));
    settle(&mut g);
    let seen = g.observe(opponent);
    assert!(
        seen.objects
            .iter()
            .any(|o| o.id == stolen && o.zone == Zone::Hand && o.owner == opponent)
    );
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn esper_origins_returns_only_from_flashback_and_maduins_lore_completes_the_saga() {
    let p = Player::P0;
    for flashback in [false, true] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Esper Origins",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let mill = put(&mut g, p, "Shock", Zone::Library);
        let permanent = put(&mut g, p, "Island", Zone::Library);
        if flashback {
            act(&mut g, Action::AnnounceFlashback { card: source });
            pay(&mut g);
        } else {
            cast(&mut g, source, None);
        }
        pass_pair(&mut g);
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: mill,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: permanent,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.object_view(mill).zone, Zone::Graveyard);
        if !flashback {
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.object_view(permanent).zone, Zone::Library);
            continue;
        }
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).card.name(), "Summon: Esper Maduin");
        assert_eq!(g.object_view(source).counters.lore, 1);
        assert_eq!(g.object_view(source).counters.get(CounterKind::Finality), 1);
        assert_eq!(g.object_view(permanent).zone, Zone::Hand);
        let before = g.state.players[p.index()].mana[4];
        for lore in 2..=3 {
            let clash = put(&mut g, p, "Clash of the Eikons", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: clash });
            act(&mut g, Action::ChooseMode { mode: 2 });
            target(&mut g, source);
            pay(&mut g);
            settle(&mut g);
            if lore == 2 {
                assert_eq!(g.state.players[p.index()].mana[4], before + 1);
                assert_eq!(g.object_view(source).counters.lore, 2);
            }
        }
        assert_eq!(stats(&g, bear), (4, 4));
        assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}
