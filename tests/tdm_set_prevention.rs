//! Actual TDM chosen-source prevention, damage-controller and departed-source outcomes.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}

fn reach(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::FinishStackTargets
            )
        });
    }
    panic!(
        "96-action actual prevention interaction bound: {:?}",
        g.state.pending
    );
}

fn drain(g: &mut Game) {
    reach(g, idle);
}

fn priority(g: &mut Game, p: Player) {
    assert_eq!(g.state.pending, Pending::Priority);
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}

fn waiting_trigger(g: &Game, source: ObjectRef) -> bool {
    g.state.pending == Pending::Priority
        && g.state.resolving.is_none()
        && g.state.stack.iter().any(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source.is_some_and(|s| s.object == source)
        })
}

fn source_choice(g: &mut Game, p: Player, spell: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [1, 1, 0, 1, 0, 2];
    cast(g, spell, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    reach(g, |g| {
        matches!(g.state.pending, Pending::DamageSource { .. })
    });
    assert_eq!(g.actor(), Some(p));
    assert!(g.observe(p).damage_source_choice.is_some());
    assert!(g.observe(p.other()).damage_source_choice.is_none());
}

fn finish_choice(g: &mut Game, p: Player, source: ObjectRef) {
    act(g, Action::ChooseDamageSource { source });
    reach(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.resolving.is_none()
            && g.observe(p)
                .chosen_source_shields
                .as_ref()
                .is_some_and(|shields| {
                    shields.len() == 1 && shields[0].chosen == DamageSourceRef::Object(source)
                })
    });
}

fn same_views(a: &Game, b: &Game, restored: bool) {
    assert_eq!(a.snapshot(), b.snapshot());
    for viewer in [Player::P0, Player::P1] {
        let left = a.observe(viewer);
        let mut right = b.observe(viewer);
        if restored {
            // Restore advances the external revision epoch. Only that top-level
            // epoch differs; every other public and private observation field is exact.
            right.revision = left.revision;
        }
        assert_eq!(
            serde_json::to_value(left).unwrap(),
            serde_json::to_value(right).unwrap()
        );
    }
}

fn owned_source_choice(parent: Game, source: ObjectRef, wrong_incarnation: ObjectRef) -> Game {
    let actor = parent.actor().unwrap();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let action = Action::ChooseDamageSource { source };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    assert_eq!(
        raw.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    reject(
        &mut raw,
        Action::ChooseDamageSource {
            source: wrong_incarnation,
        },
    );
    act(&mut raw, action.clone());
    assert_eq!(
        parent.snapshot(),
        saved,
        "child choice leaves parent unchanged"
    );
    drop(parent);
    indexed.apply_index(actor, revision, index).unwrap();
    same_views(&raw, &indexed, false);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.snapshot(), saved);
    act(&mut indexed, action);
    same_views(&raw, &indexed, true);
    raw
}

fn exile_stalwart(g: &mut Game, p: Player, severance: CardId, source: CardId, old: ObjectRef) {
    priority(g, p);
    g.state.players[p.index()].mana = [1, 0, 1, 0, 1, 0];
    act(g, Action::AnnounceCast { card: severance });
    assert!(matches!(g.state.pending, Pending::SpellCost { .. }));
    choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    reach(g, |g| {
        g.object_view(source).zone == Zone::Exile
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
    });
    assert_ne!(g.object_ref(source), old);
    assert!(
        waiting_trigger(g, old),
        "actual Stalwart trigger survives source exile"
    );
    assert_eq!(g.object_view(severance).zone, Zone::Graveyard);
}

#[test]
fn f15_stalwart_second_cast_damage_is_prevented_reflected_and_drawn_after_actual_severance() {
    for p in [Player::P0, Player::P1] {
        for choose_after_exile in [false, true] {
            let opponent = p.other();
            let mut g = scenario(opponent);
            let stalwart = put(&mut g, opponent, "Cori Mountain Stalwart", Zone::Hand);
            g.state.players[opponent.index()].mana = [1, 0, 0, 1, 0, 1];
            cast(&mut g, stalwart, None);
            drain(&mut g);
            assert_eq!(g.state.players[opponent.index()].mana, [0; 6]);
            assert_eq!(g.state.players[opponent.index()].spells_cast_this_turn, 1);
            let old = g.object_ref(stalwart);
            let second = put(&mut g, opponent, "Krumar Initiate", Zone::Hand);
            let forward = put(&mut g, p, "New Way Forward", Zone::Hand);
            let severance = put(&mut g, p, "Kin-Tree Severance", Zone::Hand);
            let library = g.library_card_count(p);
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            g.state.players[opponent.index()].mana = [0, 0, 1, 0, 0, 1];
            cast(&mut g, second, None);
            assert_eq!(g.state.players[opponent.index()].mana, [0; 6]);
            reach(&mut g, |g| waiting_trigger(g, old));
            assert_eq!(g.state.players[opponent.index()].spells_cast_this_turn, 2);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[opponent.index()].life, 20);

            if choose_after_exile {
                exile_stalwart(&mut g, p, severance, stalwart, old);
            }
            source_choice(&mut g, p, forward);
            let choice = g.observe(p).damage_source_choice.unwrap();
            assert!(choice.candidates.contains(&old));
            if choose_after_exile {
                let exiled = g.object_ref(stalwart);
                assert!(!choice.candidates.contains(&exiled));
                reject(&mut g, Action::ChooseDamageSource { source: exiled });
                if p == Player::P0 {
                    g = owned_source_choice(g, old, exiled);
                    reach(&mut g, |g| {
                        g.state.pending == Pending::Priority && g.state.resolving.is_none()
                    });
                } else {
                    finish_choice(&mut g, p, old);
                }
            } else {
                finish_choice(&mut g, p, old);
                exile_stalwart(&mut g, p, severance, stalwart, old);
            }
            assert_eq!(g.object_view(forward).zone, Zone::Graveyard);
            assert!(waiting_trigger(&g, old));
            assert_eq!(
                g.observe(p).chosen_source_shields.unwrap()[0].chosen,
                DamageSourceRef::Object(old)
            );
            let forward_card = Card::from_name("New Way Forward").unwrap();
            reach(&mut g, |g| {
                g.state.pending == Pending::Priority
                    && g.state.resolving.is_none()
                    && g.state.stack.iter().any(|item| {
                        matches!(item.kind, StackKind::TriggeredAbility { .. })
                            && item.source.is_some_and(|s| s.card == forward_card)
                    })
            });
            assert_eq!(
                g.state.players[p.index()].life,
                20,
                "the actual two damage was prevented"
            );
            assert_eq!(
                g.state.players[opponent.index()].life,
                22,
                "Stalwart gains two before the reflexive damage resolves"
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn);
            assert!(g.observe(p).chosen_source_shields.is_none());
            let callback = g.state.stack.last().unwrap();
            assert!(
                callback.targets.is_empty(),
                "reflected damage uses the source's controller without targeting"
            );
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(
                g.state.players[opponent.index()].life,
                20,
                "two reflected damage offsets the printed gain of two"
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 2);
            assert_eq!(g.library_card_count(p), library - 2);
            assert_eq!(
                g.hand_count(p),
                2,
                "both actual instants were spent and two cards drawn"
            );
            assert_eq!(g.object_view(stalwart).zone, Zone::Exile);
            assert_eq!(g.object_view(second).zone, Zone::Battlefield);
            assert_eq!(stats(&g, second), (2, 2));
        }
    }
}

fn activate_headliner(g: &mut Game, owner: Player, source: CardId, target: CardId) {
    priority(g, owner);
    g.state.players[owner.index()].mana = [0, 0, 0, 1, 0, 1];
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Player(owner.other()),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    assert_eq!(g.state.players[owner.index()].mana, [0; 6]);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert!(
        g.state
            .stack
            .iter()
            .any(|item| matches!(item.kind, StackKind::ActivatedAbility { ability: 0 }))
    );
}

#[test]
fn f15_headliner_can_be_chosen_before_or_after_sacrifice_but_creature_damage_keeps_player_shield() {
    for p in [Player::P0, Player::P1] {
        for choose_after_sacrifice in [false, true] {
            let opponent = p.other();
            let mut g = scenario(opponent);
            let headliner = put(&mut g, opponent, "Stadium Headliner", Zone::Hand);
            g.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, headliner, None);
            drain(&mut g);
            assert_eq!(g.state.players[opponent.index()].mana, [0; 6]);
            let old = g.object_ref(headliner);
            put(&mut g, opponent, "Marang River Regent", Zone::Battlefield);
            let victim = put(&mut g, p, "Marang River Regent", Zone::Battlefield);
            assert_eq!(stats(&g, victim), (6, 7));
            let forward = put(&mut g, p, "New Way Forward", Zone::Hand);
            let library = g.library_card_count(p);
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            if choose_after_sacrifice {
                activate_headliner(&mut g, opponent, headliner, victim);
            }
            source_choice(&mut g, p, forward);
            let choice = g.observe(p).damage_source_choice.unwrap();
            assert!(choice.candidates.contains(&old));
            if choose_after_sacrifice {
                let graveyard = g.object_ref(headliner);
                assert_ne!(graveyard, old);
                assert!(!choice.candidates.contains(&graveyard));
                reject(&mut g, Action::ChooseDamageSource { source: graveyard });
            }
            finish_choice(&mut g, p, old);
            if !choose_after_sacrifice {
                activate_headliner(&mut g, opponent, headliner, victim);
            }
            drain(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(victim).damage,
                1,
                "the surviving creature counts after Headliner paid its sacrifice cost"
            );
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[opponent.index()].life, 20);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn);
            assert_eq!(g.library_card_count(p), library);
            assert_eq!(g.hand_count(p), 0);
            assert_eq!(g.object_view(forward).zone, Zone::Graveyard);
            assert_eq!(g.object_view(headliner).zone, Zone::Graveyard);
            let shields = g.observe(p).chosen_source_shields.unwrap();
            assert_eq!(
                shields.len(),
                1,
                "creature damage does not consume protection against damage to you"
            );
            assert_eq!(shields[0].controller, p);
            assert_eq!(shields[0].chosen, DamageSourceRef::Object(old));
        }
    }
}
#[test]
fn f15_reflexive_damage_uses_current_or_departure_controller_of_its_exact_source() {
    let attacker = Player::P0;
    let defender = Player::P1;
    let mut game = scenario(attacker);
    let bear = put(&mut game, defender, "Bear Cub", Zone::Battlefield);
    let original = game.object_ref(bear);
    let saga = put(&mut game, attacker, "The Super Hero Civil War", Zone::Hand);
    let forward = put(&mut game, defender, "New Way Forward", Zone::Hand);
    let disenchant = put(&mut game, defender, "Disenchant", Zone::Hand);
    let murder = put(&mut game, attacker, "Murder", Zone::Hand);
    let target = Action::ChooseTarget {
        target: Target::Card(bear),
    };
    cast(&mut game, saga, None);
    reach(&mut game, |game| game.legal_actions().contains(&target));
    act(&mut game, target);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(bear).controller, attacker);
    assert_eq!(game.object_ref(bear), original);

    source_choice(&mut game, defender, forward);
    finish_choice(&mut game, defender, original);
    // The fixture starts combat after the stolen creature can attack.
    game.state.objects[bear as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = attacker;
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |game| game.state.pending == Pending::Blockers);
    act(&mut game, Action::FinishBlockers);
    let on_stack = |game: &Game| {
        game.state.pending == Pending::Priority
            && game.state.stack.len() == 1
            && game.state.stack[0].source.unwrap().id == forward
    };
    reach(&mut game, on_stack);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.players[1].life, 20);
    let before_draw = game.state.players[defender.index()].cards_drawn_this_turn;

    for (changed, departed) in [(false, false), (true, false), (true, true)] {
        let mut branch = game.clone();
        if changed {
            priority(&mut branch, defender);
            branch.state.players[defender.index()].mana = [1, 0, 0, 0, 0, 1];
            cast(&mut branch, disenchant, Some(Target::Card(saga)));
            reach(&mut branch, on_stack);
            assert_eq!(branch.object_view(bear).controller, defender);
            assert_eq!(branch.object_ref(bear), original);
        }
        if departed {
            priority(&mut branch, attacker);
            branch.state.players[attacker.index()].mana = [0, 0, 2, 0, 0, 1];
            cast(&mut branch, murder, Some(Target::Card(bear)));
            reach(&mut branch, on_stack);
            assert_eq!(branch.object_view(bear).zone, Zone::Graveyard);
            assert_ne!(branch.object_ref(bear), original);
        }
        drain(&mut branch);
        assert_eq!(branch.state.players[0].life, if changed { 20 } else { 18 });
        assert_eq!(branch.state.players[1].life, if changed { 18 } else { 20 });
        assert_eq!(
            branch.state.players[defender.index()].cards_drawn_this_turn,
            before_draw + 2
        );
    }
}
