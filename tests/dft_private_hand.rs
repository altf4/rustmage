//! Look is private; choosing a card, rather than successful discard, causes the draw.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const LOOK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::LookTargetHandMayDiscardThenDraw],
    ),
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileAllCardsAndTokens),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn start(g: &mut Game, p: Player, target: Player) -> CardId {
    let source = put(g, p, "Divination", Zone::Hand);
    cast(g, source, Some(Target::Player(target)));
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return source;
        }
        act(g, Action::Pass);
    }
    panic!("private hand choice missing");
}
fn pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(
        g,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
}

#[test]
fn look_is_private_and_decline_keeps_every_card_and_causes_no_draw() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &LOOK)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hidden = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
        start(&mut g, p, p.other());
        let private = g.observe(p);
        let other = g.observe(p.other());
        let choice = private.hand_choice.as_ref().unwrap();
        assert!(choice.optional);
        let seen = choice.inspected_cards.as_ref().unwrap();
        assert!(
            seen.iter()
                .any(|o| o.id == hidden && o.card.name() == "Grizzly Bears")
        );
        assert!(
            seen.iter()
                .any(|o| o.id == land && o.card.name() == "Mountain")
        );
        assert!(other.hand_choice.is_none());
        assert!(
            !private.known_hand.contains(&g.object_ref(hidden)),
            "look never reveals publicly"
        );
        act(&mut g, Action::ChooseHandCard { object: None });
        settle(&mut g);
        assert_eq!(g.object_view(hidden).zone, Zone::Hand);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        assert!(g.observe(p).hand_choice.is_none());
    }
}

#[test]
fn choosing_any_card_including_land_discards_then_draws_for_the_targeted_player() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &LOOK)]);
    for (p, target) in [
        (Player::P0, Player::P1),
        (Player::P1, Player::P0),
        (Player::P0, Player::P0),
    ] {
        let mut g = scenario(p);
        let selected = put(&mut g, target, "Mountain", Zone::Hand);
        let untouched = put(&mut g, target, "Grizzly Bears", Zone::Hand);
        let source = start(&mut g, p, target);
        // The resolving source's departure does not change the captured chooser/victim.
        g.move_object(source, Zone::Exile);
        pick(&mut g, selected);
        settle(&mut g);
        assert_eq!(g.object_view(selected).zone, Zone::Graveyard);
        assert_eq!(g.object_view(untouched).zone, Zone::Hand);
        assert_eq!(g.state.players[target.index()].cards_drawn_this_turn, 1);
        if p != target {
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}

#[test]
fn empty_hand_still_allows_the_same_optional_private_phase_without_a_draw() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &LOOK)]);
    let p = Player::P1;
    let mut g = scenario(p);
    start(&mut g, p, p.other());
    assert!(
        g.observe(p)
            .hand_choice
            .as_ref()
            .unwrap()
            .inspected_cards
            .as_ref()
            .unwrap()
            .is_empty()
    );
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseHandCard { object: None })
    );
    act(&mut g, Action::ChooseHandCard { object: None });
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
}

#[test]
fn chosen_card_draw_occurs_after_discard_redirection_or_prevention() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[
        (Card::Divination, &LOOK),
        (Card::from_name("Short Bow").unwrap(), &EXILE),
    ]);
    for prevented in [false, true] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
        if !prevented {
            put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        }
        start(&mut g, Player::P0, Player::P1);
        if prevented {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(Some(g.object_ref(card))));
        }
        pick(&mut g, card);
        settle(&mut g);
        assert_eq!(
            g.object_view(card).zone,
            if prevented { Zone::Hand } else { Zone::Exile }
        );
        assert_eq!(
            g.state.players[1].cards_drawn_this_turn, 1,
            "the condition was choosing the card"
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
    }
}

#[test]
fn private_pending_choice_rejects_other_hand_and_is_owned_across_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &LOOK)]);
    let mut g = scenario(Player::P0);
    let chosen = put(&mut g, Player::P1, "Mountain", Zone::Hand);
    let wrong = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    start(&mut g, Player::P0, Player::P1);
    let wrong_ref = g.object_ref(wrong);
    reject(
        &mut g,
        Action::ChooseHandCard {
            object: Some(wrong_ref),
        },
    );
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let action = Action::ChooseHandCard {
        object: Some(g.object_ref(chosen)),
    };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let rev = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(Player::P0, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(g);
    act(&mut indexed, Action::ChooseHandCard { object: None });
    settle(&mut indexed);
    settle(&mut raw);
    assert_eq!(raw.object_view(chosen).zone, Zone::Graveyard);
    assert_eq!(indexed.object_view(chosen).zone, Zone::Hand);
    assert_eq!(raw.state.players[1].cards_drawn_this_turn, 1);
    assert_eq!(indexed.state.players[1].cards_drawn_this_turn, 0);
}

#[test]
fn draw_waits_until_a_replacement_entry_and_its_owned_choice_have_completed() {
    const ENTER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        entry: EntryTapped::PayLife(2),
        extras: &CardExtras {
            zone_replacement: Some(ZoneReplacementRule::OpponentDiscardToBattlefield),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::Divination, &LOOK), (Card::GrizzlyBears, &ENTER)]);
    let mut g = scenario(Player::P0);
    let chosen = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    start(&mut g, Player::P0, Player::P1);
    pick(&mut g, chosen);
    assert!(matches!(
        g.state.pending,
        Pending::EntryLife { player: Player::P1 }
    ));
    assert_eq!(g.object_view(chosen).zone, Zone::Hand);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 0);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let action = Action::ChooseEntryLife { pay: false };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let rev = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(Player::P1, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    settle(&mut raw);
    assert_eq!(raw.object_view(chosen).zone, Zone::Battlefield);
    assert_eq!(raw.state.players[1].cards_drawn_this_turn, 1);
    indexed.restore(&saved).unwrap();
    drop(g);
    act(&mut indexed, Action::ChooseEntryLife { pay: true });
    settle(&mut indexed);
    assert_eq!(indexed.object_view(chosen).zone, Zone::Battlefield);
    assert_eq!(indexed.state.players[1].cards_drawn_this_turn, 1);
    assert_eq!(indexed.state.players[1].life, 18);
}
