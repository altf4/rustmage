//! Paid full-card tribal/counter obligations; checked actions compare owned branches and restore.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn activate(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, action);
    }
}
#[test]
fn fdn_tribal_death_baron_paid_union_not_twice_and_blank() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let zombie = named(&mut g, p, "Diregraf Ghoul", Zone::Battlefield);
        let skeleton = named(&mut g, p, "Reassembling Skeleton", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Diregraf Ghoul", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let changeling = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let base = g.stats(changeling).power;
        let lord = paid(&mut g, p, "Death Baron", [1, 0, 2, 0, 0, 0]);
        assert_eq!(g.stats(changeling).power, base + 1);
        assert!(g.keywords(changeling).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.stats(lord).power, 2);
        assert!(!g.keywords(lord).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.stats(zombie).power, 3);
        assert_eq!(g.stats(skeleton).power, 2);
        assert_eq!(g.stats(foe).power, 2);
        assert_eq!(g.stats(bear).power, 2);
        assert!(g.keywords(zombie).contains(Keywords::DEATHTOUCH));
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, stop, None, &[Target::Card(lord)]);
        settle(&mut g);
        assert_eq!(g.stats(zombie).power, 2);
        assert!(!g.keywords(zombie).contains(Keywords::DEATHTOUCH));
        g.move_object(stop, Zone::Graveyard);
        assert_eq!(g.stats(zombie).power, 3);
        g.move_object(lord, Zone::Graveyard);
        assert_eq!(g.stats(skeleton).power, 1);
    }
}
#[test]
fn fdn_tribal_dwynen_counts_current_attacking_elves_on_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let resting = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let lord = paid(&mut g, p, "Dwynen, Gilt-Leaf Daen", [2, 0, 0, 0, 2, 0]);
        assert!(g.keywords(lord).contains(Keywords::REACH));
        assert_eq!(g.stats(elf).power, 2);
        assert_eq!(g.stats(resting).power, 2);
        assert_eq!(g.stats(lord).power, 3);
        g.state.objects[lord as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for id in [lord, elf, bear] {
            checked(&mut g, Action::Attack { card: id });
        }
        checked(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        g.move_object(elf, Zone::Graveyard);
        let life = g.state.players[p.index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
    }
}
#[test]
fn fdn_tribal_hare_counts_other_named_creatures_at_resolution_and_unlimited_deck() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = paid(&mut g, p, "Hare Apparent", [2, 0, 0, 0, 0, 0]);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o
                    .card
                    .token_info()
                    .is_some_and(|t| t.name == "Rabbit Token")
                    && o.zone == Zone::Battlefield)
                .count(),
            0
        );
        named(&mut g, p.other(), "Hare Apparent", Zone::Battlefield);
        let second = named(&mut g, p, "Hare Apparent", Zone::Hand);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        cast(&mut g, second, None, &[]);
        super::tests::pass_pair(&mut g);
        g.prepare_triggers();
        g.move_object(second, Zone::Exile);
        settle(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.card
                    .token_info()
                    .is_some_and(|t| t.name == "Rabbit Token")
                    && o.zone == Zone::Battlefield
            })
            .collect();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].controller, p);
        assert!(!tokens[0].tapped);
        assert_eq!(g.stats(first).power, 2);
        assert_eq!(
            Card::from_name("Hare Apparent").unwrap().info().deck_limit,
            None
        );
    }
    let hare = Card::from_name("Hare Apparent").unwrap();
    assert!(
        Game::new(Config {
            decks: [vec![hare; 60], vec![hare; 60]],
            deck_rules: DeckRules::Constructed,
            seed: 1,
            starting_player: Player::P0
        })
        .is_ok()
    );
}
#[test]
fn fdn_tribal_heroes_bane_paid_entry_and_current_power_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bane = paid(&mut g, p, "Heroes' Bane", [3, 0, 0, 0, 2, 0]);
        assert_eq!(g.state.objects[bane as usize].counters.plus_one, 4);
        assert_eq!(g.stats(bane).power, 4);
        g.state.players[p.index()].mana = [3, 0, 0, 0, 0, 0];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: bane,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [2, 0, 0, 0, 2, 0];
        activate(&mut g, bane);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.objects[bane as usize].growth.power = 3;
        settle(&mut g);
        assert_eq!(g.state.objects[bane as usize].counters.plus_one, 11);
        // An old activation cannot put counters on a new incarnation.
        g.state.players[p.index()].mana = [2, 0, 0, 0, 2, 0];
        activate(&mut g, bane);
        g.move_object(bane, Zone::Exile);
        g.move_object(bane, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[bane as usize].counters.plus_one, 4);
    }
}
#[test]
fn fdn_tribal_paladin_own_turn_counters_and_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let plain = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[own as usize].counters.plus_one = 1;
        g.state.objects[foe as usize].counters.plus_one = 1;
        let pal = paid(&mut g, p, "Inspiring Paladin", [3, 0, 0, 0, 0, 0]);
        for id in [pal, own] {
            assert!(g.keywords(id).contains(Keywords::FIRST_STRIKE));
        }
        for id in [plain, foe] {
            assert!(!g.keywords(id).contains(Keywords::FIRST_STRIKE));
        }
        next_turn(&mut g);
        for id in [pal, own] {
            assert!(!g.keywords(id).contains(Keywords::FIRST_STRIKE));
        }
        next_turn(&mut g);
        assert!(g.keywords(own).contains(Keywords::FIRST_STRIKE));
        while g.state.step != Step::Main1 || g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, stop, None, &[Target::Card(pal)]);
        settle(&mut g);
        assert!(!g.keywords(pal).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(own).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn fdn_tribal_leonin_intervening_if_checks_both_times() {
    for p in [Player::P0, Player::P1] {
        for lose in [false, true] {
            let mut g = setup(p);
            let v = paid(&mut g, p, "Leonin Vanguard", [1, 0, 0, 0, 0, 0]);
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let third = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            if lose {
                g.move_object(third, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.stats(v).power, if lose { 1 } else { 2 });
            assert_eq!(g.state.players[p.index()].life, life + i128::from(!lose));
            next_turn(&mut g);
            assert_eq!(g.stats(v).power, 1);
        }
    }
}
#[test]
fn fdn_tribal_halana_legal_other_target_current_and_lki_power_haste() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let pair = paid(&mut g, p, "Halana and Alena, Partners", [2, 0, 0, 1, 1, 0]);
            assert!(g.keywords(pair).contains(Keywords::REACH));
            assert!(g.keywords(pair).contains(Keywords::FIRST_STRIKE));
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(pair),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            while g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            g.state.objects[pair as usize].growth.power = 3;
            if leave {
                g.move_object(pair, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[target as usize].counters.plus_one, 5);
            assert!(g.keywords(target).contains(Keywords::HASTE));
            next_turn(&mut g);
            assert!(!g.keywords(target).contains(Keywords::HASTE));
            assert_eq!(g.stats(target).power, 7);
        }
    }
}

#[test]
fn fdn_tribal_heroes_bane_funds_announced_activation_from_empty_pool() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bane = paid(&mut g, p, "Heroes' Bane", [3, 0, 0, 0, 2, 0]);
        let lands = [
            named(&mut g, p, "Forest", Zone::Battlefield),
            named(&mut g, p, "Forest", Zone::Battlefield),
            named(&mut g, p, "Plains", Zone::Battlefield),
            named(&mut g, p, "Plains", Zone::Battlefield),
        ];
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: bane,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        for card in lands {
            checked(&mut g, Action::TapMana { card });
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.objects[bane as usize].counters.plus_one, 8);
        assert!(lands.iter().all(|&id| g.object_view(id).tapped));
    }
}
