use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn settle(g: &mut Game, accept: bool) {
    for _ in 0..200 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::PutTrigger { .. } | Action::Pass | Action::ArrangeLibrary { .. } => true,
                Action::Discard { .. } => accept,
                Action::ChooseOptional { accept: a } => *a == accept,
                Action::DeclineEffectPayment => !accept,
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => g.object_view(*id).owner != g.actor().unwrap(),
                _ => false,
            })
            .expect("explicit fixture choice");
        replay(g, a);
    }
    panic!("did not settle");
}
fn event(g: &mut Game, id: CardId, p: Player, kind: StackKind) {
    let card = named(g, p, "Giant Growth", Zone::Hand);
    let mut item = g.cast_item(p, card, [0; 6]);
    item.kind = kind;
    g.target_becomes_bound(&item, g.bind_target(Target::Card(id)));
}
#[test]
fn opponent_target_draws_before_spell_and_survives_source_departure() {
    for name in ["Old Fat Spider", "Ruric Thar, Biomagus"] {
        let mut g = scenario();
        let source = named(&mut g, Player::P1, name, Zone::Battlefield);
        let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
        g.state.players[0].mana = [10; 6];
        cast(&mut g, shock, Some(Target::Card(source)));
        assert_eq!(g.state.stack.len(), 2);
        assert_eq!(g.state.stack.last().unwrap().card, source);
        assert!(g.state.stack.last().unwrap().related_stack.is_none());
        assert_eq!(g.object_view(source).damage, 0);
        g.move_object(source, Zone::Graveyard);
        let drawn = g.state.players[1].cards_drawn_this_turn;
        settle(&mut g, true);
        assert_eq!(g.state.players[1].cards_drawn_this_turn, drawn + 1);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        // Own targets never trigger this opponent-only body.
        g.move_object(source, Zone::Battlefield);
        event(&mut g, source, Player::P1, StackKind::Spell);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn target_events_deduplicate_groups_but_repeat_for_new_spells_and_all_ability_kinds() {
    let mut g = scenario();
    let a = named(&mut g, Player::P1, "Old Fat Spider", Zone::Battlefield);
    for kind in [
        StackKind::Spell,
        StackKind::ActivatedAbility { ability: 0 },
        StackKind::TriggeredAbility { ability: 0 },
    ] {
        let card = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
        let mut item = g.cast_item(Player::P0, card, [0; 6]);
        item.kind = kind;
        let bound = g.bind_target(Target::Card(a));
        g.target_becomes_bound(&item, bound);
        item.targets.push(bound);
        let mut other = bound;
        other.group = 1;
        g.target_becomes_bound(&item, other);
        assert_eq!(g.state.pending_triggers.len(), 1);
        settle(&mut g, true);
    }
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 3);
    g.move_object(a, Zone::Hand);
    event(&mut g, a, Player::P0, StackKind::Spell);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn valiant_ignores_opponent_targets_and_consumes_first_event_even_if_trigger_is_countered() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Seedglaive Mentor", Zone::Battlefield);
    event(&mut g, a, Player::P1, StackKind::Spell);
    assert!(g.state.triggered_this_turn.is_empty());
    event(
        &mut g,
        a,
        Player::P0,
        StackKind::ActivatedAbility { ability: 0 },
    );
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.state.triggered_this_turn.len(), 1);
    // Removing the queued trigger models countering it; this cannot restore the first-event opportunity.
    g.state.pending_triggers.clear();
    event(&mut g, a, Player::P0, StackKind::Spell);
    assert!(g.state.pending_triggers.is_empty());
    g.begin_turn();
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    event(
        &mut g,
        a,
        Player::P0,
        StackKind::TriggeredAbility { ability: 0 },
    );
    settle(&mut g, true);
    assert_eq!(g.object_view(a).counters.plus_one, 1);
    g.move_object(a, Zone::Hand);
    g.move_object(a, Zone::Battlefield);
    event(&mut g, a, Player::P0, StackKind::Spell);
    settle(&mut g, true);
    assert_eq!(g.object_view(a).counters.plus_one, 1); // old counters left with the old incarnation
}
#[test]
fn valiant_queues_at_target_declaration_before_payment_and_rejects_forged_choices() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Nettle Guard", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    replay(&mut g, Action::AnnounceCast { card: spell });
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(
            Player::P0,
            Action::ChooseTarget {
                target: Target::Player(Player::P1)
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    assert_eq!(g.observe(Player::P0).decision, "pay_costs");
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.stats(a).toughness, 1);
    let land = g
        .legal_actions()
        .into_iter()
        .find(|x| matches!(x, Action::TapMana { .. }))
        .unwrap();
    replay(&mut g, land);
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|x| matches!(x, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    settle(&mut g, true);
    assert_eq!(
        g.stats(a),
        Stats {
            power: 6,
            toughness: 6
        }
    );
    g.finish_cleanup();
    assert_eq!(
        g.stats(a),
        Stats {
            power: 3,
            toughness: 1
        }
    );
}
#[test]
fn mouse_trapper_targets_only_opposing_creatures_and_no_target_still_consumes_first_event() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Mouse Trapper", Zone::Battlefield);
    assert!(g.keywords(a).contains(Keywords::FLASH));
    event(&mut g, a, Player::P0, StackKind::Spell);
    settle(&mut g, true);
    assert_eq!(g.state.triggered_this_turn.len(), 1);
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    event(&mut g, a, Player::P0, StackKind::Spell);
    assert!(g.state.pending_triggers.is_empty());
    g.begin_turn();
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    event(&mut g, a, Player::P0, StackKind::Spell);
    g.prepare_triggers();
    while let Some(x) = g
        .legal_actions()
        .into_iter()
        .find(|x| matches!(x, Action::PutTrigger { .. }))
    {
        replay(&mut g, x);
    }
    let actions = g.legal_actions();
    assert!(actions.contains(&Action::ChooseTarget {
        target: Target::Card(enemy)
    }));
    assert!(!actions.contains(&Action::ChooseTarget {
        target: Target::Card(a)
    }));
    settle(&mut g, true);
    assert!(g.object_view(enemy).tapped);
}
#[test]
fn cactarantula_discount_checks_own_desert_and_draw_is_optional() {
    for desert in [None, Some(Player::P0), Some(Player::P1)] {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, "Cactarantula", Zone::Hand);
        if let Some(p) = desert {
            named(&mut g, p, "Bristling Backwoods", Zone::Battlefield);
        }
        g.state.players[0].mana = [0, 0, 0, 0, 5, 0];
        let payable = g
            .legal_actions()
            .iter()
            .any(|x| matches!(x,Action::Cast{card,..}if *card==a));
        assert_eq!(payable, desert == Some(Player::P0));
        g.state.players[0].mana = [10; 6];
        cast(&mut g, a, None);
        settle(&mut g, true);
        assert!(g.keywords(a).contains(Keywords::REACH));
        event(&mut g, a, Player::P1, StackKind::Spell);
        settle(&mut g, false);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        event(&mut g, a, Player::P1, StackKind::Spell);
        settle(&mut g, true);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    }
}
#[test]
fn veteran_growth_first_strike_and_private_scry_resume_then_expire() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Veteran Guardmouse", Zone::Battlefield);
    event(&mut g, a, Player::P0, StackKind::Spell);
    g.prepare_triggers();
    for _ in 0..10 {
        if g.legal_actions()
            .iter()
            .any(|x| matches!(x, Action::ArrangeLibrary { .. }))
        {
            break;
        }
        let x = g
            .legal_actions()
            .into_iter()
            .find(|x| matches!(x, Action::PutTrigger { .. } | Action::Pass))
            .unwrap();
        replay(&mut g, x);
    }
    assert!(g.keywords(a).contains(Keywords::FIRST_STRIKE));
    assert_eq!(g.stats(a).power, 4);
    assert!(g.observe(Player::P1).actions.is_empty());
    settle(&mut g, true);
    g.finish_cleanup();
    assert_eq!(g.stats(a).power, 3);
    assert!(!g.keywords(a).contains(Keywords::FIRST_STRIKE));
}
#[test]
fn scribe_optional_discard_is_private_and_declining_does_not_rearm_valiant() {
    for accept in [false, true] {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, "Whiskerquill Scribe", Zone::Battlefield);
        let hand = named(&mut g, Player::P0, "Mountain", Zone::Hand);
        event(&mut g, a, Player::P0, StackKind::Spell);
        g.prepare_triggers();
        for _ in 0..10 {
            if g.observe(Player::P0).decision == "effect_discard" {
                break;
            }
            let actions = g.legal_actions();
            if actions
                .iter()
                .any(|a| matches!(a, Action::Discard { .. } | Action::DeclineEffectPayment))
            {
                break;
            }
            let x = actions
                .into_iter()
                .find(|x| matches!(x, Action::PutTrigger { .. } | Action::Pass))
                .unwrap();
            replay(&mut g, x);
        }
        assert!(g.observe(Player::P1).actions.is_empty());
        settle(&mut g, accept);
        assert_eq!(
            g.object_view(hand).zone,
            if accept { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(g.state.players[0].cards_drawn_this_turn, u64::from(accept));
        event(&mut g, a, Player::P0, StackKind::Spell);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn ruric_double_prowess_and_nettle_sacrifice_are_complete_other_card_bodies() {
    let mut g = scenario();
    let ruric = named(
        &mut g,
        Player::P0,
        "Ruric Thar, Biomagus",
        Zone::Battlefield,
    );
    let guard = named(&mut g, Player::P0, "Nettle Guard", Zone::Battlefield);
    let artifact = named(&mut g, Player::P1, "Gold Pan", Zone::Battlefield);
    let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    assert_eq!(
        g.state.stack.len() + g.state.pending_triggers.len() + g.state.ordering_triggers.len(),
        3
    );
    settle(&mut g, true);
    assert_eq!(
        g.stats(ruric),
        Stats {
            power: 6,
            toughness: 8
        }
    );
    let x=g.legal_actions().into_iter().find(|x|matches!(x,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==guard&&*t==artifact)).unwrap();
    replay(&mut g, x);
    assert_eq!(g.object_view(guard).zone, Zone::Graveyard);
    settle(&mut g, true);
    assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
}
