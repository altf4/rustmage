//! Source-duration Saga chapters, prevention ordering and LKI.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseDamagePrevention { .. }
            )
        });
    }
    panic!("Saga decision budget: {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.replacements.is_none()
    });
}
fn step(g: &mut Game) {
    g.begin_step(Step::Main1);
    g.give_priority();
    g.prepare_triggers();
}
fn targets(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
}
fn bind(g: &mut Game, id: Option<CardId>) {
    if let Some(id) = id {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn saga(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Old Fat Spider Can't See Me", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, id, None);
    id
}
fn damage(g: &mut Game, source: DamageSource, player: Player, combat: bool, preventable: bool) {
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(player),
        amount: 3,
        combat,
        preventable,
    }]);
    if g.state.replacements.is_some() {
        g.advance_replacements(true);
    }
    while matches!(g.state.pending, Pending::Replacement { .. }) {
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseReplacement { .. } | Action::ChooseDamagePrevention { .. }
            )
        });
    }
}
#[test]
fn hob_old_spider_paid_chapters_targets_draws_and_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for optional in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let id = saga(&mut g, p);
            targets(&mut g);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            contracts::branch_contract(g.clone());
            bind(&mut g, Some(own));
            finish(&mut g);
            assert!(g.keywords(own).contains(Keywords::HEXPROOF));
            let mut opponent = g.clone();
            opponent.state.priority = p.other();
            let shock = put(&mut opponent, p.other(), "Shock", Zone::Hand);
            act(&mut opponent, Action::AnnounceCast { card: shock });
            reject(
                &mut opponent,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            assert_eq!(g.counter_count(id, CounterKind::Lore), 1);
            step(&mut g);
            targets(&mut g);
            bind(&mut g, optional.then_some(foe));
            finish(&mut g);
            let source = g.damage_source(foe);
            damage(&mut g, source, p, false, true);
            assert_eq!(
                g.state.players[p.index()].life,
                if optional { 20 } else { 17 }
            );
            for chapter in 3..=4 {
                let before = g.state.players[p.index()].cards_drawn_this_turn;
                let drawn = *g.state.players[p.index()].library.last().unwrap();
                step(&mut g);
                finish(&mut g);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
                assert!(g.observe(p).objects.iter().any(|o| o.id == drawn));
                assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == drawn));
                assert_eq!(
                    g.object_view(id).zone,
                    if chapter == 4 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
            assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        }
    }
}
#[test]
fn hob_old_spider_lki_prevention_tracks_old_source_and_ends_with_saga() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        let frozen = g.damage_source(foe);
        damage(&mut g, frozen, p, true, true);
        damage(&mut g, frozen, p, false, true);
        assert_eq!(g.state.players[p.index()].life, 20);
        let unrelated = g.damage_source(other);
        damage(&mut g, unrelated, p, false, true);
        assert_eq!(g.state.players[p.index()].life, 17);
        g.move_object(foe, Zone::Exile);
        damage(&mut g, frozen, p, false, true);
        assert_eq!(
            g.state.players[p.index()].life,
            17,
            "old source remains prevented"
        );
        g.move_object(foe, Zone::Battlefield);
        let fresh = g.damage_source(foe);
        damage(&mut g, fresh, p, false, true);
        assert_eq!(
            g.state.players[p.index()].life,
            14,
            "new incarnation is distinct"
        );
        g.move_object(id, Zone::Exile);
        damage(&mut g, frozen, p, false, true);
        assert_eq!(g.state.players[p.index()].life, 11);
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn hob_old_spider_unpreventable_damage_and_prevention_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        put(&mut g, p.other(), "Gratuitous Violence", Zone::Battlefield);
        let source = g.damage_source(foe);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p),
            amount: 3,
            combat: false,
            preventable: true,
        }]);
        g.advance_replacements(true);
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        contracts::branch_contract(g.clone());
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::ChooseReplacement { .. } | Action::ChooseDamagePrevention { .. }
                )
            });
        }
        assert_eq!(g.state.players[p.index()].life, 20);
        g.state.damage_prevention_prohibited = true;
        damage(&mut g, source, p, false, false);
        assert_eq!(g.state.players[p.index()].life, 14);
        assert!(g.on_battlefield(id));
    }
}
#[test]
fn hob_old_spider_queued_ability_from_departed_creature_is_prevented() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Unruly Catapult", Zone::Battlefield);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        g.state.priority = p.other();
        act(
            &mut g,
            Action::AnnounceActivate {
                card: foe,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.move_object(foe, Zone::Exile);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn hob_old_spider_source_and_target_must_survive_chapter_resolution() {
    for p in [Player::P0, Player::P1] {
        for remove_source in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = saga(&mut g, p);
            targets(&mut g);
            bind(&mut g, Some(own));
            if remove_source {
                g.move_object(id, Zone::Exile);
            } else {
                g.move_object(own, Zone::Exile);
                g.move_object(own, Zone::Battlefield);
            }
            finish(&mut g);
            assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
            assert!(g.source_linked_effect_views().is_none());
        }
    }
}
#[test]
fn hob_old_spider_hexproof_respects_ability_removal_timestamps() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, Some(Target::Card(own)));
        finish(&mut g);
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn hob_old_spider_actual_combat_damage_is_prevented() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: foe });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        act(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn hob_old_spider_two_sources_offer_independent_prevention_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(own));
        finish(&mut g);
        step(&mut g);
        targets(&mut g);
        bind(&mut g, Some(foe));
        finish(&mut g);
        let source = g.damage_source(foe);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p),
            amount: 3,
            combat: false,
            preventable: true,
        }]);
        g.advance_replacements(true);
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        let choices = g
            .legal_actions()
            .iter()
            .filter(|a| {
                matches!(
                    a,
                    Action::ChooseDamagePrevention {
                        prevention: DamagePreventionKind::SourceLinkedDamage,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(choices, 2);
        contracts::branch_contract(g.clone());
        choose(&mut g, |a| {
            matches!(a, Action::ChooseDamagePrevention { .. })
        });
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
