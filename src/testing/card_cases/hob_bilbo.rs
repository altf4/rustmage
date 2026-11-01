//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid resolution casts retain chosen-face, origin, cost and exact-object facts.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Bilbo budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn bilbo(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Bilbo, Thief in the Night", Zone::Battlefield)
}
fn attack(g: &mut Game, p: Player, id: CardId) {
    g.state.active = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
    g.prepare_triggers();
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(g.actor(), Some(p));
}
fn select(g: &mut Game, id: CardId, face: CastFace) {
    let object = g.object_ref(id);
    act(g, Action::CastExiledDuringResolution { object, face });
}
fn finish_offer(g: &mut Game) {
    act(g, Action::FinishResolutionCasts);
    finish(g);
}
fn offered(g: &Game, id: CardId, face: CastFace) -> bool {
    g.legal_actions()
        .contains(&Action::CastExiledDuringResolution {
            object: g.object_ref(id),
            face,
        })
}
#[test]
fn hob_bilbo_paid_body_and_hand_casts_keep_printed_cost() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bilbo, Thief in the Night", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, id, None);
        finish(&mut g);
        assert_eq!(stats(&g, id), (2, 2));
        assert!(g.has_subtype(id, "Halfling") && g.has_subtype(id, "Rogue"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: id });
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: pick });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, pick, None);
        finish(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_bilbo_optional_nontargeting_choice_filters_controller_kind_and_affordability() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enchant = put(&mut g, p, "Witness Protection", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        attack(&mut g, p, b);
        for id in [artifact, creature, enchant, land, foreign] {
            assert!(!offered(&g, id, CastFace::Normal));
        }
        let public =
            serde_json::to_value(g.observe(p.other()).resolution_free_casts.unwrap()).unwrap();
        assert_eq!(public["paid"]["kinds"], 28);
        assert_eq!(public["paid"]["exile_instants_sorceries"], true);
        assert_eq!(public["casts_remaining"], 1);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        finish_offer(&mut g);
        for id in [artifact, creature, enchant, land, foreign] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}

#[test]
fn hob_bilbo_printed_mana_alternative_or_additional_sacrifice_both_work() {
    {
        let p = Player::P0;
        for alternate in [false, true] {
            let mut g = scenario(p);
            let b = bilbo(&mut g, p);
            let spell = put(&mut g, p, "Stir Up Trouble", Zone::Graveyard);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let fodder = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            g.state.players[p.index()].mana = if alternate {
                [0, 0, 1, 0, 0, 3]
            } else {
                [0, 0, 1, 0, 0, 0]
            };
            attack(&mut g, p, b);
            select(&mut g, spell, CastFace::Normal);
            act(
                &mut g,
                Action::ChooseSpellCost {
                    cost: if alternate { 1 } else { 0 },
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if !alternate {
                let object = g.object_ref(fodder);
                act(&mut g, Action::ChooseAdditionalCost { object });
            }
            pay(&mut g);
            finish_offer(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(fodder).zone,
                if alternate {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn hob_bilbo_mana_payment_sacrificing_provider_preserves_locked_reduced_cost() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(b)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        attack(&mut g, p, b);
        // Lotus Ring grants vigilance, so the actual attacker remains untapped.
        assert!(!g.object_view(b).tapped);
        assert!(offered(&g, pick, CastFace::Normal));
        select(&mut g, pick, CastFace::Normal);
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            1
        );
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: b,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 2, 0, 0, 0]);
    }
}
#[test]
fn hob_bilbo_library_origin_reduction_floors_generic_and_preserves_colored() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        bilbo(&mut g, p);
        put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        act(&mut g, Action::AnnounceCast { card: bear });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let shock = put(&mut g, p, "Shock", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(shock), false);
        reject(&mut g, Action::AnnounceExile { card: shock });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceExile { card: shock });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(
            g.object_view(shock).zone,
            Zone::Graveyard,
            "ordinary exile permission has no Bilbo replacement"
        );
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hob_bilbo_doubled_attack_packets_each_offer_exactly_one_paid_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(b)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        attack(&mut g, p, b);
        select(&mut g, pick, CastFace::Normal);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(!offered(&g, shock, CastFace::Normal));
        act(&mut g, Action::FinishResolutionCasts);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        select(&mut g, shock, CastFace::Normal);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
