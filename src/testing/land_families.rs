//! Ordinary land entry, conditional entry choices, and printed mana outcomes.
use super::*;
use crate::cards::land_coverage_bindings::{Entry, EntryTrigger, Family, Member, families};

fn play(g: &mut Game, member: &Member, pay_life: bool) -> CardId {
    let land = put(g, Player::P0, &member.card, Zone::Hand);
    act(g, Action::PlayLand { card: land });
    if g.legal_actions()
        .contains(&Action::ChooseEntryLife { pay: pay_life })
    {
        act(g, Action::ChooseEntryLife { pay: pay_life });
    }
    assert_eq!(g.object_view(land).zone, Zone::Battlefield);
    assert_eq!(g.object_view(land).controller, Player::P0);
    land
}
fn entry_outcome(family: &Family, member: &Member, enabled: bool) {
    let mut g = game();
    match family.entry {
        Entry::AtMostTwo | Entry::AtLeastTwo => {
            let others = if matches!(family.entry, Entry::AtMostTwo) {
                if enabled { 2 } else { 3 }
            } else if enabled {
                2
            } else {
                1
            };
            for _ in 0..others {
                put(&mut g, Player::P0, "Forest", Zone::Battlefield);
            }
        }
        Entry::LowLife if enabled => g.state.players[1].life = 13,
        Entry::Planeswalker if enabled => {
            let ajani = put(&mut g, Player::P0, "Ajani Resolute", Zone::Battlefield);
            g.state.objects[ajani as usize].counters.loyalty = 3;
        }
        _ => (),
    }
    let arranged = if matches!(family.trigger, EntryTrigger::Scry | EntryTrigger::Surveil) {
        assert_eq!(family.amount, 1);
        Some(put(&mut g, Player::P0, "Island", Zone::Library))
    } else {
        None
    };
    let land = play(&mut g, member, enabled);
    let tapped = match family.entry {
        Entry::Never => false,
        Entry::Always => true,
        _ => !enabled,
    };
    assert_eq!(g.object_view(land).tapped, tapped, "{} entry", member.card);
    if let Some(card) = arranged {
        let destination = if matches!(family.trigger, EntryTrigger::Surveil) {
            LibraryDestination::Graveyard
        } else {
            LibraryDestination::Bottom
        };
        reach(&mut g, &Action::ArrangeLibrary { card, destination });
        act(&mut g, Action::ArrangeLibrary { card, destination });
    }
    if matches!(family.trigger, EntryTrigger::DamageOpponent) {
        reach(
            &mut g,
            &Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
    }
    resolve(&mut g);
    let paid = if matches!(family.entry, Entry::PayTwoLife) && enabled {
        2
    } else {
        0
    };
    let gained = if matches!(family.trigger, EntryTrigger::GainLife) {
        i128::from(family.amount)
    } else {
        0
    };
    assert_eq!(g.state.players[0].life, 20 - paid + gained);
    match family.trigger {
        EntryTrigger::DamageOpponent => {
            assert_eq!(g.state.players[1].life, 20 - i128::from(family.amount))
        }
        EntryTrigger::Scry => {
            let card = arranged.unwrap();
            assert_eq!(g.state.players[0].library[0], card);
            assert_eq!(g.state.players[0].library.len(), 9);
        }
        EntryTrigger::Surveil => {
            assert_eq!(g.object_view(arranged.unwrap()).zone, Zone::Graveyard);
            assert_eq!(g.state.players[0].library.len(), 8);
        }
        _ => (),
    }
}
#[test]
fn representative_land_entry_and_mana_have_their_printed_outcomes() {
    for family in families() {
        let member = family
            .members
            .iter()
            .find(|m| m.card == family.representative)
            .unwrap();
        println!("land outcome {}: {}", family.id, member.card);
        entry_outcome(&family, member, false);
        if !matches!(family.entry, Entry::Never | Entry::Always) {
            entry_outcome(&family, member, true);
        }
        for color in 0..6 {
            if family.mask & (1 << color) == 0 {
                continue;
            }
            let mut g = game();
            let land = put(&mut g, Player::P0, &member.card, Zone::Battlefield);
            g.state.players[0].mana = [0; 6];
            let mut expected = [0; 6];
            expected[color] = 1;
            let actions = g.legal_actions();
            let action = actions.iter().find(|a| {
                matches!(a,Action::ActivateMana {card,production,..} | Action::TapGrantedMana {card,production,..} if *card==land && *production==expected)
            }).or_else(|| actions.iter().find(|a|matches!(a,Action::TapMana {card} if *card==land)))
                .expect("printed mana choice").clone();
            act(&mut g, action);
            assert_eq!(g.state.players[0].mana, expected);
            assert!(g.object_view(land).tapped);
        }
        covered(&format!("family/{}", family.id));
    }
}
#[test]
fn each_land_is_played_from_hand() {
    for family in families() {
        for member in &family.members {
            println!("land play: {}", member.card);
            play(&mut game(), member, false);
            covered(&format!("card/{}/play/land", member.catalogue_id));
        }
    }
}
