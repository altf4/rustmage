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

#[test]
fn sos_preparation_public_copy_paid_cast_transaction_and_exact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Landscape Painter", Zone::Hand);
        let mana: u32 = g.state.players[p.index()].mana.iter().sum();
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            2
        );
        assert_eq!(stats(&g, source), (2, 1));
        assert!(g.object_view(source).prepared);
        let row = g.state.preparations.as_ref().unwrap()[0];
        let copy = row.spell_copy.card;
        assert_eq!(g.object_view(copy).name(), "Vibrant Idea");
        assert_eq!(g.object_view(copy).zone, Zone::Exile);
        assert_eq!(g.object_view(copy).card.info().mana_cost, "{4}{U}");
        for viewer in [p, p.other()] {
            let observation = g.observe(viewer);
            assert!(
                observation
                    .objects
                    .iter()
                    .any(|o| o.reference == row.spell_copy)
            );
            assert_eq!(observation.preparations, vec![row]);
        }
        g.prepare_permanent(g.object_ref(source));
        assert_eq!(g.state.preparations.as_ref().unwrap().len(), 1);
        big_helpers::checked(&mut g, Action::AnnounceExile { card: copy });
        assert!(g.object_view(source).prepared);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert!(g.object_view(source).prepared);
        let hand = g.hand_count(p);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        big_helpers::checked(&mut g, payment);
        assert!(!g.object_view(source).prepared);
        assert!(g.state.preparations.is_none());
        assert_eq!(
            g.state.stack.last().unwrap().source_card.name(),
            "Vibrant Idea"
        );
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .cast
                .unwrap()
                .payment
                .iter()
                .sum::<u32>(),
            5
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.object_view(copy).zone, Zone::Outside);
        g.move_object(source, Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let second = g.state.preparations.as_ref().unwrap()[0];
        assert_ne!(second.permanent, row.permanent);
        assert_ne!(second.spell_copy, row.spell_copy);
        g.move_object(source, Zone::Exile);
        assert!(g.state.preparations.is_none());
        assert_eq!(g.object_view(second.spell_copy.card).zone, Zone::Outside);
    }
}

#[test]
fn sos_preparation_survives_ability_type_and_controller_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Landscape Painter", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let copy = g.state.preparations.as_ref().unwrap()[0].spell_copy.card;
        let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(source)));
        settle(&mut g);
        assert!(!g.is_creature(source));
        assert!(!g.printed_abilities_available(source));
        assert!(g.object_view(source).prepared);
        assert!(g.can_cast_prepared(p, copy));
        let source_owner = g.state.objects[source as usize].owner;
        g.record_control(g.object_ref(source), p.other(), None);
        assert_eq!(g.state.objects[source as usize].owner, source_owner);
        assert_eq!(g.state.objects[copy as usize].owner, p);
        assert!(!g.can_cast_prepared(p, copy));
        assert!(g.can_cast_prepared(p.other(), copy));
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [8; 6];
        let hand = g.hand_count(p.other());
        act(&mut g, Action::AnnounceExile { card: copy });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), hand + 2);
        assert!(!g.object_view(source).prepared);
    }
}

fn prepared_cast(g: &mut Game, source: CardId, targets: &[Target]) -> CardId {
    let row = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(source))
        .copied()
        .unwrap();
    let id = row.spell_copy.card;
    act(g, Action::AnnounceExile { card: id });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    id
}

#[test]
fn sos_prepared_whole_card_bodies_and_inset_effects() {
    for p in [Player::P0, Player::P1] {
        for (name, power, toughness, keyword, bonus_keyword) in [
            (
                "Honorbound Page",
                3,
                3,
                Keywords::FIRST_STRIKE,
                Keywords::FLYING,
            ),
            (
                "Quill-Blade Laureate",
                1,
                1,
                Keywords::DOUBLE_STRIKE,
                Keywords::DOUBLE_STRIKE,
            ),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (power, toughness));
            assert!(g.keywords(source).contains(keyword));
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            prepared_cast(&mut g, source, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(stats(&g, target), (3, 2));
            assert!(g.keywords(target).contains(bonus_keyword));
            g.finish_cleanup();
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(bonus_keyword));
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Campus Composer", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 4));
        prepared_cast(&mut g, source, &[]);
        settle(&mut g);
        let made = tokens(&g, p);
        assert_eq!(made.len(), 1);
        assert_eq!(stats(&g, made[0]), (3, 3));
        assert_eq!(g.object_view(made[0]).colors, 2 | 8);
        assert!(g.keywords(made[0]).contains(Keywords::FLYING));
        let mut g = scenario(p);
        let source = put(&mut g, p, "Spellbook Seeker", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        prepared_cast(&mut g, source, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
    }
}

#[test]
fn sos_prepared_ward_counter_decline_and_attack_reprepare() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Campus Composer", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        g.state.priority = p.other();
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(source)));
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bounce).zone, Zone::Graveyard);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Encouraging Aviator", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (2, 3));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(!g.object_view(source).prepared);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(g.object_view(source).prepared);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        prepared_cast(&mut g, source, &[Target::Card(target)]);
        settle(&mut g);
        assert!(g.keywords(target).contains(Keywords::FLYING));
        assert!(!g.object_view(source).prepared);
    }
}

#[test]
fn sos_preparation_reexiled_copy_ceases_and_designation_is_not_copied() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Landscape Painter", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let row = g.state.preparations.as_ref().unwrap()[0];
        g.move_object(row.spell_copy.card, Zone::Exile);
        g.state_based_actions();
        assert_eq!(g.object_view(row.spell_copy.card).zone, Zone::Outside);
        assert!(g.object_view(source).prepared);
        assert!(!g.can_cast_prepared(p, row.spell_copy.card));
        let source = put(&mut g, p, "Encouraging Aviator", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        g.prepare_permanent(g.object_ref(source));
        let definition = g.state.objects[source as usize].card;
        g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        );
        g.advance_entries();
        let copied = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token() && o.card == definition)
            .map(|(id, _)| id as CardId)
            .unwrap();
        assert!(g.object_view(source).prepared);
        assert!(!g.object_view(copied).prepared);
        g.prepare_permanent(g.object_ref(copied));
        assert!(g.object_view(copied).prepared);
        let before = g.snapshot();
        let rows = g.state.preparations.as_ref().unwrap().len();
        g.prepare_permanent(g.object_ref(copied));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.state.preparations.as_ref().unwrap().len(), rows);
    }
}
