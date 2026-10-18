//! Paid Kaito interactions through the existing loyalty, trigger and emblem core.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;
use crate::ChoiceColor;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Kaito, Cunning Infiltrator", Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(id, CounterKind::Loyalty), 3);
    id
}
fn activate(g: &mut Game, source: CardId, ability: u16, target: Option<CardId>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(card) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    for _ in 0..5 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("Kaito announcement did not finish");
}
fn until_discard(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no loot discard");
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn ninjas(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card == Card::KaitoNinjaToken
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn fdn_kaito_paid_optional_zero_target_loot_privacy_and_loyalty_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        let drawn = spawn(&mut g, p, Card::Island, Zone::Library);
        let held = spawn(&mut g, p, Card::Swamp, Zone::Hand);
        let foreign = spawn(&mut g, p.other(), Card::Island, Zone::Hand);
        activate(&mut g, kaito, 0, None);
        assert_eq!(g.counter_count(kaito, CounterKind::Loyalty), 4);
        until_discard(&mut g);
        assert_eq!(g.state.objects[drawn as usize].zone, Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == drawn));
        assert!(g.observe(p).objects.iter().any(|o| o.id == drawn));
        reject(&mut g, Action::Discard { card: foreign });
        checked(&mut g, Action::Discard { card: held });
        settle(&mut g);
        assert_eq!(g.state.objects[held as usize].zone, Zone::Graveyard);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == kaito))
        );
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: kaito,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn fdn_kaito_controlled_target_unblockability_cleanup_and_late_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: kaito,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        until_discard(&mut g);
        let discard = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        checked(&mut g, discard);
        settle(&mut g);
        assert!(g.keywords(own).contains(Keywords::CANT_BE_BLOCKED));
        assert!(!g.keywords(other).contains(Keywords::CANT_BE_BLOCKED));
        next_turn(&mut g);
        assert!(!g.keywords(own).contains(Keywords::CANT_BE_BLOCKED));

        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        activate(&mut g, kaito, 0, Some(own));
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        let bounce = named(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(own)]);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws);
        assert_eq!(g.counter_count(kaito, CounterKind::Loyalty), 4);
    }
}
#[test]
fn fdn_kaito_last_loyalty_creates_correct_ninja_after_source_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: kaito,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        g.set_counters(kaito, Counters::of(CounterKind::Loyalty, 2));
        activate(&mut g, kaito, 1, None);
        settle(&mut g);
        assert_eq!(g.state.objects[kaito as usize].zone, Zone::Graveyard);
        let ids = ninjas(&g, p);
        assert_eq!(ids.len(), 1);
        let view = g.object_view(ids[0]);
        assert_eq!(view.name(), "Ninja Token");
        assert!(view.has_subtype("Ninja"));
        assert_eq!(view.colors, 1 << ChoiceColor::Blue.index());
        assert_eq!(
            g.stats(ids[0]),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert!(!view.tapped);
        assert!(view.summoning_sick);
        assert_eq!(ninjas(&g, p.other()).len(), 0);
    }
}
#[test]
fn fdn_kaito_command_emblem_survives_source_and_observes_both_players_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        g.set_counters(kaito, Counters::of(CounterKind::Loyalty, 9));
        activate(&mut g, kaito, 2, None);
        settle(&mut g);
        assert_eq!(g.state.objects[kaito as usize].zone, Zone::Graveyard);
        let emblem = g
            .state
            .objects
            .iter()
            .find(|o| o.zone == Zone::Command && o.card == Card::KaitoEmblem)
            .unwrap();
        assert_eq!(emblem.controller, p);
        let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, creature, None, &[]);
        settle(&mut g);
        assert_eq!(ninjas(&g, p).len(), 1);
        checked(&mut g, Action::Pass);
        let growth = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(ninjas(&g, p).len(), 2);
        assert_eq!(ninjas(&g, p.other()).len(), 0);
    }
}
#[test]
fn fdn_kaito_per_creature_combat_damage_counts_not_fragments_or_other_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kaito = paid(&mut g, p);
        let first = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let event = |source, player, amount, combat| DamageEvent {
            source: g.damage_source(source),
            target: Target::Player(player),
            amount,
            combat,
        };
        let events = [
            event(first, p.other(), 1, true),
            event(first, p.other(), 1, true),
            event(second, p.other(), 1, true),
            event(foreign, p, 1, true),
            event(artifact, p.other(), 1, true),
            event(first, p.other(), 1, false),
        ];
        g.apply_damage_batch(&events);
        settle(&mut g);
        assert_eq!(g.counter_count(kaito, CounterKind::Loyalty), 5);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let event = DamageEvent {
            source: g.damage_source(first),
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[event]);
        settle(&mut g);
        assert_eq!(g.counter_count(kaito, CounterKind::Loyalty), 7);
    }
}
