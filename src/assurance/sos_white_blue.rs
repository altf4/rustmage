//! Whole-card Secrets of Strixhaven white/blue outcomes, through paid actions.
use super::*;
use crate::engine::untap_rules::UntapCause;

fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(i, _)| i as CardId)
        .collect()
}

#[test]
fn sos_harsh_annotation_uses_controller_lki_even_when_destroy_fails() {
    for p in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[victim as usize].controller = p.other();
            if indestructible {
                g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            let spell = put(&mut g, p, "Harsh Annotation", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(victim)));
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            let made = tokens(&g, p.other());
            assert_eq!(made.len(), 1);
            let token = made[0];
            assert_eq!(stats(&g, token), (1, 1));
            assert!(g.keywords(token).contains(Keywords::FLYING));
            assert_eq!(g.object_view(token).colors, 1 | 4);
            assert!(g.has_subtype(token, "Inkling"));
            assert!(tokens(&g, p).is_empty());
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Harsh Annotation", Zone::Hand);
        // Representative paid announcement checks raw/index ownership and restore.
        big_helpers::cast(&mut g, spell, None, &[Target::Card(victim)]);
        let old = g.object_ref(victim);
        g.move_object(victim, Zone::Hand);
        g.move_object(victim, Zone::Battlefield);
        assert_ne!(g.object_ref(victim), old);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert!(tokens(&g, p.other()).is_empty());
    }
}

#[test]
fn sos_rapier_wit_turn_condition_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for own_turn in [true, false] {
            let mut g = scenario(p);
            if !own_turn {
                g.state.active = p.other();
            }
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Rapier Wit", Zone::Hand);
            let hand = g.hand_count(p);
            cast(&mut g, spell, Some(Target::Card(victim)));
            settle(&mut g);
            assert!(g.object_view(victim).tapped);
            assert_eq!(
                g.counter_count(victim, CounterKind::Stun),
                u64::from(own_turn)
            );
            assert_eq!(g.hand_count(p), hand);
            g.untap_objects(&[victim], UntapCause::Effect);
            assert_eq!(g.object_view(victim).tapped, own_turn);
            assert_eq!(g.counter_count(victim, CounterKind::Stun), 0);
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Rapier Wit", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, spell, Some(Target::Card(victim)));
        g.move_object(victim, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand - 1);
    }
}

fn homesickness(g: &mut Game, spell: CardId, player: Player, creatures: &[CardId]) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    for &creature in creatures {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn sos_homesickness_zero_two_and_partial_targets() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let creatures: Vec<_> = (0..count)
                .map(|_| put(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
                .collect();
            let spell = put(&mut g, p, "Homesickness", Zone::Hand);
            let hand = g.hand_count(p.other());
            homesickness(&mut g, spell, p.other(), &creatures);
            if count == 2 {
                g.move_object(creatures[0], Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p.other()), hand + 2 + usize::from(count == 2));
            for (index, &creature) in creatures.iter().enumerate() {
                if count == 2 && index == 0 {
                    continue;
                }
                assert!(g.object_view(creature).tapped);
                assert_eq!(g.counter_count(creature, CounterKind::Stun), 1);
            }
        }
    }
}

#[test]
fn sos_wisdom_returns_only_own_spells_exiles_and_persists_hand_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let instant = put(&mut g, p, "Unsummon", Zone::Graveyard);
        let sorcery = put(&mut g, p, "Divination", Zone::Graveyard);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let opposing = put(&mut g, p.other(), "Divination", Zone::Graveyard);
        let spell = put(&mut g, p, "Wisdom of Ages", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(instant).zone, Zone::Hand);
        assert_eq!(g.object_view(sorcery).zone, Zone::Hand);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(g.object_view(opposing).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert!(g.state.players[p.index()].no_maximum_hand_size);
        assert!(!g.state.players[p.other().index()].no_maximum_hand_size);
        for _ in 0..10 {
            put(&mut g, p, "Forest", Zone::Hand);
        }
        g.finish_cleanup();
        assert_ne!(g.state.pending, Pending::Discard);
    }
}

#[test]
fn sos_card_copy_normal_paid_cast_counts_and_ceases_without_physical_recovery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let copy = g.allocate_card_copy(Card::from_name("Unsummon").unwrap(), p);
        g.grant_exile_play_permission(p, copy, false);
        assert_eq!(g.object_view(copy.card).zone, Zone::Exile);
        assert!(!g.object_view(copy.card).is_token());
        let casts = g.state.players[p.index()].spells_cast_this_turn;
        big_helpers::checked(&mut g, Action::AnnounceExile { card: copy.card });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, payment);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts + 1);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Exile
        );
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
        assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
        assert_ne!(g.object_ref(copy.card), copy);
        assert_eq!(
            g.state
                .free_tokens
                .iter()
                .filter(|&&id| id == copy.card)
                .count(),
            1
        );
        g.move_object(copy.card, Zone::Hand);
        assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
        let next = g.allocate_card_copy(Card::from_name("Divination").unwrap(), p);
        assert_eq!(next.card, copy.card);
        assert_ne!(next, copy);
        g.cease_card_copy(copy);
        assert_eq!(g.object_view(next.card).zone, Zone::Exile);
        g.cease_card_copy(next);
        g.cease_card_copy(next);
        assert_eq!(
            g.state
                .free_tokens
                .iter()
                .filter(|&&id| id == next.card)
                .count(),
            1
        );
    }
}
