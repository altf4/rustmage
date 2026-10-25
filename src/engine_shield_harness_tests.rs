//! Individual shield/harness rules, with gated definitions during development only.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const CAP: &str = "Captain America, Super-Soldier";
const MIND: &str = "The Mind Stone";
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!([CAP, MIND].contains(&name));
    let id = named(g, p, "Hill Giant", zone);
    g.state.objects[id as usize].card = card;
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..220 {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::AssignDamage { .. }
                        | Action::ChooseKicker { pay: false }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseKicker { pay: false }
                )
            })
            .unwrap();
        checked(g, a);
    }
}
fn targets(g: &mut Game, id: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == id)
    });
}
fn activation(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, p: Player, id: CardId, ability: u16, mana: Mana) {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(g, activation(id, ability));
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn harness(g: &mut Game, p: Player, id: CardId) {
    activate(g, p, id, 1, [1, 0, 0, 0, 0, 5]);
    settle(g);
    assert_eq!(g.object_view(id).harnessed, Some(true));
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let bytes = g.state_bytes();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.active == p && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn next_main(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    main(g, p);
}
fn choose(g: &mut Game, id: CardId, target: CardId) {
    targets(g, id);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
}
fn suplex(g: &mut Game, p: Player, id: CardId) {
    priority(g, p);
    let spell = named(g, p, "Suplex", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    cast(g, spell, Some(1), &[Target::Card(id)]);
}
#[test]
fn msh_shield_harness_captain_paid_body_current_hero_player_hexproof_and_shield_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(g.counter_count(id, CounterKind::Shield), 1);
        assert!(g.keywords(id).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
        assert!(g.player_has_hexproof(p));
        let hero = paid(&mut g, p, "Hawkeye, Young Avenger", &[]);
        let foreign = named(
            &mut g,
            p.other(),
            "Hawkeye, Young Avenger",
            Zone::Battlefield,
        );
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(g.keywords(hero).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(foreign).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
        priority(&mut g, p.other());
        let spell = named(&mut g, p.other(), "Shock", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hero),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(id).damage, 0);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
        assert!(!g.player_has_hexproof(p));
        assert!(!g.keywords(hero).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn msh_shield_harness_captain_destroy_replaces_once_then_actual_mass_destroy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
        paid(&mut g, p.other(), "Murder", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p, "Day of Judgment", &[]);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_shield_harness_captain_unpreventable_consumes_one_of_multiple_shields() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        g.set_counters(
            id,
            Counters::of(CounterKind::Shield, 3).with(CounterKind::PlusOne, 2),
        );
        paid(&mut g, p, "Impractical Joke", &[Target::Card(id)]);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 2);
        assert_eq!(g.object_view(id).damage, 3);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(g.player_has_hexproof(p));
    }
}
#[test]
fn msh_shield_harness_captain_shield_survives_ability_loss_but_conditional_grants_do_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let hero = paid(&mut g, p, "Hawkeye, Young Avenger", &[]);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        assert!(!g.player_has_hexproof(p));
        assert!(!g.keywords(hero).contains(Keywords::HEXPROOF));
        paid(&mut g, p.other(), "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 0);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        assert!(g.keywords(id).contains(Keywords::FIRST_STRIKE));
        assert!(!g.player_has_hexproof(p));
    }
}
#[test]
fn msh_shield_harness_captain_indestructible_preserves_shield_and_zero_toughness_still_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        paid(&mut g, p, "Take Up the Shield", &[Target::Card(id)]);
        paid(&mut g, p, "Murder", &[Target::Card(id)]);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 1);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        g.set_counters(
            id,
            Counters::of(CounterKind::Shield, 1).with(CounterKind::MinusOne, 2),
        );
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_shield_harness_captain_entry_replacements_and_noncopiable_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        let id = paid(&mut g, p, CAP, &[]);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 4);
        let mimic = paid(&mut g, p, "Taskmaster, Mercenary Mimic", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        choose(&mut g, mimic, id);
        settle(&mut g);
        assert_eq!(
            g.stats(mimic),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(g.counter_count(mimic, CounterKind::Shield), 0);
        assert!(g.keywords(mimic).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn msh_shield_harness_captain_actual_first_strike_does_not_need_shield() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        next_main(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 1);
        assert_eq!(g.object_view(id).damage, 0);
    }
}
#[test]
fn msh_shield_harness_captain_simultaneous_sources_choose_which_shield_prevents_owned() {
    for p in [Player::P0, Player::P1] {
        for prevent_deathtouch in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, CAP, &[]);
            paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
            g.set_counters(
                id,
                Counters::of(CounterKind::Shield, 1).with(CounterKind::PlusOne, 4),
            );
            let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let fynn = named(&mut g, p.other(), "Fynn, the Fangbearer", Zone::Battlefield);
            next_main(&mut g, p);
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            checked(
                &mut g,
                Action::Block {
                    blocker: giant,
                    attacker: id,
                },
            );
            checked(
                &mut g,
                Action::Block {
                    blocker: fynn,
                    attacker: id,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            assert_eq!(g.actor(), Some(p));
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.kind, "DamageEvent");
            assert_eq!(view.damage_event_order.len(), 2);
            assert!(view.effects.is_empty());
            assert!(view.damage_preventions.is_empty());
            let source = if prevent_deathtouch { fynn } else { giant };
            let index = view
                .damage_event_order
                .iter()
                .find(|row| row.event.source.reference.card == source)
                .unwrap()
                .index;
            reject(&mut g, Action::ChooseDamageEvent { index: u32::MAX });
            let action = Action::ChooseDamageEvent { index };
            assert_eq!(
                serde_json::from_value::<Action>(serde_json::to_value(&action).unwrap()).unwrap(),
                action
            );
            g = orphan(g, action);
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            assert_eq!(
                g.object_view(id).zone,
                if prevent_deathtouch {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if prevent_deathtouch {
                assert_eq!(g.object_view(id).damage, 3);
                assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
            }
        }
    }
}
#[test]
fn msh_shield_harness_mind_paid_mana_indestructible_and_inactive_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(g.object_view(id).harnessed, None);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceMana {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert!(g.object_view(id).tapped);
        reject(&mut g, activation(id, 1));
        until(&mut g, |g| {
            g.state.step == Step::End && g.state.pending == Pending::Priority
        });
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_shield_harness_mind_harness_costs_opponent_turn_and_repeatable_designation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        for mana in [[0, 1, 0, 0, 0, 5], [1, 0, 0, 0, 0, 4]] {
            g.state.players[p.index()].mana = mana;
            reject(&mut g, activation(id, 1));
        }
        next_turn(&mut g);
        main(&mut g, p.other());
        harness(&mut g, p, id);
        assert!(g.object_view(id).tapped);
        next_turn(&mut g);
        main(&mut g, p);
        harness(&mut g, p, id);
        assert_eq!(
            g.state.roles.as_ref().unwrap().harnessed,
            vec![g.object_ref(id)]
        );
    }
}
#[test]
fn msh_shield_harness_mind_actual_granted_end_trigger_targets_and_owned_blink() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.set_counters(bear, Counters::of(CounterKind::PlusOne, 3));
        harness(&mut g, p, id);
        targets(&mut g, id);
        for bad in [id, foreign, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        let old = g.object_ref(bear);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_ne!(g.object_ref(bear), old);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).counters, Counters::default());
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}
#[test]
fn msh_shield_harness_mind_blinks_stolen_recipient_to_actual_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Evil's Thrall", &[Target::Card(bear)]);
        assert_eq!(g.object_view(bear).controller, p);
        harness(&mut g, p, id);
        choose(&mut g, id, bear);
        settle(&mut g);
        assert_eq!(g.object_view(bear).controller, p.other());
    }
}
#[test]
fn msh_shield_harness_mind_source_departure_before_harness_resolution_does_not_mark_new_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "High Fae Trickster", &[]);
        let id = paid(&mut g, p, MIND, &[]);
        activate(&mut g, p, id, 1, [1, 0, 0, 0, 0, 5]);
        suplex(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_eq!(g.object_view(id).harnessed, None);
        assert!(
            !g.state
                .roles
                .as_ref()
                .unwrap()
                .harnessed
                .contains(&g.object_ref(id))
        );
    }
}
#[test]
fn msh_shield_harness_mind_queued_blink_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "High Fae Trickster", &[]);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        harness(&mut g, p, id);
        choose(&mut g, id, bear);
        let old = g.object_ref(bear);
        suplex(&mut g, p, id);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_ne!(g.object_ref(bear), old);
    }
}
#[test]
fn msh_shield_harness_captain_current_control_moves_both_conditional_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let own = paid(&mut g, p, "Hawkeye, Young Avenger", &[]);
        let foreign = named(
            &mut g,
            p.other(),
            "Hawkeye, Young Avenger",
            Zone::Battlefield,
        );
        paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(id)],
        );
        assert!(!g.player_has_hexproof(p));
        assert!(g.player_has_hexproof(p.other()));
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        assert!(g.keywords(foreign).contains(Keywords::HEXPROOF));
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 0);
        assert!(!g.player_has_hexproof(p.other()));
        assert!(!g.keywords(foreign).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn msh_shield_harness_mind_decline_optional_blink_keeps_exact_recipient() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        harness(&mut g, p, id);
        targets(&mut g, id);
        let old = g.object_ref(bear);
        checked(&mut g, Action::FinishTargets);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_ref(bear), old);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_shield_harness_mind_designation_is_not_copied_but_persists_through_copy_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        harness(&mut g, p, id);
        let mimic = paid(&mut g, p, "Absorbing Man", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        choose(&mut g, mimic, id);
        settle(&mut g);
        assert_eq!(g.object_view(mimic).harnessed, None);
        assert!(g.keywords(mimic).contains(Keywords::INDESTRUCTIBLE));
        harness(&mut g, p, mimic);
        next_turn(&mut g);
        next_turn(&mut g);
        choose(&mut g, mimic, id);
        settle(&mut g);
        assert_eq!(g.object_view(mimic).harnessed, Some(true));
        assert!(
            g.object_view(mimic)
                .abilities
                .as_ref()
                .unwrap()
                .grants
                .iter()
                .any(|grant| grant.grantor == g.object_ref(mimic)
                    && grant.granting_card == Card::from_name(MIND).unwrap())
        );
    }
}
#[test]
fn msh_shield_harness_captain_real_lands_pay_colored_and_generic_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = seed(&mut g, p, CAP, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: id });
        let a = named(&mut g, p, "Plains", Zone::Battlefield);
        let b = named(&mut g, p, "Plains", Zone::Battlefield);
        let c = named(&mut g, p, "Forest", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: id });
        for card in [a, b, c] {
            checked(&mut g, Action::TapMana { card });
        }
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_shield_harness_captain_paid_sacrifice_and_exile_do_not_use_destroy_shield() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let spell = named(&mut g, p, "Corrupted Conviction", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        for _ in 0..8 {
            let a = Action::ChooseAdditionalCost {
                object: g.object_ref(id),
            };
            if g.legal_actions().contains(&a) {
                g = orphan(g, a);
                break;
            }
            finish(&mut g);
        }
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        next_turn(&mut g);
        main(&mut g, p.other());
        let light = packet(&mut g, p.other(), "Banishing Light", &[]);
        choose(&mut g, light, id);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
}
#[test]
fn msh_shield_harness_captain_destroy_and_create_still_creates_for_unchanged_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        let before = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .count();
        paid(&mut g, p.other(), "Stroke of Midnight", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(id, CounterKind::Shield), 0);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                .count(),
            before + 1
        );
    }
}
#[test]
fn msh_shield_harness_captain_live_hero_types_and_changeling_receive_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, CAP, &[]);
        let hero = paid(&mut g, p, "Hawkeye, Young Avenger", &[]);
        let changeling = paid(&mut g, p, "Three Tree Mascot", &[]);
        assert!(g.keywords(changeling).contains(Keywords::HEXPROOF));
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(hero)]);
        assert!(!g.keywords(hero).contains(Keywords::HEXPROOF));
        assert!(g.player_has_hexproof(p));
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        assert!(g.keywords(hero).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn msh_shield_harness_captain_unpreventable_deathtouch_still_causes_state_based_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, CAP, &[]);
        paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        g.set_counters(
            id,
            Counters::of(CounterKind::Shield, 1).with(CounterKind::PlusOne, 4),
        );
        let fynn = named(&mut g, p.other(), "Fynn, the Fangbearer", Zone::Battlefield);
        next_main(&mut g, p);
        paid(&mut g, p, "Impractical Joke", &[]);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: fynn,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_shield_harness_shield_prevention_can_be_saved_by_fog_bank_effect_order() {
    for p in [Player::P0, Player::P1] {
        for consume in [false, true] {
            let mut g = setup(p.other());
            let attacker = paid(&mut g, p.other(), "Hill Giant", &[]);
            let bank = named(&mut g, p, "Fog Bank", Zone::Battlefield);
            g.set_counters(bank, Counters::of(CounterKind::Shield, 1));
            next_main(&mut g, p.other());
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: attacker });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            checked(
                &mut g,
                Action::Block {
                    blocker: bank,
                    attacker,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            let source = g.object_ref(bank);
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.damage_preventions.len(), 2);
            g = orphan(
                g,
                Action::ChooseDamagePrevention {
                    source,
                    prevention: if consume {
                        DamagePreventionKind::ShieldCounter
                    } else {
                        DamagePreventionKind::CombatToRecipient
                    },
                },
            );
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            assert_eq!(g.object_view(bank).damage, 0);
            assert_eq!(
                g.counter_count(bank, CounterKind::Shield),
                u64::from(!consume)
            );
        }
    }
}
#[test]
fn msh_shield_harness_mind_reserved_tap_cannot_fund_itself_real_land_payment_is_nonmutating() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        g.state.players[p.index()].mana = [0; 6];
        let lands: Vec<_> = (0..5)
            .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        reject(&mut g, activation(id, 1));
        let white = named(&mut g, p, "Plains", Zone::Battlefield);
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        assert!(g.legal_actions().contains(&activation(id, 1)));
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
            views
        );
        checked(&mut g, activation(id, 1));
        for card in lands.into_iter().chain([white]) {
            checked(&mut g, Action::TapMana { card });
        }
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(id).harnessed, Some(true));
        assert!(g.object_view(id).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_shield_harness_mind_grant_tracks_current_controller_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        harness(&mut g, p, id);
        paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(id)],
        );
        until(&mut g, |g| {
            g.state.step == Step::End && g.state.pending == Pending::Priority
        });
        assert!(g.state.stack.is_empty());
        next_turn(&mut g);
        main(&mut g, p.other());
        let old = g.object_ref(bear);
        choose(&mut g, id, bear);
        settle(&mut g);
        assert_ne!(g.object_ref(bear), old);
        assert_eq!(g.object_view(id).harnessed, Some(true));
    }
}
#[test]
fn msh_shield_harness_mind_real_copy_harness_suppression_restoration_and_blink_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let mimic = paid(&mut g, p, "Absorbing Man", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        choose(&mut g, mimic, id);
        settle(&mut g);
        harness(&mut g, p, mimic);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(mimic)]);
        assert_eq!(g.object_view(mimic).harnessed, Some(true));
        assert!(
            g.object_view(mimic)
                .abilities
                .as_ref()
                .unwrap()
                .grants
                .is_empty()
        );
        reject(&mut g, activation(mimic, 1));
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        assert!(
            !g.object_view(mimic)
                .abilities
                .as_ref()
                .unwrap()
                .grants
                .is_empty()
        );
        let old = g.object_ref(mimic);
        paid(&mut g, p, "Daydream", &[Target::Card(mimic)]);
        assert_ne!(g.object_ref(mimic), old);
        assert_eq!(g.object_view(mimic).harnessed, None);
        assert_eq!(
            g.object_view(mimic).card,
            Card::from_name("Absorbing Man").unwrap()
        );
        assert_eq!(g.counter_count(mimic, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_shield_harness_mind_blink_owns_entry_choice_pause() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let automaton = paid(&mut g, p, "Adaptive Automaton", &[]);
        harness(&mut g, p, id);
        choose(&mut g, id, automaton);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        let Action::ChooseEntryValue { value } = action else {
            unreachable!()
        };
        g = orphan(g, Action::ChooseEntryValue { value });
        settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Battlefield);
        assert_eq!(g.object_view(automaton).entry_choice, Some(value));
    }
}
#[test]
fn msh_shield_harness_mind_blinks_noncreature_equipment_and_detaches_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, activation(pole, 0));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[pole as usize].attached_to,
            Some(g.object_ref(bear))
        );
        harness(&mut g, p, id);
        let old = g.object_ref(pole);
        choose(&mut g, id, pole);
        settle(&mut g);
        assert_ne!(g.object_ref(pole), old);
        assert_eq!(g.state.objects[pole as usize].attached_to, None);
        assert_eq!(g.object_view(pole).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_shield_harness_mind_blinked_token_ceases_and_does_not_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let skrull = paid(&mut g, p, "Super-Skrull", &[]);
        activate(&mut g, p, skrull, 0, [1, 0, 0, 0, 0, 2]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .map(|(id, _)| id as CardId)
            .unwrap();
        harness(&mut g, p, id);
        choose(&mut g, id, token);
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}
#[test]
fn msh_shield_harness_mind_blinked_aura_has_owned_nontarget_entry_attachment_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, MIND, &[]);
        let old = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let new = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(old)]);
        harness(&mut g, p, id);
        choose(&mut g, id, aura);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AuraEntry { .. })
        });
        let object = g.object_ref(new);
        g = orphan(g, Action::ChooseAuraAttachment { object });
        settle(&mut g);
        assert_eq!(
            g.stats(old),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(
            g.stats(new),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(
            g.state.objects[aura as usize].attached_to,
            Some(g.object_ref(new))
        );
    }
}
