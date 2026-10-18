//! Paid casting permissions, independent resolution choices and owned replay.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
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
fn advance(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("missing decision: {:?}", g.state.pending);
}
fn finish_cast(g: &mut Game) {
    for _ in 0..40 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("unfinished cast: {:?}", g.state.pending);
}
fn grant(g: &mut Game, source: CardId, target: CardId) {
    let p = g.state.objects[source as usize].controller;
    advance(g, |g| matches!(g.state.pending, Pending::Attackers));
    checked(g, Action::Attack { card: source });
    checked(g, Action::FinishAttackers);
    advance(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        })
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    advance(g, |g| {
        g.has_graveyard_cast(p, target, GraveyardCastKind::Flashback)
    });
}
fn free(card: CardId) -> Action {
    Action::AnnounceFree {
        card,
        face: CastFace::Normal,
    }
}
#[test]
fn casting_permissions_sphinx_paid_flash_attack_competing_costs_source_exit_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sphinx = fixture(&mut g, p, "Sphinx of Forgotten Lore", Zone::Hand);
        g.state.active = p.other();
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        cast(&mut g, sphinx, None, &[]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 2, 0, 0, 0, 2]
        );
        settle(&mut g);
        assert!(g.keywords(sphinx).contains(Keywords::FLYING));
        next_turn(&mut g);
        advance(&mut g, |g| {
            g.state.pending == Pending::Priority
                && g.state.active == p
                && g.state.step == Step::Main1
        });
        let spell = fixture(&mut g, p, "Think Twice", Zone::Graveyard);
        grant(&mut g, sphinx, spell);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: spell })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceGrantedFlashback { card: spell })
        );
        let original = g.object_ref(spell);
        let snapshot = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.state.priority = p;
        child.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        let unsummon = fixture(&mut child, p, "Unsummon", Zone::Hand);
        cast(&mut child, unsummon, None, &[Target::Card(sphinx)]);
        settle(&mut child);
        assert_eq!(child.object_view(sphinx).zone, Zone::Hand);
        assert!(
            !child
                .legal_actions()
                .contains(&Action::AnnounceFlashback { card: spell })
        );
        checked(&mut child, Action::AnnounceGrantedFlashback { card: spell });
        assert!(child.observe(p).graveyard_cast_permissions.is_empty());
        finish_cast(&mut child);
        let item = child.state.stack.last().unwrap();
        assert_eq!(item.cast.unwrap().origin, Zone::Graveyard);
        assert_eq!(item.cast.unwrap().method, CastMethod::GrantedFlashback);
        assert_eq!(item.cast.unwrap().payment, [0, 1, 0, 0, 0, 1]);
        assert_ne!(item.object, original);
        settle(&mut child);
        assert_eq!(child.object_view(spell).zone, Zone::Exile);
        child.restore(&snapshot).unwrap();
        assert!(child.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
        child.move_object(spell, Zone::Hand);
        child.move_object(spell, Zone::Graveyard);
        assert!(!child.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
        assert_eq!(g.snapshot(), snapshot);
    }
}
#[test]
fn casting_permissions_sphinx_additional_discard_cost_counter_exit_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sphinx = fixture(&mut g, p, "Sphinx of Forgotten Lore", Zone::Battlefield);
        let spell = fixture(&mut g, p, "Thrill of Possibility", Zone::Graveyard);
        grant(&mut g, sphinx, spell);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceGrantedFlashback { card: spell })
        );
        let discard = fixture(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::AnnounceGrantedFlashback { card: spell });
        advance(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        let object = g.object_ref(discard);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish_cast(&mut g);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        checked(&mut g, Action::Pass);
        let negate = fixture(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, negate, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        // A later grant expires during cleanup, and cannot authorize a new turn.
        next_turn(&mut g);
        next_turn(&mut g);
        advance(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        let later = fixture(&mut g, p, "Think Twice", Zone::Graveyard);
        grant(&mut g, sphinx, later);
        next_turn(&mut g);
        assert!(g.observe(p).graveyard_cast_permissions.is_empty());
    }
}
#[test]
fn casting_permissions_zul_paid_activation_timing_target_filter_and_independent_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let zul = fixture(&mut g, p, "Zul Ashur, Lich Lord", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        cast(&mut g, zul, None, &[]);
        settle(&mut g);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == zul))
        );
        next_turn(&mut g);
        next_turn(&mut g);
        advance(&mut g, |g| {
            g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
                && g.state.active == p
        });
        let zombie = fixture(&mut g, p, "Diregraf Ghoul", Zone::Graveyard);
        let other = fixture(&mut g, p, "Bear Cub", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: zul,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(zombie),
            },
        );
        finish_cast(&mut g);
        assert!(g.object_view(zul).tapped);
        settle(&mut g);
        g.sacrifice_batch(&[(p, zul)]);
        g.state_based_actions();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceCast { card: zombie });
        g.state.active = p;
        checked(&mut g, Action::AnnounceCast { card: zombie });
        finish_cast(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Normal
        );
        settle(&mut g);
        assert_eq!(g.object_view(zombie).zone, Zone::Battlefield);
        assert!(g.object_view(zombie).tapped);
    }
}
#[test]
fn casting_permissions_ward_life_choices_affordability_exact_related_spell_and_atomicity() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let zul = fixture(&mut g, p.other(), "Zul Ashur, Lich Lord", Zone::Battlefield);
            let shock = fixture(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, shock, None, &[Target::Card(zul)]);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionLife { .. })
            });
            assert_eq!(g.actor(), Some(p));
            let request = g.observe(p).resolution_payment.unwrap();
            assert_eq!(request.life, Some(2));
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|s| Some(s.id) == request.victim)
                    .unwrap()
                    .card,
                shock
            );
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            let life = g.state.players[p.index()].life;
            if pay {
                g.state.players[p.index()].life = 1;
                reject(&mut g, Action::PayResolutionLife);
                g.state.players[p.index()].life = life;
                checked(&mut g, Action::PayResolutionLife);
                assert_eq!(g.state.players[p.index()].life, life - 2);
            } else {
                checked(&mut g, Action::DeclineResolutionLife);
            }
            settle(&mut g);
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(zul).zone,
                if pay {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn casting_permissions_omniscience_paid_optional_free_hand_cast_additional_cost_and_private_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let omni = fixture(&mut g, p, "Omniscience", Zone::Hand);
        g.state.players[p.index()].mana = [0, 3, 0, 0, 0, 7];
        cast(&mut g, omni, None, &[]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let giant = fixture(&mut g, p, "Gigantosaurus", Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == giant));
        checked(&mut g, free(giant));
        finish_cast(&mut g);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(giant).zone, Zone::Battlefield);
        let thrill = fixture(&mut g, p, "Thrill of Possibility", Zone::Hand);
        assert!(!g.legal_actions().contains(&free(thrill)));
        let discard = fixture(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, free(thrill));
        advance(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        let object = g.object_ref(discard);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish_cast(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        let gy = fixture(&mut g, p, "Think Twice", Zone::Graveyard);
        reject(&mut g, free(gy));
        g.move_object(omni, Zone::Graveyard);
        let hand = fixture(&mut g, p, "Gigantosaurus", Zone::Hand);
        reject(&mut g, free(hand));
    }
}
#[test]
fn casting_permissions_free_x_zero_kicker_and_alternate_faces_preserve_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        fixture(&mut g, p, "Omniscience", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let x = fixture(&mut g, p, "Exsanguinate", Zone::Hand);
        checked(&mut g, free(x));
        assert_ne!(g.state.pending, Pending::X { player: p });
        reject(&mut g, Action::ChooseX { value: 5 });
        finish_cast(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(0));
        settle(&mut g);
        let bear = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let roil = fixture(&mut g, p, "Into the Roil", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        checked(&mut g, free(roil));
        checked(&mut g, Action::ChooseKicker { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish_cast(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 1, 0, 0, 0, 1]
        );
        settle(&mut g);
        let omen = fixture(
            &mut g,
            p,
            "Whirlwing Stormbrood // Dynamic Soar",
            Zone::Hand,
        );
        let own = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceFree {
                card: omen,
                face: CastFace::Omen,
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish_cast(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().face,
            CastFace::Omen
        );
        settle(&mut g);
        assert_eq!(g.object_view(omen).zone, Zone::Library);
        let adventure = fixture(
            &mut g,
            p,
            "Bilbo Baggins, Burglar // Take a Glance",
            Zone::Hand,
        );
        checked(
            &mut g,
            Action::AnnounceFree {
                card: adventure,
                face: CastFace::Adventure,
            },
        );
        finish_cast(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(adventure).zone, Zone::Exile);
        g.state.active = p.other();
        g.state.priority = p;
        let sorcery = fixture(&mut g, p, "Exsanguinate", Zone::Hand);
        reject(&mut g, free(sorcery));
        reject(
            &mut g,
            Action::AnnounceFree {
                card: adventure,
                face: CastFace::Normal,
            },
        );
    }
}
#[test]
fn casting_permissions_granted_flashback_x_and_zul_turn_expiry_are_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sphinx = fixture(&mut g, p, "Sphinx of Forgotten Lore", Zone::Battlefield);
        let spell = fixture(&mut g, p, "Exsanguinate", Zone::Graveyard);
        grant(&mut g, sphinx, spell);
        advance(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 3];
        checked(&mut g, Action::AnnounceGrantedFlashback { card: spell });
        assert!(matches!(g.state.pending, Pending::X { .. }));
        reject(&mut g, Action::ChooseX { value: 4 });
        checked(&mut g, Action::ChooseX { value: 3 });
        finish_cast(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(3));
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 2, 0, 0, 3]
        );
        let before = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 3);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        let zul = fixture(&mut g, p, "Zul Ashur, Lich Lord", Zone::Battlefield);
        let zombie = fixture(&mut g, p, "Diregraf Ghoul", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: zul,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(zombie),
            },
        );
        finish_cast(&mut g);
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, zombie, GraveyardCastKind::Cast));
        let snapshot = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        next_turn(&mut child);
        assert!(!child.has_graveyard_cast(p, zombie, GraveyardCastKind::Cast));
        assert!(g.has_graveyard_cast(p, zombie, GraveyardCastKind::Cast));
        child.restore(&snapshot).unwrap();
        assert!(child.has_graveyard_cast(p, zombie, GraveyardCastKind::Cast));
    }
}
#[test]
fn casting_permissions_ward_does_not_offer_payment_after_related_spell_is_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let zul = fixture(&mut g, p.other(), "Zul Ashur, Lich Lord", Zone::Battlefield);
        let shock = fixture(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(zul)]);
        assert!(g.state.stack.iter().any(|s| s.related_stack.is_some()));
        let negate = fixture(&mut g, p, "Negate", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, negate, None, &[Target::Spell(shock)]);
        let life = g.state.players[p.index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        assert_eq!(g.object_view(zul).zone, Zone::Battlefield);
    }
}
