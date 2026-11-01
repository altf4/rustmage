//! Focused printed outcomes; shared engine invariants live in core tests.
//! Callous Sell-Sword's controlled deaths and Burn Together's two targets.
use super::*;

fn start_burn(g: &mut Game, p: Player, source: CardId, destination: Target) -> CardId {
    let spell = put(g, p, "Callous Sell-Sword // Burn Together", Zone::Hand);
    big_helpers::checked(g, Action::AnnounceAdventure { card: spell });
    big_helpers::checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    big_helpers::checked(
        g,
        Action::ChooseTarget {
            target: destination,
        },
    );
    pay(g);
    spell
}

#[test]
fn burn_together_damage_uses_creature_then_sacrifices_and_goes_on_adventure() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.other().index()].life;
        let spell = start_burn(&mut g, p, source, Target::Player(p.other()));
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: spell })
        );
    }
}

#[test]
fn burn_together_keeps_legal_instructions_when_one_target_is_gone() {
    {
        let p = Player::P0;
        for source_gone in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = start_burn(&mut g, p, source, Target::Card(enemy));
            g.move_object(if source_gone { source } else { enemy }, Zone::Hand);
            big_helpers::settle(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if source_gone {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if source_gone {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
        }
    }
}

#[test]
fn callous_front_has_stable_identity_and_front_characteristics() {
    let card = Card::from_name("Callous Sell-Sword // Burn Together").unwrap();
    assert_eq!(Card::from_name("Callous Sell-Sword"), Some(card));
    assert_eq!(card.identity(), "Callous Sell-Sword // Burn Together");
    assert_eq!(card.name(), "Callous Sell-Sword");
    assert_eq!(card.info().mana_cost, "{1}{B}");
    assert_eq!(card.fixed_mana_value(), 2);
    assert_eq!(Card::from_name("Burn Together"), card.adventure_spell());
}
