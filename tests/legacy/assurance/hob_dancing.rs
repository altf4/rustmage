//! Captured cast mana value and actual landfall token outcomes.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("dancing decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn recipient(g: &mut Game, id: CardId) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn source(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Dancing from Dark to Dawn", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 3];
    cast(g, id, None);
    drive(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn bears(g: &Game, p: Player) -> Vec<CardId> {
    let factory = Card::from_name("@token/hob-bear").unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == factory)
                .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn dancing_paid_enchantment_actual_landfall_exact_token_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let enchantment = source(&mut g, p);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        drive(&mut g);
        let tokens = bears(&g, p);
        assert_eq!(tokens.len(), 1);
        assert!(bears(&g, p.other()).is_empty());
        let bear = tokens[0];
        let card = g.state.objects[bear as usize].card;
        assert!(card.is_token());
        assert!(card.has_subtype("Bear"));
        assert_eq!(card.colors(), 16);
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.object_view(bear).tapped);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
        // A distinct actual entry queues a second landfall. The queued ability
        // remains effective after its source leaves; token creation is no cast.
        let second = put(&mut g, p, "Swamp", Zone::Hand);
        g.move_object(second, Zone::Battlefield);
        g.move_object(enchantment, Zone::Graveyard);
        g.give_priority();
        drive(&mut g);
        assert_eq!(bears(&g, p).len(), 2);
        for id in bears(&g, p) {
            assert_eq!(stats(&g, id), (2, 2));
        }
    }
}
#[test]
fn dancing_creature_cast_mandatory_own_target_and_repeated_mana_value_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        for n in 1..=2 {
            let spell = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            cast(&mut g, spell, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            for invalid in [enemy, land, spell] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(invalid),
                    },
                );
            }
            reject(&mut g, Action::FinishTargets);
            recipient(&mut g, own);
            drive(&mut g);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2 * n);
            assert_eq!(
                stats(&g, own),
                (2 + 2 * i128::from(n), 2 + 2 * i128::from(n))
            );
            assert_eq!(stats(&g, spell), (2, 2));
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, own), (6, 6));
    }
}
#[test]
fn dancing_excludes_noncreature_opponent_casts_and_noncast_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, Some(Target::Player(p.other())));
        drive(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let entered = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.move_object(entered, Zone::Battlefield);
        g.give_priority();
        drive(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        let spell = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, spell, None);
        drive(&mut g);
        let land = put(&mut g, p.other(), "Forest", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        drive(&mut g);
        assert!(bears(&g, p).is_empty());
        assert!(bears(&g, p.other()).is_empty());
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 0);
        assert_eq!(stats(&g, entered), (2, 2));
        assert_eq!(stats(&g, spell), (2, 2));
    }
}
#[test]
fn dancing_reduced_and_free_costs_keep_printed_spell_mana_value() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            let mut g = scenario(p);
            source(&mut g, p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = if free {
                put(&mut g, p, "Omniscience", Zone::Battlefield);
                let id = put(&mut g, p, "Grizzly Bears", Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                act(
                    &mut g,
                    Action::AnnounceFree {
                        card: id,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                );
                pay(&mut g);
                id
            } else {
                put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
                let id = put(&mut g, p, "Large Bear", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 1];
                cast(&mut g, id, None);
                id
            };
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let value = if free { 2 } else { 5 };
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|s| s.kind == StackKind::Spell
                        && s.source.is_some_and(|source| source.id == spell))
                    .unwrap()
                    .mana_value(),
                value
            );
            recipient(&mut g, own);
            drive(&mut g);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), value);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn dancing_chosen_x_countered_spell_frozen_event_departure_stale_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let enchantment = source(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Wildwood Scourge", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: 3 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.kind == StackKind::Spell
                    && s.source.is_some_and(|source| source.id == spell))
                .unwrap()
                .mana_value(),
            4
        );
        recipient(&mut g, own);
        contracts::branch_contract(g.clone());
        // The creature is below its cast trigger. Counter that physical spell,
        // rather than the top ability, before resolving the captured amount.
        if g.actor() == Some(p) {
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.actor(), Some(p.other()));
        let counter = put(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, counter, Some(Target::Spell(spell)));
        until(&mut g, |g| g.object_view(spell).zone == Zone::Graveyard);
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
        assert!(!g.state.stack.is_empty());
        g.move_object(enchantment, Zone::Graveyard);
        g.give_priority();
        let saved = g.snapshot();
        let mut valid = g.clone();
        let mut stale = g.clone();
        drop(g);
        drive(&mut valid);
        assert_eq!(g_counter(&valid, own), 4);
        assert_eq!(stats(&valid, own), (6, 6));
        stale.move_object(own, Zone::Hand);
        stale.move_object(own, Zone::Battlefield);
        stale.give_priority();
        drive(&mut stale);
        assert_eq!(g_counter(&stale, own), 0);
        assert_eq!(stats(&stale, own), (2, 2));
        stale.restore(&saved).unwrap();
        drive(&mut stale);
        assert_eq!(stale.snapshot(), valid.snapshot());
    }
}
fn g_counter(g: &Game, id: CardId) -> u64 {
    g.counter_count(id, CounterKind::PlusOne)
}
#[test]
fn dancing_first_creature_without_legal_recipient_does_not_target_itself_on_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        let spell = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, spell, None);
        drive(&mut g);
        assert_eq!(stats(&g, spell), (2, 2));
        assert_eq!(g_counter(&g, spell), 0);
    }
}
