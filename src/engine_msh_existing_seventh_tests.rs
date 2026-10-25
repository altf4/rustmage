//! Whole existing-core consumers: actual payments, exact returns, opening privacy and owned replay.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

const CAR: &str = "S.H.I.E.L.D. Flying Car";
const JESSICA: &str = "Jessica Jones, Private Eye";
const QUICK: &str = "Quicksilver, Brash Blur";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected choice {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("did not reach stop {:?}", g.state.pending);
}
fn pay(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn car(g: &mut Game, p: Player, target: Option<CardId>) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, CAR, Zone::Hand);
    cast(g, id, None, &[]);
    if target.is_none() {
        settle(g);
        return id;
    }
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == id && a.item.kind == (StackKind::TriggeredAbility { ability: 0 })
        })
    });
    if let Some(c) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(c),
            },
        );
    }
    pay(g);
    settle(g);
    id
}
fn ready_jessica(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, JESSICA, &[]);
    reject(g, activation(id));
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    assert!(!g.object_view(id).summoning_sick);
    id
}
fn orphan_step(g: Game, action: Action) -> Game {
    let snapshot = g.snapshot();
    let actor = g.actor().unwrap();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let revision = g.revision();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&snapshot).unwrap();
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_seventh_car_paid_flash_optional_targets_and_noncreature_vehicle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p;
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, CAR, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == id));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        cast(&mut g, id, None, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == id)
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own)
        }));
        checked(&mut g, Action::FinishTargets);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.current_kinds(id) & 1, 0);
        assert!(
            g.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::FLASH))
        );
        assert!(g.observe(p).delayed_returns.is_empty());
    }
}
#[test]
fn msh_seventh_car_delayed_owner_return_survives_source_and_control_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let original = g.object_ref(target);
        g.change_controller(original, p);
        let id = car(&mut g, p, Some(target));
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        let link = g.observe(p).delayed_returns[0].clone();
        assert_eq!(link.returning.controller, p.other());
        assert_eq!(link.returning.object, g.object_ref(target));
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Disenchant", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_ref(target), link.returning.object);
        next_turn(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).controller, p.other());
        assert_eq!(g.object_view(target).owner, p.other());
        assert_ne!(g.object_ref(target), original);
        assert!(g.observe(p).delayed_returns.is_empty());
    }
}
#[test]
fn msh_seventh_car_all_invalid_target_does_not_exile_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, CAR, Zone::Hand);
        cast(&mut g, id, None, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == id)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let old = g.object_ref(target);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(target)]);
        until(&mut g, |g| {
            g.object_view(target).zone == Zone::Hand && g.state.pending == Pending::Priority
        });
        let fresh = g.object_ref(target);
        assert_ne!(fresh, old);
        settle(&mut g);
        assert_eq!(g.object_ref(target), fresh);
        assert_eq!(g.object_view(target).zone, Zone::Hand);
        assert!(g.observe(p).delayed_returns.is_empty());
    }
}
#[test]
fn msh_seventh_car_exact_exile_link_owned_after_parent_drop_and_token_ceases() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let target = if token {
                paid(&mut g, p, "Hire a Crew", &[]);
                g.state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                    .unwrap()
                    .0 as CardId
            } else {
                named(&mut g, p, "Bear Cub", Zone::Battlefield)
            };
            car(&mut g, p, Some(target));
            let exact = g.object_ref(target);
            assert_eq!(
                g.object_view(target).zone,
                if token { Zone::Outside } else { Zone::Exile }
            );
            g = orphan_step(g, Action::Pass);
            next_turn(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if token {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
            if !token {
                assert_ne!(g.object_ref(target), exact);
            }
            assert!(g.observe(p).delayed_returns.is_empty());
        }
    }
}
#[test]
fn msh_seventh_car_crew_sick_creature_exact_power_flying_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = car(&mut g, p, None);
        let crewer = paid(&mut g, p, "Llanowar Elves", &[]);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.object_view(crewer).summoning_sick);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, activation(id));
        reject(&mut g, Action::FinishCrew);
        let object = g.object_ref(enemy);
        reject(&mut g, Action::ChooseCrewer { object });
        let object = g.object_ref(crewer);
        g = orphan_step(g, Action::ChooseCrewer { object });
        reject(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(crewer).tapped);
        assert_eq!(g.current_kinds(id) & (1 | 16), 1 | 16);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.keywords(id).contains(Keywords::FLYING));
        next_turn(&mut g);
        assert_eq!(g.current_kinds(id) & 1, 0);
    }
}
#[test]
fn msh_seventh_jessica_paid_stun_cost_is_not_effect_doubled_and_replaces_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready_jessica(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Doubling Season", &[]);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, activation(id));
        pay(&mut g);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.counter_count(id, CounterKind::Stun), 1);
        reject(&mut g, activation(id));
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.counter_count(id, CounterKind::Stun), 0);
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(!g.object_view(id).tapped);
    }
}
#[test]
fn msh_seventh_jessica_dynamic_power_public_exile_real_cast_land_and_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready_jessica(&mut g, p);
        let land = named(&mut g, p, "Mountain", Zone::Library);
        let shock = named(&mut g, p, "Shock", Zone::Library);
        assert!(
            !g.observe(p)
                .objects
                .iter()
                .any(|o| o.id == land || o.id == shock)
        );
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        for viewer in [p, p.other()] {
            for c in [land, shock] {
                assert!(
                    g.observe(viewer)
                        .objects
                        .iter()
                        .any(|o| o.id == c && o.zone == Zone::Exile)
                );
            }
        }
        reject(&mut g, Action::AnnounceExile { card: shock });
        let mut foreign = g.clone();
        foreign.state.active = p.other();
        foreign.state.priority = p.other();
        reject(&mut foreign, Action::AnnounceExile { card: shock });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceExile { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.life(p.other()), 18);
        g = orphan_step(g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        let extra = named(&mut g, p, "Forest", Zone::Exile);
        reject(&mut g, Action::PlayLand { card: extra });
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Shore Up", &[Target::Card(id)]);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.counter_count(id, CounterKind::Stun), 0);
        paid(&mut g, p, "Shore Up", &[Target::Card(id)]);
        assert!(!g.object_view(id).tapped);
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        let remaining = g
            .observe(p)
            .exile_permissions
            .iter()
            .map(|x| x.object.card)
            .collect::<Vec<_>>();
        assert!(!remaining.is_empty());
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(g.observe(p).exile_permissions.is_empty());
        for c in remaining {
            reject(&mut g, Action::PlayLand { card: c });
        }
    }
}
#[test]
fn msh_seventh_jessica_current_power_and_departure_lki_keep_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let id = ready_jessica(&mut g, p);
            checked(&mut g, activation(id));
            pay(&mut g);
            g.state.players[p.index()].mana = [30; 6];
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, None, &[Target::Card(id)]);
            until(&mut g, |g| {
                g.state.stack.len() == 1
                    && g.state.resolving.is_none()
                    && g.state.pending == Pending::Priority
            });
            assert_eq!(g.stats(id).power, 5);
            let before = g.state.players[p.index()].library.len();
            if depart {
                let kill = named(&mut g, p, "Murder", Zone::Hand);
                cast(&mut g, kill, None, &[Target::Card(id)]);
            } else {
                let object = g.object_ref(id);
                g.change_controller(object, p.other());
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - 5);
            assert_eq!(g.observe(p).exile_permissions.len(), 5);
            assert!(g.observe(p).exile_permissions.iter().all(|x| x.player == p));
            if depart {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn msh_seventh_jessica_zero_power_exiles_nothing_and_still_pays_stun() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready_jessica(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Dead Weight", &[Target::Card(id)]);
        assert_eq!(g.stats(id).power, 0);
        let before = g.state.players[p.index()].library.len();
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
        assert_eq!(g.counter_count(id, CounterKind::Stun), 1);
    }
}
fn opening(p: Player) -> Game {
    let quick = Card::from_name(QUICK).unwrap();
    let mut g = Game::new(Config {
        decks: [vec![quick; 20], vec![quick; 20]],
        seed: 20261025,
        starting_player: p,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    checked(&mut g, Action::Keep);
    checked(&mut g, Action::Keep);
    assert!(matches!(g.state.pending, Pending::OpeningHand { .. }));
    g
}
#[test]
fn msh_seventh_quicksilver_real_opening_decline_private_eligibility_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = opening(p);
        let own = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.owner == p && o.zone == Zone::Hand)
            .unwrap()
            .0 as CardId;
        let enemy = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.owner == p.other() && o.zone == Zone::Hand)
            .unwrap()
            .0 as CardId;
        assert_eq!(
            g.legal_actions()
                .iter()
                .filter(|a| matches!(a, Action::BeginOnBattlefield { .. }))
                .count(),
            7
        );
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == own));
        let object = g.object_ref(enemy);
        reject(&mut g, Action::BeginOnBattlefield { object });
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.hand_count(p), 7);
        let object = g.object_ref(own);
        reject(&mut g, Action::BeginOnBattlefield { object });
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.object_view(own).zone, Zone::Hand);
        assert_eq!(g.state.active, p);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::BeginOnBattlefield { .. }))
        );
    }
}
#[test]
fn msh_seventh_quicksilver_opening_duplicates_legend_and_first_turn_no_discount() {
    for p in [Player::P0, Player::P1] {
        let mut g = opening(p);
        let ids = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| {
                if let Action::BeginOnBattlefield { object } = a {
                    Some(object)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        checked(&mut g, Action::BeginOnBattlefield { object: ids[0] });
        reject(&mut g, Action::BeginOnBattlefield { object: ids[0] });
        g = orphan_step(g, Action::BeginOnBattlefield { object: ids[1] });
        // CR103/704: pregame placements finish before the first legend-rule SBA.
        assert_eq!(g.object_view(ids[0].card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(ids[1].card).zone, Zone::Battlefield);
        checked(&mut g, Action::FinishOpeningHand);
        checked(&mut g, Action::FinishOpeningHand);
        let survivor = g.object_ref(ids[1].card);
        assert!(
            g.legal_actions()
                .contains(&Action::KeepLegend { object: survivor })
        );
        reject(&mut g, Action::KeepLegend { object: ids[1] });
        checked(&mut g, Action::KeepLegend { object: survivor });
        assert_eq!(g.object_view(ids[0].card).zone, Zone::Graveyard);
        let id = ids[1].card;
        assert!(!g.object_view(id).entered_this_turn);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(&mut g, activation(id));
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 1);
    }
}
#[test]
fn msh_seventh_quicksilver_paid_entry_discount_once_counters_and_double_strike_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let id = paid(&mut g, p, QUICK, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(id).entered_this_turn && g.object_view(id).summoning_sick);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, activation(id));
        g.state.players[p.index()].mana[5] = 4;
        checked(&mut g, activation(id));
        let request = g.announcement_payment_request(p).unwrap();
        assert_eq!((request.required, request.generic), ([0; 6], 4));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            g.keywords(id)
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::HASTE))
        );
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 1);
        reject(&mut g, activation(id));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p.other()), 16);
        reject(&mut g, activation(id));
        next_turn(&mut g);
        next_turn(&mut g);
        reject(&mut g, activation(id));
    }
}
#[test]
fn msh_seventh_quicksilver_counter_replacement_keyword_removal_and_exact_blink_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let id = paid(&mut g, p, QUICK, &[]);
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 2);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        g.state.players[p.index()].mana = [30; 6];
        let destroy = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, destroy, None, &[Target::Card(aura)]);
        settle(&mut g);
        assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        let old = g.object_ref(id);
        car(&mut g, p, Some(id));
        next_turn(&mut g);
        assert_ne!(g.object_ref(id), old);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 0);
        assert!(g.object_view(id).power_up_uses.is_empty());
        g.state.players[p.other().index()].mana = [30; 6];
        let object = g.object_ref(id);
        g.change_controller(object, p.other());
        assert_eq!(g.actor(), Some(p.other()));
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 1);
    }
}

#[test]
fn msh_seventh_jessica_full_scalar_exiles_above_u16_and_bounds_by_real_library() {
    // Exercise the concrete CardId selector above u16 with only a 256 KiB
    // library slice, then resolve a real paid Jessica ability with that scalar.
    // No giant permanent arena or memory-pressure allocation is needed.
    let library: Vec<CardId> = (0..65_537).collect();
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready_jessica(&mut g, p);
        let power = if p == Player::P0 { 65_536 } else { i128::MAX };
        g.state.objects[id as usize].growth.power = power - 2;
        checked(&mut g, activation(id));
        pay(&mut g);
        let amount = g.item_amount(
            crate::cards::AmountValue::SourcePower,
            g.state.stack.last().unwrap(),
        );
        assert_eq!(amount, power as u128);
        let selected = exile::top_library_cards(&library, amount);
        assert_eq!(
            selected.len(),
            if p == Player::P0 { 65_536 } else { 65_537 }
        );
        assert_eq!(selected.first(), Some(&65_536));
        assert_eq!(selected.last(), Some(&if p == Player::P0 { 1 } else { 0 }));
        let before = g.state.players[p.index()].library.len();
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.exile_permissions.len(), before);
        assert!(g.state.players[p.index()].library.is_empty());
        assert!(
            g.state
                .exile_permissions
                .iter()
                .all(|row| row.player == p && row.object == g.object_ref(row.object.card))
        );
    }
}

#[test]
fn msh_seventh_quicksilver_paid_bounce_queued_counters_and_new_incarnation_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, QUICK, &[]);
        let old = g.object_ref(id);
        checked(&mut g, activation(id));
        pay(&mut g);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_ne!(g.object_ref(id), old);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 0);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        checked(&mut g, activation(id));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(id, CounterKind::DoubleStrike), 1);
        let object = g.object_ref(id);
        g.change_controller(object, p.other());
        g.state.priority = p.other();
        reject(&mut g, activation(id));
    }
}
