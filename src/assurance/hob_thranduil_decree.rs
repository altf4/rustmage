//! Exact exile permission after a successful counter, with independent cast routes.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Decree budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
    });
}
fn spell(g: &mut Game, owner: Player, name: &str) -> (CardId, u64) {
    g.state.active = owner;
    g.state.priority = owner;
    g.state.step = Step::Main1;
    let card = put(g, owner, name, Zone::Hand);
    cast(
        g,
        card,
        if name == "Shock" {
            Some(Target::Player(owner.other()))
        } else {
            None
        },
    );
    (
        card,
        g.state
            .stack
            .iter()
            .find(|s| s.source.is_some_and(|source| source.id == card))
            .unwrap()
            .id,
    )
}
fn decree(g: &mut Game, p: Player, target: u64) -> CardId {
    g.state.priority = p;
    let id = put(g, p, "Thranduil's Decree", Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
    let victim = g.state.stack.iter().find(|s| s.id == target).unwrap();
    let target = if victim.kind == StackKind::Spell && !victim.is_copy {
        Target::Spell(victim.source.unwrap().id)
    } else {
        Target::Stack(target)
    };
    cast(g, id, Some(target));
    id
}
fn free(card: CardId, face: CastFace) -> Action {
    Action::AnnounceExileFree {
        card,
        face,
        flash_surcharge: false,
    }
}
fn own_main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
}
fn received(g: &mut Game, p: Player, name: &str) -> CardId {
    let (id, stack) = spell(g, p.other(), name);
    decree(g, p, stack);
    finish(g);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
    assert!(g.has_exile_free_permission(p, id));
    id
}
#[test]
fn hob_decree_paid_instant_counters_opponent_permanent_and_free_cast_keeps_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (bear, stack) = spell(&mut g, p.other(), "Bear Cub");
        let spent = g.state.players[p.other().index()].mana;
        let decree = decree(&mut g, p, stack);
        finish(&mut g);
        assert_eq!(g.object_view(decree).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.players[p.other().index()].mana,
            spent,
            "countering refunds no original payment"
        );
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert!(g.has_exile_free_permission(p, bear));
        let view = g.observe(p.other()).exile_permissions;
        assert_eq!(view.len(), 1);
        assert_eq!(view[0].player, p);
        assert_eq!(view[0].object, g.object_ref(bear));
        assert_eq!(view[0].expiry, ExilePlayExpiry::WhileExiled);
        let json = serde_json::to_value(view[0]).unwrap();
        assert_eq!(json["without_paying_mana_cost"], true);
        assert_eq!(json["expiry"], "WhileExiled");
        assert!(!g.has_exile_free_permission(p.other(), bear));
        g.state.priority = p;
        reject(&mut g, free(bear, CastFace::Normal)); // still the opponent's turn
        own_main(&mut g, p);
        reject(&mut g, Action::AnnounceExile { card: bear });
        act(&mut g, free(bear, CastFace::Normal));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).controller, p);
        assert_eq!(g.object_view(bear).owner, p.other());
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(g.observe(p).exile_permissions.is_empty());
        g.move_object(bear, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}
#[test]
fn hob_decree_nonpermanent_countered_to_graveyard_or_other_exile_never_grants() {
    for p in [Player::P0, Player::P1] {
        for rip in [false, true] {
            let mut g = scenario(p);
            if rip {
                let id = put(&mut g, p, "Rest in Peace", Zone::Battlefield);
                g.register_zone_rule_card(g.state.objects[id as usize].card);
            }
            let (shock, stack) = spell(&mut g, p.other(), "Shock");
            decree(&mut g, p, stack);
            finish(&mut g);
            assert_eq!(
                g.object_view(shock).zone,
                if rip { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(g.state.players[p.index()].life, 20);
            assert!(g.observe(p).exile_permissions.is_empty());
        }
    }
}
#[test]
fn hob_decree_uncounterable_is_still_legal_target_but_never_exiled_or_granted() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (sphinx, stack) = spell(&mut g, p.other(), "Sphinx of the Final Word");
        decree(&mut g, p, stack);
        finish(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
        assert_eq!(g.object_view(sphinx).controller, p.other());
        assert!(g.observe(p).exile_permissions.is_empty());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_decree_stale_stack_target_and_illegal_ability_target_are_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (bear, stack) = spell(&mut g, p.other(), "Bear Cub");
        let d = decree(&mut g, p, stack);
        g.counter_stack_item(stack, false);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(d).zone, Zone::Graveyard);
        assert!(g.observe(p).exile_permissions.is_empty());
        own_main(&mut g, p);
        let fire = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: fire,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        let ability = g.state.stack.last().unwrap().id;
        let d = put(&mut g, p, "Thranduil's Decree", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        reject(&mut g, Action::AnnounceCast { card: d }); // the only stack object is an ability
        assert_eq!(g.object_view(d).zone, Zone::Hand);
        assert!(g.state.stack.iter().any(|s| s.id == ability));
    }
}
#[test]
fn hob_decree_free_permission_is_optional_lasts_across_turns_and_exact_departure_clears() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = received(&mut g, p, "Goldvein Pick");
        let exact = g.object_ref(card);
        let start = g.state.turn;
        for _ in 0..160 {
            if g.state.turn >= start + 2
                && g.state.active == p
                && matches!(g.state.step, Step::Main1)
                && g.state.pending == Pending::Priority
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert!(g.state.turn >= start + 2);
        assert_eq!(g.object_ref(card), exact);
        assert!(g.has_exile_free_permission(p, card));
        g.move_object(card, Zone::Hand);
        g.move_object(card, Zone::Exile);
        assert_ne!(g.object_ref(card), exact);
        assert!(!g.has_exile_free_permission(p, card));
        assert!(g.observe(p).exile_permissions.is_empty());
        own_main(&mut g, p);
        reject(&mut g, free(card, CastFace::Normal));
    }
}
#[test]
fn hob_decree_actual_countered_adventure_permanent_can_cast_either_face_for_free() {
    for p in [Player::P0, Player::P1] {
        for face in [CastFace::Normal, CastFace::Adventure] {
            let mut g = scenario(p);
            let card = received(&mut g, p, "Great Ugly-Looking Goblin // Clap! Snap!");
            own_main(&mut g, p);
            act(&mut g, free(card, face));
            pay(&mut g);
            let packet = g.state.stack.last().unwrap();
            assert_eq!(packet.cast.unwrap().method, CastMethod::Free);
            assert_eq!(packet.cast.unwrap().payment, [0; 6]);
            assert_eq!(
                packet.source.unwrap().card.name(),
                if face == CastFace::Normal {
                    "Great Ugly-Looking Goblin"
                } else {
                    "Clap! Snap!"
                }
            );
            finish(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                if face == CastFace::Normal {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert!(!g.has_exile_free_permission(p, card));
            if face == CastFace::Adventure {
                assert!(g.can_play_adventure(p, card));
                reject(&mut g, free(card, CastFace::Normal));
            }
        }
    }
}
#[test]
fn hob_decree_copied_permanent_spell_counter_creates_no_card_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let owner = p.other();
        let (bear, original) = spell(&mut g, owner, "Bear Cub");
        let sparks = put(&mut g, owner, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 1 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(bear),
            },
        );
        pay(&mut g);
        until(&mut g, |g| {
            g.state.stack.iter().any(|s| {
                s.is_copy && s.source.unwrap().card == Card::from_name("Bear Cub").unwrap()
            })
        });
        let copied = g
            .state
            .stack
            .iter()
            .find(|s| s.is_copy && s.source.unwrap().card == Card::from_name("Bear Cub").unwrap())
            .unwrap()
            .id;
        assert_ne!(copied, original);
        assert!(g.state.stack.iter().any(|s| s.id == original && !s.is_copy));
        decree(&mut g, p, copied);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(g.observe(p).exile_permissions.is_empty());
    }
}
#[test]
fn hob_decree_new_free_route_and_existing_paid_life_and_land_routes_are_independent() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = scenario(p);
            let id = put(&mut g, p.other(), "Shock", Zone::Exile);
            let object = g.object_ref(id);
            // Generic permission contract: all three independent routes on one exact card.
            g.grant_exile_free_permission(p, object);
            g.grant_exile_play_permission(p, object, false);
            let paid = g
                .state
                .exile_permissions
                .iter_mut()
                .find(|r| !r.without_paying_mana_cost)
                .unwrap();
            assert!(
                serde_json::to_value(*paid)
                    .unwrap()
                    .get("without_paying_mana_cost")
                    .is_none()
            );
            paid.spend_any_mana = true;
            paid.exile_on_graveyard = true;
            let mut life = *paid;
            life.life_instead_of_mana = true;
            life.spend_any_mana = false;
            life.exile_on_graveyard = false;
            g.state.exile_permissions.push(life);
            g.state.players[p.index()].mana = if route == 1 {
                [0, 1, 0, 0, 0, 0]
            } else {
                [0; 6]
            };
            act(
                &mut g,
                match route {
                    0 => free(id, CastFace::Normal),
                    1 => Action::AnnounceExile { card: id },
                    _ => Action::AnnounceExileLife {
                        card: id,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    },
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            if route == 2 {
                let clause = g
                    .observe(p)
                    .cast_costs
                    .unwrap()
                    .clauses
                    .iter()
                    .position(|r| r.clause == CastCostClause::AlternativeLife(1))
                    .unwrap() as u16;
                act(&mut g, Action::ChooseCastCost { clause });
            }
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.exile_on_graveyard, route == 1);
            assert_eq!(cast.spend_any_mana, route == 1);
            assert!(!cast.stash_origin);
            finish(&mut g);
            assert_eq!(
                g.object_view(id).zone,
                if route == 1 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.players[p.index()].life,
                if route == 2 { 19 } else { 20 }
            );
            let land = put(&mut g, p, "Forest", Zone::Exile);
            g.grant_exile_free_permission(p, g.object_ref(land));
            reject(&mut g, free(land, CastFace::Normal));
            reject(&mut g, Action::PlayLand { card: land });
            g.grant_exile_play_permission(p, g.object_ref(land), false);
            act(&mut g, Action::PlayLand { card: land });
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn hob_decree_free_cast_additional_costs_x_and_other_alternatives_remain_rules_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p.other(), "Stir Up Trouble", Zone::Exile);
        g.grant_exile_free_permission(p, g.object_ref(spell));
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let fodder = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, free(spell, CastFace::Normal));
        reject(&mut g, Action::ChooseSpellCost { cost: 1 });
        act(&mut g, Action::ChooseSpellCost { cost: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        let x = put(&mut g, p.other(), "Mind Spring", Zone::Exile);
        g.grant_exile_free_permission(p, g.object_ref(x));
        act(&mut g, free(x, CastFace::Normal));
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        reject(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn hob_decree_owned_free_announcement_and_exact_permission_restore_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = received(&mut g, p, "Goldvein Pick");
        own_main(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let action = free(card, CastFace::Normal);
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut raw, action.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        contracts::branch_contract(raw.clone());
        pay(&mut raw);
        finish(&mut raw);
        let expected = raw.snapshot();
        raw.restore(&saved).unwrap();
        act(&mut raw, action);
        pay(&mut raw);
        finish(&mut raw);
        assert_eq!(raw.snapshot(), expected);
        pay(&mut indexed);
        finish(&mut indexed);
        assert_eq!(indexed.snapshot(), expected);
        assert!(
            raw.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == card && o.zone == Zone::Battlefield)
        );
        let secret = put(&mut raw, p, "Shock", Zone::Hand);
        assert!(
            raw.observe(p.other())
                .objects
                .iter()
                .all(|o| o.id != secret)
        );
    }
}
#[test]
fn hob_decree_countered_adventure_is_a_sorcery_not_its_permanent_front() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let owner = p.other();
        let card = put(
            &mut g,
            owner,
            "Great Ugly-Looking Goblin // Clap! Snap!",
            Zone::Hand,
        );
        act(&mut g, Action::AnnounceAdventure { card });
        pay(&mut g);
        let stack = g.state.stack.last().unwrap().id;
        decree(&mut g, p, stack);
        finish(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        assert!(g.observe(p).exile_permissions.is_empty());
    }
}
#[test]
fn hob_decree_free_flash_permanent_is_legal_on_opponent_turn_and_own_spell_can_be_targeted() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = received(&mut g, p, "Spectral Sailor");
        g.state.priority = p;
        assert_eq!(g.state.active, p.other());
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, free(card, CastFace::Normal));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(card).controller, p);
        assert!(g.keywords(card).contains(Keywords::FLASH));
        let mut g = scenario(p);
        let (card, stack) = spell(&mut g, p, "Goldvein Pick");
        decree(&mut g, p, stack);
        finish(&mut g);
        assert!(g.has_exile_free_permission(p, card));
        assert_eq!(g.object_view(card).owner, p);
        act(&mut g, free(card, CastFace::Normal));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(card).controller, p);
    }
}
