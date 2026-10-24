//! One-or-more recipient events and Invisible Woman's complete printed obligations.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
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
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
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
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn optional(g: &mut Game, p: Player, accept: bool) {
    until(g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept })
    });
    assert_eq!(g.actor(), Some(p));
    reject(g, Action::FinishTargets);
    checked(g, Action::ChooseOptional { accept });
}
fn walls(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.identity() == "@token/msh-wall")
                .then_some(id as CardId)
        })
        .collect()
}
fn activate_coulson(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
}
fn hero_types(g: &mut Game, id: CardId) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &["Hero"],
            additional: false,
            timestamp,
        });
}
fn kindred_hero(g: &mut Game, p: Player) -> CardId {
    // Immutable Kindred Enchantment metadata; the gated card's behavior is never used.
    let id = named(g, p, "Hill Giant", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Boggart Mischief").unwrap();
    hero_types(g, id);
    assert_eq!(g.current_kinds(id) & 1, 0);
    assert!(g.has_subtype(id, "Hero"));
    id
}
fn place(g: &mut Game, p: Player, id: CardId, kind: CounterKind, amount: u64) {
    g.effect_counter_by(id, kind, amount, p, replacements::ResumeMode::Pending);
}
fn queued(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|item| item.object == g.object_ref(id))
        .count()
}

#[test]
fn counter_batch_invisible_paid_coulson_many_heroes_one_optional_wall_and_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let coulson = paid(&mut g, p, "Agent Phil Coulson", &[]);
        g.state.objects[coulson as usize].summoning_sick = false;
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let kindred = kindred_hero(&mut g, p);
        let secret = named(&mut g, p, "Brave Brawler", Zone::Hand);
        activate_coulson(&mut g, coulson);
        optional(&mut g, p, true);
        for id in [sue, hero, kindred] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
        settle(&mut g);
        assert_eq!(walls(&g, p).len(), 1);
        let wall = walls(&g, p)[0];
        let view = g.object_view(wall);
        assert_eq!((view.power, view.toughness), (0, 4));
        assert_eq!(view.colors, 0);
        assert!(view.has_subtype("Wall"));
        assert!(g.keywords(wall).contains(Keywords::DEFENDER));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[wall as usize].summoning_sick = false;
        reject(&mut g, Action::Attack { card: wall });
    }
}
#[test]
fn counter_batch_invisible_decline_and_independent_same_turn_events_do_not_latch() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        for accept in [false, true, true] {
            let spell = named(&mut g, p, "Take Up the Shield", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(hero)]);
            optional(&mut g, p, accept);
            settle(&mut g);
        }
        assert_eq!(g.counter_count(hero, CounterKind::PlusOne), 3);
        assert_eq!(walls(&g, p).len(), 2);
    }
}
#[test]
fn counter_batch_invisible_exact_placer_kind_other_relation_and_noncreature_hero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Brave Brawler", Zone::Battlefield);
        let ordinary = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let kindred = kindred_hero(&mut g, p);
        for (actor, id, kind, amount) in [
            (p, sue, CounterKind::PlusOne, 1),
            (p, enemy, CounterKind::PlusOne, 1),
            (p, ordinary, CounterKind::PlusOne, 1),
            (p.other(), hero, CounterKind::PlusOne, 1),
            (p, hero, CounterKind::Stun, 1),
            (p, hero, CounterKind::PlusOne, 0),
        ] {
            place(&mut g, actor, id, kind, amount);
            assert_eq!(queued(&g, sue), 0);
        }
        place(&mut g, p, kindred, CounterKind::PlusOne, 7);
        assert_eq!(queued(&g, sue), 1);
        optional(&mut g, p, true);
        settle(&mut g);
        assert_eq!(walls(&g, p).len(), 1);
    }
}
#[test]
fn counter_batch_invisible_replacements_hold_whole_batch_and_one_trigger_after_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let coulson = paid(&mut g, p, "Agent Phil Coulson", &[]);
        g.state.objects[coulson as usize].summoning_sick = false;
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let kindred = kindred_hero(&mut g, p);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        activate_coulson(&mut g, coulson);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let mut decisions = 0;
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            for id in [sue, hero, kindred] {
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
            }
            assert_eq!(queued(&g, sue), 0);
            assert!(g.observe(p).counter_placement_history.is_empty());
            let view = g.observe(p).replacement_choice.unwrap();
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
        assert_eq!(decisions, 3);
        optional(&mut g, p, true);
        settle(&mut g);
        for id in [sue, hero, kindred] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 4);
        }
        assert_eq!(walls(&g, p).len(), 4); // Two Seasons also replace the single token event.
    }
}
#[test]
fn counter_batch_invisible_simultaneous_entry_observes_other_entering_heroes_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = named(&mut g, p, "Invisible Woman, Sue Storm", Zone::Graveyard);
        let a = named(&mut g, p, "Brave Brawler", Zone::Graveyard);
        let b = named(&mut g, p, "Ultron Drone", Zone::Graveyard);
        let mut moves = Vec::new();
        for id in [sue, a, b] {
            let mut movement = g.entry_move(id, Zone::Battlefield, false);
            movement.effect_counter = Some((CounterKind::PlusOne, 2));
            moves.push(movement);
        }
        g.stage_entries(moves, None);
        optional(&mut g, p, true);
        settle(&mut g);
        assert_eq!(walls(&g, p).len(), 1);
        for id in [sue, a, b] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        }
    }
}
#[test]
fn counter_batch_invisible_entry_counter_replacements_do_not_dispatch_preview() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        let mut moves = Vec::new();
        for _ in 0..2 {
            let id = named(&mut g, p, "Brave Brawler", Zone::Graveyard);
            let mut movement = g.entry_move(id, Zone::Battlefield, false);
            movement.effect_counter = Some((CounterKind::PlusOne, 1));
            moves.push(movement);
        }
        g.stage_entries(moves, None);
        g.advance_entries();
        let mut decisions = 0;
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            assert_eq!(queued(&g, sue), 0);
            let effect = g.observe(p).replacement_choice.unwrap().effects[0];
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
        optional(&mut g, p, false);
        settle(&mut g);
        assert!(walls(&g, p).is_empty());
    }
}
#[test]
fn counter_batch_invisible_suppression_current_hero_types_and_source_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(sue)]);
        place(&mut g, p, hero, CounterKind::PlusOne, 1);
        assert_eq!(queued(&g, sue), 0);
        g.move_object(aura, Zone::Graveyard);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(hero)]);
        place(&mut g, p, hero, CounterKind::PlusOne, 1);
        assert_eq!(queued(&g, sue), 0);
        g.move_object(aura, Zone::Graveyard);
        let old = g.object_ref(sue);
        g.move_object(sue, Zone::Exile);
        place(&mut g, p, hero, CounterKind::PlusOne, 1);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(sue, Zone::Battlefield);
        assert_ne!(g.object_ref(sue), old);
        place(&mut g, p, hero, CounterKind::PlusOne, 1);
        assert_eq!(queued(&g, sue), 1);
        optional(&mut g, p, true);
        settle(&mut g);
        assert_eq!(walls(&g, p).len(), 1);
    }
}
#[test]
fn counter_batch_invisible_queued_packet_keeps_controller_after_paid_theft_or_departure() {
    for p in [Player::P0, Player::P1] {
        for theft in [false, true] {
            let mut g = setup(p);
            if theft {
                paid(&mut g, p.other(), "High Fae Trickster", &[]);
            }
            let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
            let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
            place(&mut g, p, hero, CounterKind::PlusOne, 1);
            g.prepare_triggers();
            until(&mut g, |g| {
                g.state.pending == Pending::Priority && !g.state.stack.is_empty()
            });
            if theft {
                if g.actor() != Some(p.other()) {
                    checked(&mut g, Action::Pass);
                }
                let thief = named(&mut g, p.other(), "Involuntary Employment", Zone::Hand);
                cast(&mut g, thief, None, &[Target::Card(sue)]);
                until(&mut g, |g| {
                    g.state.objects[sue as usize].controller == p.other()
                });
            } else {
                g.move_object(sue, Zone::Exile);
            }
            optional(&mut g, p, true);
            settle(&mut g);
            assert_eq!(walls(&g, p).len(), 1);
            assert_eq!(walls(&g, p.other()).len(), 0);
        }
    }
}
#[test]
fn counter_batch_invisible_paid_lifelink_combat_uses_actual_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let before = g.state.players[p.index()].life;
        let enemy = g.state.players[p.other().index()].life;
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[sue as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: sue });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.players[p.index()].life != before);
        assert_eq!(g.state.players[p.index()].life, before + 2);
        assert_eq!(g.state.players[p.other().index()].life, enemy - 2);
        assert!(walls(&g, p).is_empty());
    }
}

#[test]
fn counter_batch_invisible_mixed_kinds_and_placing_players_queue_once_per_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sue = paid(&mut g, p, "Invisible Woman, Sue Storm", &[]);
        let enemy_sue = named(
            &mut g,
            p.other(),
            "Invisible Woman, Sue Storm",
            Zone::Battlefield,
        );
        let own = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Brave Brawler", Zone::Battlefield);
        let other = kindred_hero(&mut g, p);
        let placements = [(own, p), (enemy, p.other()), (other, p)]
            .into_iter()
            .map(|(id, placed_by)| {
                let o = &g.state.objects[id as usize];
                replacements::CounterPlacement {
                    object: g.object_ref(id),
                    placed_by,
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: g.granted_subtypes(id),
                    kind: g.current_kinds(id),
                    amount: Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Stun, 3),
                    effect: true,
                    entry_index: None,
                    private: false,
                }
            })
            .collect();
        g.stage_counter_placements(placements, replacements::ResumeMode::Pending);
        assert_eq!(queued(&g, sue), 1);
        assert_eq!(queued(&g, enemy_sue), 1);
        // Resolving the independently captured APNAP packets creates one Wall for each seat.
        settle(&mut g);
        assert_eq!(walls(&g, p).len(), 1);
        assert_eq!(walls(&g, p.other()).len(), 1);
        for id in [own, enemy, other] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(id, CounterKind::Stun), 3);
        }
    }
}
