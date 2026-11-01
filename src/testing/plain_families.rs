//! One gameplay outcome per exact shared spell configuration, plus every cast.
use super::*;
use crate::cards::coverage_bindings::{Family, Member, SpellFamily, SpellTarget, families};

fn play(family: &Family, member: &Member, assert_effect: bool) {
    println!("plain spell {}: {}", family.id, member.card);
    let mut g = game();
    // Large positive counters keep even printed -9999 toughness observable.
    let creature = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[creature as usize].counters.plus_one = 10000;
    let target = match family.effect.target() {
        SpellTarget::None => vec![],
        SpellTarget::Creature => vec![Target::Card(creature)],
        SpellTarget::Any | SpellTarget::Player | SpellTarget::Opponent => {
            vec![Target::Player(Player::P1)]
        }
        SpellTarget::Spell | SpellTarget::CreatureSpell | SpellTarget::NoncreatureSpell => {
            g.state.active = Player::P1;
            g.state.priority = Player::P1;
            let (name, targets) = if matches!(family.effect.target(), SpellTarget::NoncreatureSpell)
            {
                ("Shock", vec![Target::Player(Player::P0)])
            } else {
                ("Bear Cub", vec![])
            };
            let opponent_spell = put(&mut g, Player::P1, name, Zone::Hand);
            cast(&mut g, opponent_spell, &targets);
            act(&mut g, Action::Pass);
            vec![Target::Spell(opponent_spell)]
        }
    };
    let source = put(&mut g, Player::P0, &member.card, Zone::Hand);
    g.state.players[0].mana = member.cost.pool();
    cast(&mut g, source, &target);
    assert_eq!(
        g.state.players[0].mana, [0; 6],
        "exact printed mana payment"
    );
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
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    covered(&format!("card/{}/cast/normal", member.catalogue_id));
    if !assert_effect {
        return;
    }
    match family.effect {
        SpellFamily::Draw { amount } => assert_eq!(g.hand_count(Player::P0), usize::from(amount)),
        SpellFamily::GainLife { amount } => {
            assert_eq!(g.state.players[0].life, 20 + i128::from(amount))
        }
        SpellFamily::Damage { amount, .. }
            if matches!(family.effect.target(), SpellTarget::Creature) =>
        {
            assert_eq!(
                g.state.objects[creature as usize].damage,
                i128::from(amount)
            )
        }
        SpellFamily::Damage { amount, .. } => {
            assert_eq!(g.state.players[1].life, 20 - i128::from(amount))
        }
        SpellFamily::DestroyCreature => assert_eq!(g.object_view(creature).zone, Zone::Graveyard),
        SpellFamily::ReturnCreature => assert_eq!(g.object_view(creature).zone, Zone::Hand),
        SpellFamily::GrowCreature { power, toughness } => {
            assert_eq!(
                stats(&g, creature),
                (10002 + i128::from(power), 10002 + i128::from(toughness))
            );
            g.finish_cleanup();
            assert_eq!(stats(&g, creature), (10002, 10002));
        }
        SpellFamily::Counter { .. } => {
            let Target::Spell(countered) = target[0] else {
                unreachable!()
            };
            assert_eq!(g.object_view(countered).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[0].life, 20,
                "countered noncreature does not deal damage"
            );
        }
        SpellFamily::DrawLose { cards, life } => {
            assert_eq!(g.hand_count(Player::P0), usize::from(cards));
            assert_eq!(g.state.players[0].life, 20 - i128::from(life));
        }
        SpellFamily::DrawDiscard { cards, discard } => {
            assert_eq!(g.hand_count(Player::P0), usize::from(cards - discard));
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.controller == Player::P0 && o.zone == Zone::Graveyard)
                    .count(),
                usize::from(discard) + 1
            );
        }
    }
    covered(&format!("family/{}", family.id));
}

#[test]
fn representative_outcomes_and_each_members_normal_cast() {
    for family in families() {
        for member in &family.members {
            play(&family, member, member.card == family.representative);
        }
    }
}
