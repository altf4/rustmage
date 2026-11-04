//! Actual Cori-Steel Cutter wiring, paid actions and exact created-token choices.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        assert_eq!(
            g.state.pending,
            Pending::Priority,
            "explicit choice required"
        );
        act(g, Action::Pass);
    }
    panic!("40-action Cori scenario budget exhausted");
}
fn monks(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/tdm-monk-prowess").unwrap())
            .then_some(id as CardId)
        })
        .collect()
}
fn shock(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    let id = put(g, p, "Shock", Zone::Hand);
    cast(g, id, Some(Target::Player(p.other())));
}
fn second_spell(g: &mut Game, p: Player) {
    shock(g, p);
    settle(g);
    assert!(monks(g, p).is_empty());
    shock(g, p);
    until(g, |g| {
        matches!(g.state.pending, Pending::CreatedToken { .. })
    });
}
fn decline(g: &mut Game) {
    act(g, Action::ChooseOptional { accept: false });
    settle(g);
}
fn equip(g: &mut Game, equipment: CardId, recipient: CardId) {
    choose(g, |a| {
        matches!(a, Action::Activate { card, ability: 0, target: Some(Target::Card(t)), .. }
        if *card == equipment && *t == recipient)
    });
}

#[test]
fn cutter_is_playable_in_freeform_but_stays_banned_in_standard() {
    let cutter = Card::from_name("Cori-Steel Cutter").unwrap();
    assert!(cutter.is_supported());
    let mut deck = vec![Card::Forest; 56];
    deck.extend([cutter; 4]);
    let config = Config {
        decks: [deck.clone(), deck],
        seed: 47,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    };
    assert!(matches!(
        Game::new(config.clone()),
        Err(Error::InvalidDeck(_))
    ));
    assert!(Game::new(Config {
        deck_rules: DeckRules::Freeform,
        ..config
    })
    .is_ok());
}

#[test]
fn paid_cutter_counts_as_first_spell_then_monk_has_printed_traits_and_prowess_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cutter = put(&mut g, p, "Cori-Steel Cutter", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        act(&mut g, Action::AnnounceCast { card: cutter });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(cutter).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        g.state.players[p.index()].mana = [8; 6];
        shock(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::CreatedToken { .. })
        });
        let token = monks(&g, p)[0];
        assert_eq!(stats(&g, token), (1, 1));
        assert_eq!(g.object_view(token).colors, 1);
        assert!(g.object_view(token).card.has_subtype("Monk"));
        assert_eq!(
            g.state.players[p.other().index()].life,
            20,
            "creation resolves above the triggering spell"
        );
        let object = g.object_ref(token);
        act(&mut g, Action::ChooseCreatedToken { object });
        settle(&mut g);
        assert_eq!(g.object_view(cutter).attached_to, Some(object));
        assert_eq!(stats(&g, token), (2, 2));
        assert!(g
            .object_view(token)
            .keywords
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE)));
        shock(&mut g, p);
        settle(&mut g);
        assert_eq!(monks(&g, p).len(), 1, "third spell has no flurry trigger");
        assert_eq!(stats(&g, token), (3, 3), "one prowess trigger");
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        settle(&mut g);
        assert_eq!(
            stats(&g, token),
            (3, 3),
            "creature spells do not trigger prowess"
        );
        let turn = g.state.turn;
        g.state.step = Step::End;
        until(&mut g, |g| g.state.turn > turn);
        assert_eq!(stats(&g, token), (2, 2), "prowess ends at cleanup");
    }
}

#[test]
fn paid_equip_moves_bonuses_and_obeys_controller_timing_payment_and_incarnation() {
    let p = Player::P0;
    let mut g = scenario(p);
    let cutter = put(&mut g, p, "Cori-Steel Cutter", Zone::Battlefield);
    let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let announce = Action::AnnounceActivate {
        card: cutter,
        ability: 0,
        counter_payment: Counters::default(),
    };
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut g, announce.clone());
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(&mut g, announce.clone());
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    act(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 1],
        },
    );
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(&g, first), (3, 3));
    assert!(g
        .object_view(first)
        .keywords
        .contains(Keywords::TRAMPLE.union(Keywords::HASTE)));
    g.state.players[p.index()].mana = [8; 6];
    equip(&mut g, cutter, second);
    reject(&mut g, announce.clone());
    let old = g.object_ref(second);
    g.move_object(second, Zone::Exile);
    g.move_object(second, Zone::Battlefield);
    assert_ne!(g.object_ref(second), old);
    settle(&mut g);
    assert_eq!(g.object_view(cutter).attached_to, Some(g.object_ref(first)));
    equip(&mut g, cutter, second);
    settle(&mut g);
    assert_eq!(stats(&g, first), (2, 2));
    assert!(!g.object_view(first).keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&g, second), (3, 3));
    g.state.step = Step::BeginCombat;
    reject(&mut g, announce);
    g.move_object(cutter, Zone::Graveyard);
    assert_eq!(stats(&g, second), (2, 2));
    assert!(!g.object_view(second).keywords.contains(Keywords::TRAMPLE));
}

#[test]
fn flurry_can_trigger_during_opponents_turn_and_ignores_opponents_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        put(&mut g, p, "Cori-Steel Cutter", Zone::Battlefield);
        shock(&mut g, p.other());
        settle(&mut g);
        shock(&mut g, p.other());
        settle(&mut g);
        assert!(monks(&g, p).is_empty());
        act(&mut g, Action::Pass);
        second_spell(&mut g, p);
        decline(&mut g);
        assert_eq!(monks(&g, p).len(), 1);
        assert_eq!(g.state.active, p.other());
    }
}

#[test]
fn doubled_creation_accepts_exactly_one_token_or_declines_without_moving_equipment() {
    for attach in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
        g.add_counter(elspeth, CounterKind::Loyalty, 5);
        let cutter = put(&mut g, p, "Cori-Steel Cutter", Zone::Battlefield);
        let prior = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        equip(&mut g, cutter, prior);
        settle(&mut g);
        second_spell(&mut g, p);
        let created = monks(&g, p);
        assert_eq!(created.len(), 2);
        for seat in [p, p.other()] {
            assert_eq!(
                g.observe(seat)
                    .created_token_choice
                    .as_ref()
                    .unwrap()
                    .candidates
                    .len(),
                2
            );
        }
        for &token in &created {
            assert_eq!(stats(&g, token), (1, 1));
        }
        let noncreated = g.object_ref(prior);
        reject(&mut g, Action::ChooseCreatedToken { object: noncreated });
        if attach {
            let object = g.object_ref(created[1]);
            act(&mut g, Action::ChooseCreatedToken { object });
            settle(&mut g);
        } else {
            decline(&mut g);
        }
        assert_eq!(
            g.object_view(cutter).attached_to,
            Some(g.object_ref(if attach { created[1] } else { prior }))
        );
        assert_eq!(stats(&g, created[0]), (1, 1));
        assert_eq!(stats(&g, created[1]), if attach { (2, 2) } else { (1, 1) });
    }
}

#[test]
fn old_flurry_packet_creates_monk_after_source_reentry_without_attaching_new_source() {
    let p = Player::P0;
    let mut g = scenario(p);
    let cutter = put(&mut g, p, "Cori-Steel Cutter", Zone::Battlefield);
    shock(&mut g, p);
    settle(&mut g);
    shock(&mut g, p);
    let old = g.object_ref(cutter);
    act(&mut g, Action::Pass);
    let removal = put(&mut g, p.other(), "Disenchant", Zone::Hand);
    cast(&mut g, removal, Some(Target::Card(cutter)));
    until(&mut g, |g| g.object_view(cutter).zone == Zone::Graveyard);
    g.move_object(cutter, Zone::Battlefield);
    assert_ne!(g.object_ref(cutter), old);
    settle(&mut g);
    assert_eq!(monks(&g, p).len(), 1);
    assert_eq!(g.object_view(cutter).attached_to, None);
    assert_eq!(stats(&g, monks(&g, p)[0]), (1, 1));
}

#[test]
fn actual_flurry_choice_owns_raw_index_children_and_restores_after_parent_disposal() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Cori-Steel Cutter", Zone::Battlefield);
    second_spell(&mut parent, p);
    let object = parent.object_ref(monks(&parent, p)[0]);
    let action = Action::ChooseCreatedToken { object };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut declined = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(source).attached_to, Some(object));
    decline(&mut declined);
    assert_eq!(declined.object_view(source).attached_to, None);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
