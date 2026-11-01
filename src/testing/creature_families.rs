//! Shared ordinary outcomes for exact, uncoupled creature configurations.
use super::*;
use crate::cards::creature_coverage_bindings::{Ability, Event, Family, Member, Payload, families};

fn outcome(family: &Family, member: &Member, ability: Ability, death: bool) {
    println!(
        "creature outcome {}: {} / death={death}",
        family.id, member.card
    );
    let mut g = game();
    let entry =
        matches!(ability.event, Event::Enter) || matches!(ability.event, Event::EnterDie) && !death;
    let source = put(
        &mut g,
        Player::P0,
        &member.card,
        if entry { Zone::Hand } else { Zone::Battlefield },
    );
    let mut arranged = vec![];
    if matches!(ability.effect, Payload::Surveil | Payload::Scry) {
        for _ in 0..ability.amount {
            arranged.push(put(&mut g, Player::P0, "Island", Zone::Library));
        }
    }
    if matches!(ability.effect, Payload::DiscardOpponent) {
        for _ in 0..ability.amount {
            put(&mut g, Player::P1, "Forest", Zone::Hand);
        }
    }
    let mut multiplier = 1;
    let mut outside_draw = 0;
    let mut outside_life = 0;
    match ability.event {
        Event::Enter => cast(&mut g, source, &[]),
        Event::EnterDie if !death => cast(&mut g, source, &[]),
        Event::Die | Event::EnterDie => {
            let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
            cast(&mut g, murder, &[Target::Card(source)]);
        }
        Event::Attack | Event::Tapped => attack(&mut g, source),
        Event::LifeGained => {
            let ajani = put(&mut g, Player::P0, "Ajani Resolute", Zone::Battlefield);
            g.state.objects[ajani as usize].counters.loyalty = 3;
            activate(&mut g, ajani, 0, &[]);
            outside_life = 1;
        }
        Event::OtherEnters => {
            let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
            cast(&mut g, bear, &[]);
        }
        Event::Draw | Event::SecondDraw => {
            let study = put(&mut g, Player::P0, "Quick Study", Zone::Hand);
            cast(&mut g, study, &[]);
            outside_draw = 2;
            if matches!(ability.event, Event::Draw) {
                multiplier = 2;
            }
        }
        Event::NoncreatureCast => {
            let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
            cast(&mut g, shock, &[Target::Player(Player::P1)]);
        }
    }
    let is_arrangement = matches!(ability.effect, Payload::Surveil | Payload::Scry);
    if is_arrangement {
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        let destination = if matches!(ability.effect, Payload::Surveil) {
            LibraryDestination::Graveyard
        } else {
            LibraryDestination::Bottom
        };
        for &card in &arranged {
            act(&mut g, Action::ArrangeLibrary { card, destination });
        }
    }
    g.prepare_triggers();
    for _ in 0..32 {
        if settled(&g) {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::Discard { .. }
            )
        });
    }
    assert!(settled(&g));
    let a = usize::from(ability.amount) * multiplier;
    let b = usize::from(ability.secondary) * multiplier;
    match ability.effect {
        Payload::Draw => assert_eq!(g.hand_count(Player::P0), outside_draw + a),
        Payload::DrawDiscard => {
            assert_eq!(g.hand_count(Player::P0), outside_draw + a - b);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.controller == Player::P0
                        && o.card == Card::Forest
                        && o.zone == Zone::Graveyard)
                    .count(),
                b
            );
        }
        Payload::GainLife => assert_eq!(g.state.players[0].life, 20 + outside_life + a as i128),
        Payload::Drain => {
            assert_eq!(g.state.players[1].life, 20 - a as i128);
            assert_eq!(g.state.players[0].life, 20 + outside_life + b as i128);
        }
        Payload::DiscardOpponent => {
            assert_eq!(g.hand_count(Player::P1), 0);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.controller == Player::P1 && o.zone == Zone::Graveyard)
                    .count(),
                a
            );
        }
        Payload::Surveil => {
            assert!(
                arranged
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.state.players[0].library.len(), 8);
        }
        Payload::Scry => {
            assert_eq!(g.state.players[0].library.len(), 8 + a);
            assert!(
                g.state.players[0].library[..a]
                    .iter()
                    .all(|id| arranged.contains(id))
            );
        }
        Payload::Mill => {
            assert_eq!(g.state.players[0].library.len(), 8 - a);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.controller == Player::P0
                        && o.card == Card::Forest
                        && o.zone == Zone::Graveyard)
                    .count(),
                a
            );
        }
        Payload::Counters => assert_eq!(g.counter_count(source, CounterKind::PlusOne), a as u64),
        Payload::Grow => {
            assert_eq!(
                stats(&g, source),
                (
                    i128::from(member.power) + a as i128,
                    i128::from(member.toughness) + b as i128
                )
            );
            g.finish_cleanup();
            assert_eq!(
                stats(&g, source),
                (member.power.into(), member.toughness.into())
            );
        }
    }
}
#[test]
fn representative_creature_abilities_have_their_printed_outcomes() {
    for family in families() {
        let member = family
            .members
            .iter()
            .find(|m| m.card == family.representative)
            .unwrap();
        for &ability in &family.abilities {
            outcome(&family, member, ability, false);
            if matches!(ability.event, Event::EnterDie) {
                outcome(&family, member, ability, true);
            }
        }
        covered(&format!("family/{}", family.id));
    }
}
