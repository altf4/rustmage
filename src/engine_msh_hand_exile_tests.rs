//! Paid scenarios; direct movement/turn-history probes are explicitly bounded fixtures.
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
const HEX: &str = "Hex Magic";
const MOON: &str = "Moonstone, Harsh Mistress";

fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..600 {
        if stop(g) || g.outcome().is_some() {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::ChooseOptional { accept: false }
                )
            })
            .unwrap_or_else(|| panic!("unexpected hand-exile choice {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("hand-exile continuation stalled");
}
fn settle(g: &mut Game) {
    until(g, quiet);
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p && matches!(g.state.step, Step::Main1 | Step::Main2) && quiet(g)
    });
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game, packet: u64, discard: Option<CardId>) {
    for _ in 0..30 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseGift { beneficiary: None }
                | Action::FinishTargets
                | Action::PayCosts { .. } => true,
                Action::ChooseAdditionalCost { object } => discard == Some(object.card),
                _ => false,
            })
            .unwrap_or_else(|| panic!("unpaid hand-exile packet {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("hand-exile packet stayed unpaid");
}
fn cast_existing(
    g: &mut Game,
    p: Player,
    card: CardId,
    targets: &[Target],
    discard: Option<CardId>,
) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
    let (colored, generic) = g.object_view(card).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    let action = if g.object_view(card).zone == Zone::Exile {
        Action::AnnounceExile { card }
    } else {
        Action::AnnounceCast { card }
    };
    checked(g, action);
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        for _ in 0..4 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    finish(g, packet, discard);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let card = named(g, p, name, Zone::Hand);
    cast_existing(g, p, card, targets, None);
    card
}
fn moon(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, MOON, &[]);
    settle(g);
    id
}
fn duress(g: &mut Game, victim: Player, card: CardId) {
    let q = victim.other();
    main(g, q);
    paid(g, q, "Duress", &[Target::Player(victim)]);
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseRevealedDiscard {
            object: g.object_ref(card),
        })
    });
    let object = g.object_ref(card);
    checked(g, Action::ChooseRevealedDiscard { object });
}
fn optional(g: &mut Game, p: Player) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    assert!(matches!(g.state.pending, Pending::EffectOptional { player, .. } if player==p));
    assert_eq!(g.actor(), Some(p));
}
fn permission(g: &Game, p: Player, card: CardId) -> bool {
    g.state
        .exile_permissions
        .iter()
        .any(|r| r.object == g.object_ref(card) && r.player == p)
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn reject(g: &mut Game, action: Action) {
    let (snapshot, obs, revision, bytes) = (g.snapshot(), views(g), g.revision(), g.state_bytes());
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(views(g), obs);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn orphan(g: Game, action: Action) -> Game {
    let p = g.actor().unwrap();
    let revision = g.revision();
    let snapshot = g.snapshot();
    let obs = views(&g);
    let at = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, at).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&snapshot).unwrap();
    assert_eq!(views(&raw), obs);
    assert_eq!(raw.apply_index(p, revision, at), Err(Error::StaleRevision));
    raw.apply(p, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

#[test]
fn msh_hand_exile_hex_empty_single_many_are_paid_exile_not_discard() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 4] {
            let mut g = setup(p);
            moon(&mut g, p);
            let ids: Vec<_> = (0..count)
                .map(|_| named(&mut g, p, "Bear Cub", Zone::Hand))
                .collect();
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let source = paid(&mut g, p, HEX, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), count);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                drawn + count as u64
            );
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
            for id in ids {
                assert_eq!(g.object_view(id).zone, Zone::Exile);
                assert!(permission(&g, p, id));
            }
            assert!(
                g.state
                    .exile_permissions
                    .iter()
                    .all(|r| r.expiry == ExilePlayExpiry::EndOfNextTurn
                        && !r.spend_any_mana
                        && !r.exile_on_graveyard)
            );
        }
    }
}

#[test]
fn msh_hand_exile_hex_reads_resolution_hand_after_real_vision_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "The Vision", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 2 })
        });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(
            g.state.exile_permissions.len(),
            2,
            "Vision draws into hand before Hex resolves"
        );
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}

#[test]
fn msh_hand_exile_hex_public_exile_private_new_hand_and_owned_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        checked(&mut g, Action::Pass);
        g = orphan(g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == bear && o.zone == Zone::Exile)
        );
        let hidden = g
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Hand && o.owner == p)
            .unwrap() as CardId;
        let mut other = g.clone();
        other.state.objects[hidden as usize].card = Card::from_name("Island").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(other.observe(p)).unwrap()
        );
    }
}

#[test]
fn msh_hand_exile_hex_paid_exiled_spell_and_normal_land_allowance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        settle(&mut g);
        main(&mut g, p);
        cast_existing(&mut g, p, bear, &[], None);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        main(&mut g, p);
        checked(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        main(&mut g, p);
        let extra = named(&mut g, p, "Forest", Zone::Hand);
        reject(&mut g, Action::PlayLand { card: extra });
        assert!(g.state.land_played);
        assert!(!g.can_play_exiled(p, bear));
    }
}

#[test]
fn msh_hand_exile_hex_permission_does_not_ignore_colors_or_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        settle(&mut g);
        main(&mut g, p);
        g.state.players[p.index()].mana = [3, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceExile { card: murder });
        main(&mut g, p.other());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        reject(&mut g, Action::AnnounceExile { card: bear });
        assert!(permission(&g, p, bear));
    }
}

#[test]
fn msh_hand_exile_hex_deadline_survives_source_movement_until_end_next_own_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let source = paid(&mut g, p, HEX, &[]);
        settle(&mut g);
        g.move_object(source, Zone::Exile);
        main(&mut g, p.other());
        assert!(g.can_play_exiled(p, bear));
        main(&mut g, p);
        assert!(g.can_play_exiled(p, bear));
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::End && quiet(g)
        });
        assert!(g.can_play_exiled(p, bear));
        main(&mut g, p.other());
        assert!(!g.can_play_exiled(p, bear));
    }
}

#[test]
fn msh_hand_exile_hex_old_permission_does_not_follow_leave_and_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = named(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        settle(&mut g);
        let old = g.object_ref(card);
        g.move_object(card, Zone::Graveyard);
        g.move_object(card, Zone::Exile);
        assert_ne!(g.object_ref(card), old);
        assert!(!g.can_play_exiled(p, card));
        main(&mut g, p);
        reject(&mut g, Action::AnnounceExile { card });
    }
}

#[test]
fn msh_hand_exile_hex_empty_library_zero_draw_or_actual_loss() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = setup(p);
            let library = g.state.players[p.index()].library.clone();
            g.move_objects_raw(&library, Zone::Exile);
            for _ in 0..count {
                named(&mut g, p, "Forest", Zone::Hand);
            }
            paid(&mut g, p, HEX, &[]);
            settle(&mut g);
            if count == 0 {
                assert!(g.outcome().is_none());
            } else {
                assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
            }
        }
    }
}

#[test]
fn msh_hand_exile_hex_late_draw_failure_rolls_back_exile_permissions_and_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = named(&mut g, p, "Forest", Zone::Hand);
        paid(&mut g, p, HEX, &[]);
        g.state.players[p.index()].cards_drawn_this_turn = u64::MAX;
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let obs = views(&g);
        let rev = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(views(&g), obs);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        assert!(!permission(&g, p, card));
    }
}

#[test]
fn msh_hand_exile_moon_paid_body_and_actual_flying_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = moon(&mut g, p);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 4));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let turn = g.state.turn;
        until(&mut g, |g| {
            g.state.turn > turn && g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn msh_hand_exile_moon_real_duress_accept_or_decline_exact_discard() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            moon(&mut g, p);
            let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
            duress(&mut g, p, card);
            optional(&mut g, p);
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                if accept { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(permission(&g, p, card), accept);
            assert!(!permission(&g, p.other(), card));
        }
    }
}

#[test]
fn msh_hand_exile_moon_real_discard_cost_queues_before_spell_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let card = named(&mut g, p, "Forest", Zone::Hand);
        let spell = named(&mut g, p, "Thrill of Possibility", Zone::Hand);
        cast_existing(&mut g, p, spell, &[], Some(card));
        optional(&mut g, p);
        assert!(g.hand_count(p) == 0, "Thrill has not drawn yet");
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert!(permission(&g, p, card));
    }
}

#[test]
fn msh_hand_exile_moon_actual_simultaneous_discard_has_one_choice_per_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let ids: Vec<_> = ["Forest", "Bear Cub", "Murder"]
            .iter()
            .map(|name| named(&mut g, p, name, Zone::Hand))
            .collect();
        paid(&mut g, p, "Peer Past the Veil", &[]);
        for &card in &ids {
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::Discard { card })
            });
            checked(&mut g, Action::Discard { card });
        }
        for accept in [true, false, true] {
            optional(&mut g, p);
            checked(&mut g, Action::ChooseOptional { accept });
        }
        settle(&mut g);
        assert_eq!(
            ids.iter()
                .filter(|&&id| g.object_view(id).zone == Zone::Exile)
                .count(),
            2
        );
        assert_eq!(
            ids.iter()
                .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                .count(),
            1
        );
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 3);
    }
}

#[test]
fn msh_hand_exile_moon_only_controller_discards_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let foreign = named(&mut g, p.other(), "Arc Reactor", Zone::Hand);
        paid(&mut g, p, "Duress", &[Target::Player(p.other())]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseRevealedDiscard {
                object: g.object_ref(foreign),
            })
        });
        let object = g.object_ref(foreign);
        checked(&mut g, Action::ChooseRevealedDiscard { object });
        settle(&mut g);
        assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        assert!(g.state.exile_permissions.is_empty());
    }
}

#[test]
fn msh_hand_exile_moon_replaced_discard_is_not_from_own_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        named(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
        duress(&mut g, p, card);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert!(!permission(&g, p, card));
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseOptional { accept: true })
        );
    }
}

#[test]
fn msh_hand_exile_moon_stale_discard_reference_does_not_rebind_returned_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
        duress(&mut g, p, card);
        let old = g.object_ref(card);
        g.move_object(card, Zone::Exile);
        g.move_object(card, Zone::Graveyard);
        assert_ne!(g.object_ref(card), old);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        assert!(!permission(&g, p, card));
    }
}

#[test]
fn msh_hand_exile_moon_captured_controller_survives_source_death_or_theft() {
    for p in [Player::P0, Player::P1] {
        for die in [false, true] {
            let mut g = setup(p);
            let source = moon(&mut g, p);
            let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
            duress(&mut g, p, card);
            if die {
                paid(&mut g, p.other(), "Murder", &[Target::Card(source)]);
                until(&mut g, |g| g.object_view(source).zone == Zone::Graveyard);
            } else {
                g.change_controller(g.object_ref(source), p.other());
            }
            optional(&mut g, p);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert!(permission(&g, p, card));
            assert!(!permission(&g, p.other(), card));
        }
    }
}

#[test]
fn msh_hand_exile_moon_owned_optional_commit_and_unrelated_actions_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
        let unrelated = named(&mut g, p, "Forest", Zone::Graveyard);
        duress(&mut g, p, card);
        optional(&mut g, p);
        reject(&mut g, Action::ChooseSearch { card: unrelated });
        g = orphan(g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert!(permission(&g, p, card));
        assert_eq!(g.object_view(unrelated).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_hand_exile_moon_opponent_turn_permission_lasts_through_next_own_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let card = named(&mut g, p, "Arc Reactor", Zone::Hand);
        duress(&mut g, p, card);
        optional(&mut g, p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.can_play_exiled(p, card));
        main(&mut g, p);
        assert!(g.can_play_exiled(p, card));
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::End && quiet(g)
        });
        assert!(g.can_play_exiled(p, card));
        main(&mut g, p.other());
        assert!(!g.can_play_exiled(p, card));
    }
}

#[test]
fn msh_hand_exile_moon_paid_exiled_spell_never_gets_mana_substitution_or_free_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        moon(&mut g, p);
        let card = named(&mut g, p, "Murder", Zone::Hand);
        duress(&mut g, p, card);
        optional(&mut g, p);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        main(&mut g, p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        reject(&mut g, Action::AnnounceExile { card });
        cast_existing(&mut g, p, card, &[Target::Card(target)], None);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_hand_exile_hex_draw_pause_finishes_before_permission_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let cards = [
            named(&mut g, p, "Bear Cub", Zone::Hand),
            named(&mut g, p, "Forest", Zone::Hand),
        ];
        paid(&mut g, p, HEX, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.hand_count(p), 1, "only the first draw has happened");
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(
            g.state.exile_permissions.is_empty(),
            "the grant follows all draws"
        );
        for card in cards {
            assert_eq!(g.object_view(card).zone, Zone::Exile);
        }
        assert!(g.observe(p.other()).miracle.is_none());
        assert!(!g.observe(p).miracle.unwrap().can_reveal);
        g = orphan(g, Action::DeclineMiracle);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        for card in cards {
            assert!(permission(&g, p, card));
        }
    }
}
