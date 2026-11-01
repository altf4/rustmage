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
fn callous_counts_creatures_that_died_under_its_controllers_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let one = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let two = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = p;
        let before = g.snapshot();
        g.move_object(one, Zone::Graveyard);
        g.move_object(two, Zone::Graveyard);
        g.move_object(stolen, Zone::Graveyard);
        g.move_object(enemy, Zone::Graveyard);
        let mut branch = g.clone();
        branch.restore(&before).unwrap();
        assert_eq!(
            branch.state.players[p.index()].controlled_creatures_died_this_turn,
            0
        );
        assert_eq!(
            g.state.players[p.index()].controlled_creatures_died_this_turn,
            3
        );
        let card = put(&mut g, p, "Callous Sell-Sword // Burn Together", Zone::Hand);
        big_helpers::cast(&mut g, card, None, &[]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(card).counters.plus_one, 3);
        assert_eq!(
            g.stats(card),
            Stats {
                power: 5,
                toughness: 5
            }
        );
    }
}

#[test]
fn burn_together_damage_uses_creature_then_sacrifices_and_goes_on_adventure() {
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
fn burn_together_negative_power_is_zero_damage_but_still_sacrifices() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].growth.power = -3;
        let life = g.state.players[p.other().index()].life;
        start_burn(&mut g, p, source, Target::Player(p.other()));
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn burn_together_with_no_legal_targets_does_not_resolve_or_adventure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipient = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = start_burn(&mut g, p, source, Target::Card(recipient));
        g.move_object(source, Zone::Hand);
        g.move_object(recipient, Zone::Hand);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
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

#[test]
fn burn_together_uses_creatures_lifelink_for_its_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
        let life = g.state.players[p.index()].life;
        start_burn(&mut g, p, source, Target::Player(p.other()));
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}

#[test]
fn burn_together_can_damage_a_battle() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let battle = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        recipe.overrides = Some(CopyOverrides {
            kinds: Some(crate::cards::BATTLE_KIND),
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(battle), recipe, CopyDeadline::EndOfTurn);
        g.add_counter(battle, CounterKind::Defense, 5);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        start_burn(&mut g, p, source, Target::Card(battle));
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(battle, CounterKind::Defense), 3);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
