//! Paid casting permissions, independent resolution choices and owned replay.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;
use crate::cards::CreatureType;

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
        flash_surcharge: false,
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
                .contains(&Action::AnnounceGrantedFlashback {
                    card: spell,
                    flash_surcharge: false
                })
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
        checked(
            &mut child,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
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
                .contains(&Action::AnnounceGrantedFlashback {
                    card: spell,
                    flash_surcharge: false
                })
        );
        let discard = fixture(&mut g, p, "Forest", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
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
                flash_surcharge: false,
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
                flash_surcharge: false,
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
                flash_surcharge: false,
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
        checked(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
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

#[test]
fn casting_permissions_free_flash_surcharge_pays_only_extra_cost_for_all_three_supported_bodies() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Harbinger of the Tides",
            "Mystical Tether",
            "Asinine Antics",
        ] {
            let mut g = setup(p);
            fixture(&mut g, p, "Omniscience", Zone::Battlefield);
            let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].tapped = true;
            let spell = fixture(&mut g, p, name, Zone::Hand);
            g.state.active = p.other();
            g.state.priority = p;
            let action = Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: true,
            };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, action.clone());
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            reject(&mut g, free(spell));
            reject(&mut g, Action::AnnounceFlashSurcharge { card: spell });
            checked(&mut g, action);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::PayCosts { .. })
            });
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            finish_cast(&mut g);
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.method, CastMethod::Free);
            assert!(cast.flash_surcharge);
            assert_eq!(cast.payment, [0, 0, 0, 0, 0, 2]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            match name {
                "Harbinger of the Tides" => assert_eq!(g.object_view(target).zone, Zone::Hand),
                "Mystical Tether" => assert_eq!(g.object_view(target).zone, Zone::Exile),
                "Asinine Antics" => assert_eq!(
                    g.stats(target),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                ),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn casting_permissions_sphinx_flashback_surcharge_retains_printed_cost_and_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sphinx = fixture(&mut g, p, "Sphinx of Forgotten Lore", Zone::Battlefield);
        let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = fixture(&mut g, p, "Asinine Antics", Zone::Graveyard);
        grant(&mut g, sphinx, spell);
        let action = Action::AnnounceGrantedFlashback {
            card: spell,
            flash_surcharge: true,
        };
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 3];
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        reject(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
        checked(&mut g, action);
        advance(&mut g, |g| {
            matches!(g.state.pending, Pending::PayCosts { .. })
        });
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 2, 0, 0, 0, 2],
            },
        );
        finish_cast(&mut g);
        let cast = g.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::GrantedFlashback);
        assert!(cast.flash_surcharge);
        assert_eq!(cast.payment, [0, 2, 0, 0, 0, 4]);
        settle(&mut g);
        assert_eq!(
            g.stats(target),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn smoke_preserves_spare_sources_and_funds_announced_spells_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [0; 6];
        }
        let first = fixture(&mut g, p, "Island", Zone::Battlefield);
        let second = fixture(&mut g, p, "Island", Zone::Battlefield);
        let pick = |g: &Game| {
            let actions = g.legal_actions();
            actions[crate::policy::smoke_index(g, &actions)].clone()
        };
        assert_eq!(pick(&g), Action::Pass, "empty hand must preserve mana");
        fixture(&mut g, p, "Gigantosaurus", Zone::Hand);
        assert_eq!(pick(&g), Action::Pass, "uncastable hand must preserve mana");
        let spell = fixture(&mut g, p, "Opt", Zone::Hand);
        assert_eq!(pick(&g), Action::AnnounceCast { card: spell });
        for _ in 0..20 {
            if g.state.stack.iter().any(|item| item.card == spell) {
                break;
            }
            let action = pick(&g);
            assert!(!matches!(action, Action::Pass | Action::Concede));
            checked(&mut g, action);
        }
        let cast = g
            .state
            .stack
            .iter()
            .find(|item| item.card == spell)
            .unwrap();
        assert_eq!(cast.cast.unwrap().payment, [0, 1, 0, 0, 0, 0]);
        assert_ne!(
            g.state.objects[first as usize].tapped,
            g.state.objects[second as usize].tapped
        );
        assert_eq!(pick(&g), Action::Pass, "do not waste the remaining Island");
    }
}

fn restricted_angel_mana(g: &mut Game, p: Player) {
    let giada = fixture(g, p, "Giada, Font of Hope", Zone::Battlefield);
    checked(
        g,
        Action::ActivateMana {
            card: giada,
            ability: 0,
            production: [1, 0, 0, 0, 0, 0],
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn free_flash_surcharge_preserves_restricted_mana_and_zero_payment_gives_no_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        fixture(&mut g, p, "Omniscience", Zone::Battlefield);
        let spell = fixture(&mut g, p, "Harbinger of the Tides", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        restricted_angel_mana(&mut g, p);
        g.state.active = p.other();
        let action = Action::AnnounceFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: true,
        };
        assert!(!g.legal_actions().contains(&action));
        reject(&mut g, action.clone());
        g.add_mana(p, [0, 0, 0, 0, 0, 2]);
        checked(&mut g, action);
        finish_cast(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 0, 0, 2]
        );
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);

        let mut g = setup(p);
        fixture(&mut g, p, "Omniscience", Zone::Battlefield);
        let dragon = fixture(&mut g, p, "Shivan Dragon", Zone::Hand);
        let orb = fixture(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: orb,
                ability: 0,
                production: [0, 0, 0, 1, 0, 0],
                payment: [0; 6],
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, free(dragon));
        finish_cast(&mut g);
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, [0; 6]);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        settle(&mut g);
        assert!(!g.keywords(dragon).contains(Keywords::HASTE));
    }
}

#[test]
fn granted_flashback_additional_discard_cannot_spend_angel_only_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sphinx = fixture(&mut g, p, "Sphinx of Forgotten Lore", Zone::Battlefield);
        let spell = fixture(&mut g, p, "Thrill of Possibility", Zone::Graveyard);
        fixture(&mut g, p, "Island", Zone::Hand);
        grant(&mut g, sphinx, spell);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        restricted_angel_mana(&mut g, p);
        let action = Action::AnnounceGrantedFlashback {
            card: spell,
            flash_surcharge: false,
        };
        assert!(!g.legal_actions().contains(&action));
        reject(&mut g, action.clone());
        g.add_mana(p, [0, 0, 0, 0, 0, 1]);
        checked(&mut g, action);
        finish_cast(&mut g);
        let cast = g.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::GrantedFlashback);
        assert_eq!(cast.payment, [0, 0, 0, 1, 0, 1]);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.card == Card::Island && o.zone == Zone::Graveyard)
        );
    }
}

// Exercise the generic permission core with an already supported Adventure.
// This calls the production grant directly, not Zul's Zombie-only target action:
// Gollum is a Halfling. Paid Zul tests above independently verify that filter.
fn direct_adventure_card_permission(g: &mut Game, p: Player, kind: GraveyardCastKind) -> CardId {
    let source = fixture(g, p, "Zul Ashur, Lich Lord", Zone::Battlefield);
    let card = fixture(
        g,
        p,
        "Gollum, Silent Slinker // Meager Meal",
        Zone::Graveyard,
    );
    let mut item = g.cast_preview(
        p,
        source,
        Zone::Battlefield,
        g.object_ref(source),
        0,
        [0; 6],
    );
    item.targets.push(g.bind_target(Target::Card(card)));
    g.grant_graveyard_cast(&item, kind);
    let event = g.transition_object(g.object_view(source), Zone::Graveyard, false);
    g.finish_zone_events(&[event], &[]);
    card
}

#[test]
fn exact_card_permission_casts_normal_or_adventure_with_actual_face_mana_then_recasts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = direct_adventure_card_permission(&mut g, p, GraveyardCastKind::Cast);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let court = fixture(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(court),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Halfling").unwrap()),
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: court,
                ability: 1,
                production: [0, 0, 1, 0, 0, 0],
                payment: [0; 6],
                counter_payment: Counters::default(),
            },
        );
        let original = g.object_ref(card);
        let choice = g.snapshot();
        assert!(g.legal_actions().contains(&Action::AnnounceCast { card }));
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card })
        );
        reject(&mut g, Action::AnnounceAdventure { card });
        checked(&mut g, Action::AnnounceCast { card });
        finish_cast(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        g.restore(&choice).unwrap();
        assert_eq!(g.object_ref(card), original);
        g.add_mana(p, [0, 0, 1, 0, 0, 0]);
        checked(&mut g, Action::AnnounceAdventure { card });
        finish_cast(&mut g);
        let item = g.state.stack.last().unwrap();
        let cast = item.cast.unwrap();
        assert_eq!(cast.origin, Zone::Graveyard);
        assert_eq!(cast.face, CastFace::Adventure);
        assert_eq!(cast.method, CastMethod::Normal);
        assert_eq!(cast.payment, [0, 0, 1, 0, 0, 0]);
        assert_eq!(item.source_card.name(), "Meager Meal");
        assert!(!item.source_card.is_creature());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 3]);
        assert!(!g.has_graveyard_cast(p, card, GraveyardCastKind::Cast));
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert!(g.can_play_adventure(p, card));
        let exiled = g.object_ref(card);
        assert_ne!(original, exiled);
        checked(&mut g, Action::AnnounceExile { card });
        finish_cast(&mut g);
        let cast = g.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.origin, Zone::Exile);
        assert_eq!(cast.face, CastFace::Normal);
        assert_eq!(cast.payment, [0, 0, 1, 0, 0, 3]);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        assert!(!g.can_play_adventure(p, card));
    }
}

#[test]
fn alternate_graveyard_face_requires_exact_unexpired_card_permission_not_flashback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = direct_adventure_card_permission(&mut g, p, GraveyardCastKind::Cast);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceAdventure { card })
        );
        let permission = g.snapshot();
        next_turn(&mut g);
        advance(&mut g, |g| {
            g.state.active == p
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        g.add_mana(p, [30; 6]);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card })
        );
        reject(&mut g, Action::AnnounceAdventure { card });
        g.restore(&permission).unwrap();
        let old = g.object_ref(card);
        for zone in [Zone::Hand, Zone::Graveyard] {
            let event = g.transition_object(g.object_view(card), zone, false);
            g.finish_zone_events(&[event], &[]);
        }
        assert_ne!(old, g.object_ref(card));
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card })
        );
        reject(&mut g, Action::AnnounceAdventure { card });
        let mut g = setup(p);
        let card = direct_adventure_card_permission(&mut g, p, GraveyardCastKind::Flashback);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card })
        );
        reject(&mut g, Action::AnnounceAdventure { card });
    }
}
