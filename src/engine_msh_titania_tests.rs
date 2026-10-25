//! Complete hybrid/additional casting and captured-payer ward union scenarios.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const NAME: &str = "Titania, Rugged Rumbler";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
}
fn select_discard(g: &mut Game, card: CardId) {
    let object = g.object_ref(card);
    checked(g, Action::ChooseAdditionalCost { object });
}
fn finish(g: &mut Game) {
    for _ in 0..24 {
        if matches!(g.state.pending, Pending::ZoneReplacement { .. }) {
            return;
        }
        if g.state.announcing.is_none() && !matches!(g.state.pending, Pending::ManaLotSpend { .. })
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap_or_else(|| panic!("unpaid {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid announcement");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
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
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn titan(
    g: &mut Game,
    p: Player,
    color: usize,
    mana_route: bool,
    discard: Option<CardId>,
) -> CardId {
    let id = named(g, p, NAME, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    checked(
        g,
        Action::ChooseSpellCost {
            cost: if color == 4 { 0 } else { 2 } + u16::from(mana_route),
        },
    );
    if let Some(card) = discard {
        checked(
            g,
            Action::ChooseAdditionalCost {
                object: g.object_ref(card),
            },
        );
    }
    finish(g);
    settle(g);
    id
}
fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn ward(g: &mut Game, p: Player, target: CardId, spell_name: &str) -> CardId {
    g.state.priority = p.other();
    let spell = named(g, p.other(), spell_name, Zone::Hand);
    cast(g, spell, None, &[Target::Card(target)]);
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    assert_eq!(g.actor(), Some(p.other()));
    assert_eq!(
        g.observe(p.other())
            .resolution_payment
            .as_ref()
            .unwrap()
            .payer,
        p.other()
    );
    spell
}
#[test]
fn msh_titania_both_hybrid_equivalents_cross_both_paid_additional_routes() {
    for p in [Player::P0, Player::P1] {
        for color in [2, 4] {
            for mana_route in [false, true] {
                let mut g = setup(p);
                let land = named(&mut g, p, "Forest", Zone::Hand);
                let mut pool = [0; 6];
                pool[color] = 1;
                pool[5] = if mana_route { 4 } else { 2 };
                g.state.players[p.index()].mana = pool;
                let id = titan(&mut g, p, color, mana_route, (!mana_route).then_some(land));
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 5,
                        toughness: 5
                    }
                );
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert!(g.has_subtype(id, "Human") && g.has_subtype(id, "Villain"));
                assert_eq!(
                    g.object_view(land).zone,
                    if mana_route {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.state.players[p.index()].cards_discarded_this_turn,
                    u64::from(!mana_route)
                );
            }
        }
    }
}
#[test]
fn msh_titania_black_hybrid_never_waives_mandatory_additional_cost() {
    for p in [Player::P0, Player::P1] {
        for color in [2, 4] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 2;
            reject(&mut g, Action::AnnounceCast { card: id });
            let card = named(&mut g, p, "Shock", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: id });
            reject(
                &mut g,
                Action::ChooseSpellCost {
                    cost: if color == 4 { 1 } else { 3 },
                },
            );
            checked(
                &mut g,
                Action::ChooseSpellCost {
                    cost: if color == 4 { 0 } else { 2 },
                },
            );
            let enemy = named(&mut g, p.other(), "Forest", Zone::Hand);
            for c in [enemy, id] {
                let object = g.object_ref(c);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            select_discard(&mut g, card);
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_titania_cost_reductions_preserve_selected_route_even_when_totals_coincide() {
    for p in [Player::P0, Player::P1] {
        for mana_route in [false, true] {
            let mut g = setup(p);
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Player(p.other())]);
            settle(&mut g);
            for _ in 0..4 {
                named(&mut g, p, "Raging Battle Mouse", Zone::Battlefield);
            }
            let card = named(&mut g, p, "Forest", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            let id = titan(&mut g, p, 4, mana_route, (!mana_route).then_some(card));
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(card).zone,
                if mana_route {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_titania_omniscience_free_cast_still_chooses_discard_or_two() {
    for p in [Player::P0, Player::P1] {
        for mana_route in [false, true] {
            for reduce in [false, true] {
                let mut g = setup(p);
                named(&mut g, p, "Omniscience", Zone::Battlefield);
                if reduce {
                    named(&mut g, p, "Honest Rutstein", Zone::Battlefield);
                }
                let card = named(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
                let id = named(&mut g, p, NAME, Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                if mana_route {
                    g.state.players[p.index()].mana[5] = if reduce { 1 } else { 2 };
                }
                checked(
                    &mut g,
                    Action::AnnounceFree {
                        card: id,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseSpellCost {
                        cost: u16::from(mana_route),
                    },
                );
                if !mana_route {
                    select_discard(&mut g, card);
                }
                finish(&mut g);
                settle(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert_eq!(
                    g.object_view(card).zone,
                    if mana_route {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            }
        }
    }
}
#[test]
fn msh_titania_free_cast_no_hand_or_mana_is_atomic_and_stack_copy_pays_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Omniscience", Zone::Battlefield);
        let id = named(&mut g, p, NAME, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceFree {
                card: id,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        g.state.players[p.index()].mana[5] = 2;
        checked(
            &mut g,
            Action::AnnounceFree {
                card: id,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        finish(&mut g);
        let before = g.state.players[p.index()].spells_cast_this_turn;
        let mut item = g.state.stack.last().unwrap().clone();
        item.targets = vec![g.bind_target(Target::Spell(id))];
        assert!(!g.copy_target_spell(&item, 0, false, CopyExceptions::NONE));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, before);
        assert!(g.state.announcing.is_none());
        assert_eq!(g.state.stack.len(), 2);
    }
}
#[test]
fn msh_titania_restricted_villain_spell_mana_pays_additional_cast_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        for _ in 0..4 {
            let land = named(&mut g, p, "Villainous Hideout", Zone::Battlefield);
            checked(&mut g, mana(land, 1, [0, 0, 0, 0, 1, 0]));
        }
        let id = titan(&mut g, p, 2, true, None);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_titania_ward_discard_mana_and_decline_paid_both_seats() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = setup(p);
            let id = titan(&mut g, p, 4, true, None);
            let card = named(&mut g, p.other(), "Forest", Zone::Hand);
            let spell = ward(&mut g, p, id, "Shock");
            assert!(g.observe(p).effect_choice.is_none());
            assert_eq!(
                g.observe(p.other()).effect_choice.unwrap().required[p.other().index()],
                1
            );
            reject(&mut g, Action::Pass);
            reject(&mut g, Action::Discard { card: id });
            match route {
                0 => checked(&mut g, Action::Discard { card }),
                1 => checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    },
                ),
                _ => checked(&mut g, Action::DeclineResolutionMana),
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, if route == 2 { 0 } else { 2 });
            assert_eq!(
                g.object_view(card).zone,
                if route == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(g.state.resolving.is_none());
        }
    }
}
#[test]
fn msh_titania_ward_mana_window_activates_real_producers_without_priority() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAME, Zone::Battlefield);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        let a = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        ward(&mut g, p, id, "Shock");
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        for card in [a, b] {
            checked(&mut g, Action::TapMana { card });
            assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        }
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(id).damage, 2);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
    }
}
#[test]
fn msh_titania_ward_rejects_spell_only_restricted_mana_but_keeps_discard_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAME, Zone::Battlefield);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        for _ in 0..2 {
            let h = named(&mut g, p.other(), "Villainous Hideout", Zone::Battlefield);
            checked(&mut g, mana(h, 1, [0, 0, 0, 0, 1, 0]));
        }
        let card = named(&mut g, p.other(), "Forest", Zone::Hand);
        ward(&mut g, p, id, "Shock");
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        checked(&mut g, Action::Discard { card });
        settle(&mut g);
        assert_eq!(g.object_view(id).damage, 2);
        assert_eq!(g.state.players[p.other().index()].mana, [0, 0, 0, 0, 2, 0]);
    }
}
#[test]
fn msh_titania_ward_hidden_hands_have_equal_full_nonactor_observations() {
    for p in [Player::P0, Player::P1] {
        let mut a = setup(p);
        let id = named(&mut a, p, NAME, Zone::Battlefield);
        let card = named(&mut a, p.other(), "Forest", Zone::Hand);
        let mut b = a.clone();
        b.state.objects[card as usize].card = Card::from_name("Shock").unwrap();
        ward(&mut a, p, id, "Shock");
        ward(&mut b, p, id, "Shock");
        assert_eq!(
            serde_json::to_value(a.observe(p)).unwrap(),
            serde_json::to_value(b.observe(p)).unwrap()
        );
        assert!(a.observe(p).objects.iter().all(|o| o.id != card));
        assert!(b.observe(p).objects.iter().all(|o| o.id != card));
        checked(&mut a, Action::Discard { card });
        checked(&mut b, Action::Discard { card });
        settle(&mut a);
        settle(&mut b);
        assert_eq!(a.object_view(id).damage, 2);
        assert_eq!(b.object_view(id).damage, 2);
    }
}
#[test]
fn msh_titania_ward_retains_payer_cause_after_source_and_spell_control_changes() {
    for p in [Player::P0, Player::P1] {
        for leave in 0..3 {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            let liege = named(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
            g.state.priority = p.other();
            let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Card(id)]);
            if leave == 0 {
                g.change_controller(g.object_ref(id), p.other());
            } else {
                g.move_object(id, Zone::Exile);
                if leave == 2 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            // No implemented stack-control spell is admitted here; directly perturb
            // that one stack fact to exercise CR702.21a's captured targeting player.
            g.state
                .stack
                .iter_mut()
                .find(|s| s.card == shock && s.kind == StackKind::Spell)
                .unwrap()
                .controller = p;
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            checked(&mut g, Action::Discard { card: liege });
            settle(&mut g);
            assert_eq!(g.object_view(liege).zone, Zone::Battlefield);
            assert_eq!(g.object_view(liege).controller, p.other());
            assert_eq!(g.object_view(id).damage, if leave == 0 { 2 } else { 0 });
        }
    }
}
#[test]
fn msh_titania_ward_exact_related_departure_skips_request_and_uncounterable_survives_decline() {
    for p in [Player::P0, Player::P1] {
        for uncounterable in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            if uncounterable {
                ward(&mut g, p, id, "Long Goodbye");
                checked(&mut g, Action::DeclineResolutionMana);
                settle(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            } else {
                g.state.priority = p.other();
                let spell = named(&mut g, p.other(), "Shock", Zone::Hand);
                cast(&mut g, spell, None, &[Target::Card(id)]);
                let exact = g.state.stack.iter().find(|s| s.card == spell).unwrap().id;
                assert!(g.counter_stack_item(exact, false));
                // Reusing the physical card creates a new stack ID; the old ward
                // packet cannot charge or counter the new unrelated object.
                g.move_object(spell, Zone::Hand);
                g.state.priority = p.other();
                cast(&mut g, spell, None, &[Target::Player(p)]);
                settle(&mut g);
                assert_eq!(g.object_view(id).damage, 0);
                assert_eq!(g.state.players[p.index()].life, 18);
                assert!(g.state.resolving.is_none());
            }
        }
    }
}
#[test]
fn msh_titania_ward_own_targeting_and_suppressed_ability_are_quiet() {
    for p in [Player::P0, Player::P1] {
        for suppress in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            if suppress {
                let aura = named(&mut g, p, "Eaten by Piranhas", Zone::Hand);
                cast(&mut g, aura, None, &[Target::Card(id)]);
                settle(&mut g);
                g.state.priority = p.other();
            }
            let actor = if suppress { p.other() } else { p };
            let spell = named(&mut g, actor, "Shock", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(id)]);
            assert!(g.state.pending_triggers.is_empty());
            settle(&mut g);
            assert_eq!(
                g.object_view(id).zone,
                if suppress {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn msh_titania_independent_intrinsic_and_attachment_wards_each_require_payment() {
    for p in [Player::P0, Player::P1] {
        for pay_second in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            let aura = named(&mut g, p, "Super Strength", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(id)]);
            settle(&mut g);
            let card = named(&mut g, p.other(), "Forest", Zone::Hand);
            ward(&mut g, p, id, "Shock");
            let first_is_union = g
                .state
                .resolving
                .as_ref()
                .unwrap()
                .mana_payment
                .as_ref()
                .unwrap()
                .discard;
            if first_is_union {
                checked(&mut g, Action::Discard { card });
            } else {
                checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            if !pay_second {
                checked(&mut g, Action::DeclineResolutionMana);
            } else if first_is_union {
                checked(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
            } else {
                checked(&mut g, Action::Discard { card });
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, if pay_second { 2 } else { 0 });
        }
    }
}
#[test]
fn msh_titania_ward_discard_replacements_pay_once_with_owned_suspension() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            let card = named(&mut g, p.other(), "Darksteel Colossus", Zone::Hand);
            named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            ward(&mut g, p, id, "Shock");
            checked(&mut g, Action::Discard { card });
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            assert!(g.state.resolving.as_ref().unwrap().mana_payment.is_none());
            assert!(g.state.resolving.as_ref().unwrap().choice.is_none());
            reject(&mut g, Action::DeclineResolutionMana);
            checked(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: if exile {
                        ZoneReplacementKind::Exile
                    } else {
                        ZoneReplacementKind::RevealShuffle
                    },
                },
            );
            settle(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                if exile { Zone::Exile } else { Zone::Library }
            );
            assert_eq!(g.object_view(id).damage, 2);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                1
            );
        }
    }
}
#[test]
fn msh_titania_ward_opposing_paid_activated_ability_survives_its_source_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p.other());
            let id = named(&mut g, p, NAME, Zone::Battlefield);
            let fire = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Hand);
            cast(&mut g, fire, None, &[]);
            settle(&mut g);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: fire,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(fire).zone, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            checked(
                &mut g,
                if pay {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, i128::from(pay));
        }
    }
}
#[test]
fn msh_titania_retargeted_spell_copy_creates_its_own_exact_ward_association() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let id = named(&mut g, p, NAME, Zone::Battlefield);
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p)]);
        assert!(g.state.pending_triggers.is_empty());
        let sparks = named(&mut g, p.other(), "Choreographed Sparks", Zone::Hand);
        cast(&mut g, sparks, Some(0), &[Target::Spell(shock)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(id),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().id;
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(g.observe(p).resolution_payment.unwrap().victim, Some(copy));
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(id).damage, 0);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}
#[test]
fn msh_titania_zero_resource_ward_still_presents_union_and_can_decline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAME, Zone::Battlefield);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        ward(&mut g, p, id, "Shock");
        assert_eq!(g.hand_count(p.other()), 0);
        assert_eq!(
            g.observe(p.other()).effect_choice.unwrap().required[p.other().index()],
            1
        );
        assert!(g.observe(p).effect_choice.is_none());
        assert_eq!(
            g.legal_actions()
                .iter()
                .filter(|a| !matches!(a, Action::Concede))
                .count(),
            1
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(id).damage, 0);
    }
}
#[test]
fn msh_titania_cast_discard_replacements_keep_additional_paid_through_suspension() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = setup(p);
            let card = named(&mut g, p, "Darksteel Colossus", Zone::Hand);
            named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let id = named(&mut g, p, NAME, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            checked(&mut g, Action::AnnounceCast { card: id });
            checked(&mut g, Action::ChooseSpellCost { cost: 2 });
            select_discard(&mut g, card);
            finish(&mut g);
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            checked(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: if exile {
                        ZoneReplacementKind::Exile
                    } else {
                        ZoneReplacementKind::RevealShuffle
                    },
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(card).zone,
                if exile { Zone::Exile } else { Zone::Library }
            );
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
