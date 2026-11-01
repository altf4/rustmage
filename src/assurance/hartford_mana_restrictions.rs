use super::big_helpers::{cast as paid_cast, checked, next_turn, settle};
use super::*;

fn mana(g: &mut Game, source: CardId, ability: u16, production: Mana) {
    checked(
        g,
        Action::ActivateMana {
            card: source,
            ability,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
}

#[test]
fn cavern_chosen_mana_restricts_spending_and_protects_only_spell_actually_paid() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cavern = put(&mut g, p, "Cavern of Souls", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: cavern });
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name("Elf").unwrap(),
                ),
            },
        );
        let elves = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        mana(&mut g, cavern, 1, [0, 0, 0, 0, 1, 0]);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: growth })
        );
        g.move_object(cavern, Zone::Graveyard);
        paid_cast(&mut g, elves, None, &[]);
        let spell = g.state.stack.last().unwrap().id;
        assert!(
            g.observe(p.other())
                .uncounterable_spell_ids
                .contains(&spell)
        );
        checked(&mut g, Action::Pass);
        let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
        paid_cast(&mut g, cancel, None, &[Target::Spell(elves)]);
        settle(&mut g);
        assert_eq!(g.object_view(elves).zone, Zone::Battlefield);
        let ordinary = put(&mut g, p, "Cavern of Souls", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        mana(&mut g, ordinary, 0, [0, 0, 0, 0, 0, 1]);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        paid_cast(&mut g, bear, None, &[]);
        assert!(
            !g.observe(p)
                .uncounterable_spell_ids
                .contains(&g.state.stack.last().unwrap().id)
        );
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        paid_cast(&mut g, counter, None, &[Target::Spell(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn fire_crystal_reduces_red_spells_and_copies_with_independent_delayed_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let crystal = put(&mut g, p, "The Fire Crystal", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(g.keywords(bear).contains(Keywords::HASTE));
        assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        let strike = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        paid_cast(&mut g, strike, None, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].mana = [8; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: crystal,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        g.move_object(crystal, Zone::Graveyard);
        settle(&mut g);
        let delayed = g.observe(p).delayed_sacrifices;
        assert_eq!(delayed.len(), 1);
        assert_eq!(delayed[0].tokens.len(), 1);
        let copy = delayed[0].tokens[0].card;
        assert!(g.object_view(copy).is_token());
        assert_eq!(g.current_kinds(copy), 1);
        assert!(!g.keywords(copy).contains(Keywords::HASTE));
        next_turn(&mut g);
        settle(&mut g);
        assert!(!g.on_battlefield(copy));
        assert!(g.observe(p).delayed_sacrifices.is_empty());
    }
}
