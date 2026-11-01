//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
use crate::engine::ability_world::CharacteristicWorld;

fn announce_face(g: &mut Game, card: CardId, adventure: bool, mana: Mana) {
    let p = g.actor().unwrap();
    g.state.players[p.index()].mana = mana;
    act(
        g,
        if adventure {
            Action::AnnounceAdventure { card }
        } else if g.object_view(card).zone == Zone::Exile {
            Action::AnnounceExile { card }
        } else {
            Action::AnnounceCast { card }
        },
    );
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(
        g.state.players[g.state.stack.last().unwrap().controller.index()].mana,
        [0; 6]
    );
}
fn paid(g: &mut Game, card: CardId, adventure: bool, target: Option<Target>, mana: Mana) {
    announce_face(g, card, adventure, mana);
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn normal(
    g: &mut Game,
    p: Player,
    card: CardId,
    mana: Mana,
    body: Option<(i128, i128)>,
    keywords: Keywords,
) {
    assert_eq!(g.object_view(card).zone, Zone::Exile);
    assert!(g.can_play_adventure(p, card));
    assert!(!g.can_play_adventure(p.other(), card));
    g.state.active = p;
    g.give_priority();
    g.state.players[p.index()].mana = [8; 6];
    reject(g, Action::AnnounceAdventure { card });
    paid(g, card, false, None, mana);
    settle(g);
    assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    if let Some(body) = body {
        assert_eq!(stats(g, card), body);
    }
    assert!(g.keywords(card).contains(keywords));
    assert!(!g.can_play_adventure(p, card));
}

#[test]
fn hob_inherited_two_beorn_paid_faces_extra_land_is_controller_only_and_expires() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let lands: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Hand))
            .collect();
        act(&mut g, Action::PlayLand { card: lands[0] });
        reject(&mut g, Action::PlayLand { card: lands[1] });
        let beorn = put(&mut g, p, "Beorn, Reluctant Host", Zone::Hand);
        paid(&mut g, beorn, true, None, [0, 0, 0, 0, 1, 1]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 1);
        assert_eq!(
            g.state.players[p.other().index()].extra_land_plays_remaining,
            0
        );
        act(&mut g, Action::PlayLand { card: lands[1] });
        reject(&mut g, Action::PlayLand { card: lands[2] });
        normal(
            &mut g,
            p,
            beorn,
            [0, 0, 0, 0, 1, 4],
            Some((5, 5)),
            Keywords::TRAMPLE,
        );
        g.finish_cleanup();
        assert_eq!(g.state.players[p.index()].extra_land_plays_granted, 0);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 0);
        // Fresh own main-phase fixture after expiry: exactly the ordinary land.
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.land_played = false;
        g.state.continuous_land_plays_used = 0;
        g.give_priority();
        let fourth = put(&mut g, p, "Island", Zone::Hand);
        act(&mut g, Action::PlayLand { card: lands[2] });
        reject(&mut g, Action::PlayLand { card: fourth });
    }
}

#[test]
fn hob_inherited_two_bofur_both_target_kinds_control_protection_lifelink_and_expiry() {
    {
        let p = Player::P0;
        for artifact in [false, true] {
            let mut g = scenario(p);
            let protected = put(
                &mut g,
                p,
                if artifact {
                    "Goldvein Pick"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let foreign = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let bofur = put(&mut g, p, "Bofur, Reliable Guardian", Zone::Hand);
            g.state.active = p.other(); // Instant inset remains castable on opponent turn.
            announce_face(&mut g, bofur, true, [1, 0, 0, 0, 0, 1]);
            for id in [foreign, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(protected),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert!(
                g.keywords(protected)
                    .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
            // Opponent cannot target the protected object, but has a legal fallback.
            g.state.priority = p.other();
            let hostile = put(
                &mut g,
                p.other(),
                if artifact { "Disenchant" } else { "Shock" },
                Zone::Hand,
            );
            announce_face(
                &mut g,
                hostile,
                false,
                if artifact {
                    [1, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 0, 1, 0, 0]
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(protected),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: if artifact {
                        Target::Card(foreign)
                    } else {
                        Target::Player(p)
                    },
                },
            );
            finish(&mut g);
            settle(&mut g);
            // Its controller can target it; the actual destroy spell still cannot destroy it.
            g.state.priority = p;
            let destroy = put(
                &mut g,
                p,
                if artifact { "Disenchant" } else { "Murder" },
                Zone::Hand,
            );
            paid(
                &mut g,
                destroy,
                false,
                Some(Target::Card(protected)),
                if artifact {
                    [1, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 2, 0, 0, 1]
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
            normal(
                &mut g,
                p,
                bofur,
                [1, 0, 0, 0, 0, 0],
                Some((1, 1)),
                Keywords::LIFELINK,
            );
            let life = g.state.players[p.index()].life;
            let _ = g.apply_damage_batch(&[DamageEvent {
                preventable: true,
                source: g.damage_source(bofur),
                target: Target::Player(p.other()),
                amount: 1,
                combat: true,
            }]);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            g.finish_cleanup();
            assert!(!g.keywords(protected).contains(Keywords::HEXPROOF));
            assert!(!g.keywords(protected).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

#[test]
fn hob_inherited_two_glamdring_exact_six_all_recovery_uses_normal_outside_stack_types() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let old = put(&mut g, p, "Shock", Zone::Graveyard);
        let eligible: Vec<_> = ["Shock", "Quick Study", "Duress"]
            .into_iter()
            .map(|n| put(&mut g, p, n, Zone::Library))
            .collect();
        let excluded: Vec<_> = ["Bilbo Baggins, Burglar", "Forest", "Goldvein Pick"]
            .into_iter()
            .map(|n| put(&mut g, p, n, Zone::Library))
            .collect();
        let glam = put(&mut g, p, "Glamdring, Foe-hammer", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        paid(&mut g, glam, true, None, [0, 1, 0, 0, 0, 3]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 6);
        for id in eligible {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
        for id in excluded {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(old).zone, Zone::Graveyard);
        assert!(g.observe(p).milled_choice.is_none());
        normal(&mut g, p, glam, [0, 0, 0, 0, 0, 2], None, Keywords::NONE);
        assert!(!g.is_creature(glam));
        assert!(g.is_equipment(glam));
    }
}

#[test]
fn hob_inherited_two_gloin_paid_opponent_only_damage_and_first_own_main_mana_packet() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let gloin = put(&mut g, p, "Glóin the Mighty", Zone::Hand);
        paid(&mut g, gloin, true, None, [0, 0, 0, 1, 0, 2]);
        settle(&mut g);
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
        assert_eq!(g.object_view(large).damage, 1);
        assert_eq!(g.object_view(own).damage, 0);
        assert_eq!(g.object_view(artifact).damage, 0);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        normal(
            &mut g,
            p,
            gloin,
            [0, 0, 0, 1, 0, 3],
            Some((4, 3)),
            Keywords::NONE,
        );
        for (active, step) in [(p.other(), Step::Main1), (p, Step::Main2)] {
            g.state.active = active;
            g.begin_step(step);
            g.give_priority();
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        g.state.active = p;
        g.begin_step(Step::Main1);
        g.give_priority();
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), 1);
        g.move_object(gloin, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 2, 0, 0]);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn hob_inherited_two_bird_no_eligible_milled_card_completes_without_finding_old_spell() {
    let p = Player::P0;
    let mut g = scenario(p);
    let old = put(&mut g, p, "Shock", Zone::Graveyard);
    for n in [
        "Forest",
        "Goldvein Pick",
        "Grizzly Bears",
        "Bilbo Baggins, Burglar",
    ] {
        put(&mut g, p, n, Zone::Library);
    }
    let bird = put(&mut g, p, "Most Decrepit Old Bird", Zone::Hand);
    paid(&mut g, bird, true, None, [0, 1, 0, 0, 0, 1]);
    settle(&mut g);
    assert!(g.observe(p).milled_choice.is_none());
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
    assert!(g.can_play_adventure(p, bird));
}

#[test]
fn hob_inherited_two_velvetwing_one_two_and_partial_invalid_targets_then_paid_flying_body() {
    {
        let p = Player::P0;
        for (count, stale) in [(1, false), (2, false), (2, true)] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let wing = put(&mut g, p, "Velvetwing Butterflies", Zone::Hand);
            g.state.active = p.other();
            announce_face(&mut g, wing, true, [1, 0, 0, 0, 0, 1]);
            reject(&mut g, Action::FinishTargets);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if count == 2 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            finish(&mut g);
            if stale {
                g.move_object(own, Zone::Hand);
                g.move_object(own, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.object_view(own).tapped, !stale);
            assert_eq!(g.object_view(enemy).tapped, count == 2);
            normal(
                &mut g,
                p,
                wing,
                [1, 0, 0, 0, 0, 2],
                Some((2, 2)),
                Keywords::FLYING,
            );
        }
    }
}
