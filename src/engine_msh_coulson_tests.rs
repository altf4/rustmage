//! Coulson's full permanent filter, with paid actions and owned replay.
//! Synthetic Hero producers use ordinary positive-timestamp subtype effects.
//! The noncreature fixture uses gated Kindred Enchantment metadata only; its
//! printed behavior is unused, and the existing Goblin token recipe is unchanged.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;

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
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished payment");
}
fn cast(g: &mut Game, card: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, &[]);
    settle(g);
    id
}
fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, id: CardId) {
    checked(g, activation(id));
    finish(g);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..60 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn set_types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    assert!(timestamp > 0);
    let roles = g.state.roles.get_or_insert_default();
    roles
        .creature_types
        .retain(|row| row.recipient != recipient);
    roles
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types,
            additional: false,
            timestamp,
        });
}
fn kindred_metadata(g: &mut Game, p: Player, hero: bool) -> CardId {
    // The normal named helper requires an admitted card. Reuse its ordinary
    // object initialization, then replace only immutable metadata before any
    // rules action. Boggart Mischief stays unsupported and is never cast or used.
    let id = named(g, p, "Hill Giant", Zone::Battlefield);
    let card = Card::from_name("Boggart Mischief").unwrap();
    assert!(!card.is_supported());
    assert_eq!(card.kind_mask(), 32 | 128);
    g.state.objects[id as usize].card = card;
    if hero {
        set_types(g, id, &["Hero"]);
    }
    assert_eq!(g.current_kinds(id) & 1, 0);
    assert_ne!(g.current_kinds(id) & 128, 0);
    assert_eq!(g.has_subtype(id, "Hero"), hero);
    id
}
fn ready_coulson(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, "Agent Phil Coulson");
    g.state.objects[id as usize].summoning_sick = false;
    id
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}
fn assert_history(g: &Game, p: Player, objects: &[CardId]) {
    for viewer in [p, p.other()] {
        let history = g.observe(viewer).counter_placement_history;
        assert_eq!(history.len(), objects.len());
        for &id in objects {
            assert!(
                history.iter().any(|row| row.object == g.object_ref(id)
                    && row.placed_by == p
                    && row.plus_one)
            );
        }
    }
}

#[test]
fn msh_coulson_paid_tap_cost_kindred_creature_token_and_own_other_filter_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = ready_coulson(&mut g, p);
        let kindred = kindred_metadata(&mut g, p, true);
        let enemy_kindred = kindred_metadata(&mut g, p.other(), true);
        let ordinary_kindred = kindred_metadata(&mut g, p, false);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Brave Brawler", Zone::Battlefield);
        let ordinary = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let grave = named(&mut g, p, "Brave Brawler", Zone::Graveyard);
        let secret = named(&mut g, p, "Brave Brawler", Zone::Hand);
        paid(&mut g, p, "Dragon Fodder");
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token() && o.zone == Zone::Battlefield && o.controller == p)
                    .then_some(id as CardId)
            })
            .unwrap();
        set_types(&mut g, token, &["Hero"]);
        g.state.players[p.index()].mana = [0; 6];
        activate(&mut g, source);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reject(&mut g, activation(source));
        settle(&mut g);
        for id in [kindred, hero, token] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        for id in [
            source,
            enemy_kindred,
            ordinary_kindred,
            enemy,
            ordinary,
            grave,
            secret,
        ] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        }
        assert_history(&g, p, &[kindred, hero, token]);
        assert!(!visible(&g, p.other(), secret));
        assert!(visible(&g, p.other(), kindred));
    }
}

#[test]
fn msh_coulson_paid_summoning_sickness_tap_and_illegal_payment_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Agent Phil Coulson");
        let kindred = kindred_metadata(&mut g, p, true);
        reject(&mut g, activation(source));
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(!g.object_view(source).tapped);
        checked(&mut g, activation(source));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(kindred),
            },
        );
        reject(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert!(g.object_view(source).tapped);
        reject(&mut g, activation(source));
        settle(&mut g);
        assert_eq!(g.counter_count(kindred, CounterKind::PlusOne), 1);
    }
}

#[test]
fn msh_coulson_one_counter_batch_waits_for_every_replacement_before_any_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = ready_coulson(&mut g, p);
        let kindred = kindred_metadata(&mut g, p, true);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        paid(&mut g, p, "Doubling Season");
        paid(&mut g, p, "Doubling Season");
        activate(&mut g, source);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let mut decisions = 0;
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            for id in [kindred, hero] {
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
            }
            assert!(g.observe(p).counter_placement_history.is_empty());
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.kind, "Counters");
            let effect = view.effects[0];
            checked(
                &mut g,
                Action::ChooseReplacement {
                    source: effect.source,
                    ability: effect.ability,
                },
            );
            decisions += 1;
        }
        assert_eq!(decisions, 2);
        settle(&mut g);
        for id in [kindred, hero] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 4);
        }
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_history(&g, p, &[kindred, hero]);
    }
}

#[test]
fn msh_coulson_current_subtypes_and_controllers_are_queried_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = ready_coulson(&mut g, p);
        let lost = kindred_metadata(&mut g, p, true);
        let gained = kindred_metadata(&mut g, p, false);
        let stolen = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let acquired = named(&mut g, p.other(), "Brave Brawler", Zone::Battlefield);
        activate(&mut g, source);
        // Synthetic subtype producers use ordinary positive-timestamp owned rows.
        set_types(&mut g, lost, &["Goblin"]);
        set_types(&mut g, gained, &["Hero"]);
        g.change_controller(g.object_ref(stolen), p.other());
        g.change_controller(g.object_ref(acquired), p);
        settle(&mut g);
        for id in [lost, stolen, source] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        }
        for id in [gained, acquired] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert_history(&g, p, &[gained, acquired]);
    }
}

#[test]
fn msh_coulson_queued_ability_keeps_captured_controller_after_paid_control_transfer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "High Fae Trickster");
        let source = ready_coulson(&mut g, p);
        let own = kindred_metadata(&mut g, p, true);
        let enemy = kindred_metadata(&mut g, p.other(), true);
        activate(&mut g, source);
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            &[Target::Player(p.other()), Target::Card(source)],
        );
        until(&mut g, |g| g.object_view(source).controller == p.other());
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_history(&g, p, &[own]);
    }
}

#[test]
fn msh_coulson_source_departure_or_paid_blink_does_not_rebind_other_exclusion() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            if blink {
                paid(&mut g, p, "High Fae Trickster");
            }
            let source = ready_coulson(&mut g, p);
            let own = kindred_metadata(&mut g, p, true);
            let old = g.object_ref(source);
            activate(&mut g, source);
            if blink {
                let spell = named(&mut g, p, "Daydream", Zone::Hand);
                cast(&mut g, spell, &[Target::Card(source)]);
                until(&mut g, |g| g.object_ref(source) != old);
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            } else {
                g.move_object(source, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
            if blink {
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
                assert_history(&g, p, &[own, source]);
            } else {
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
                assert_history(&g, p, &[own]);
            }
        }
    }
}
