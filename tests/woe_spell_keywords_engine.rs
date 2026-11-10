//! Live instant and sorcery keyword grants use the actual damage source.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledInstantSorceryKeywords(
            Keywords::LIFELINK,
        )],
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn a_live_provider_grants_actual_spell_and_copy_damage_lifelink() {
    let copy = Card::from_name("Quick Study").unwrap();
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER), (copy, &COPY)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut g = scenario(player);
        let provider = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        let copy = put(&mut g, player, "Quick Study", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(other)));
        assert!(g.object_view(shock).keywords.contains(Keywords::LIFELINK));

        let mut removed = g.clone();
        act(&mut removed, Action::Pass);
        let bounce = put(&mut removed, other, "Unsummon", Zone::Hand);
        cast(&mut removed, bounce, Some(Target::Card(provider)));
        settle(&mut removed);
        assert_eq!(removed.state.players[player.index()].life, 20);
        assert_eq!(removed.state.players[other.index()].life, 18);

        cast(&mut g, copy, Some(Target::Spell(shock)));
        settle(&mut g);
        assert_eq!(g.state.players[player.index()].life, 24);
        assert_eq!(g.state.players[other.index()].life, 16);
    }
}

#[test]
fn an_adventure_spell_gains_lifelink_but_the_creature_doing_power_damage_does_not() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut g = scenario(player);
        put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let adventure = put(&mut g, player, "Frolicking Familiar", Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: adventure });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(other),
            },
        );
        pay(&mut g);
        assert!(
            g.object_view(adventure)
                .keywords
                .contains(Keywords::LIFELINK)
        );
        settle(&mut g);
        assert_eq!(g.object_view(adventure).zone, Zone::Exile);
        assert_eq!(g.state.players[player.index()].life, 21);
        assert_eq!(g.state.players[other.index()].life, 19);

        let dealer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let receiver = put(&mut g, other, "Hill Giant", Zone::Battlefield);
        let bite = put(&mut g, player, "Bite Down", Zone::Hand);
        announce(
            &mut g,
            bite,
            &[Target::Card(dealer), Target::Card(receiver)],
        );
        assert!(g.object_view(bite).keywords.contains(Keywords::LIFELINK));
        settle(&mut g);
        assert_eq!(g.object_view(receiver).damage, 2);
        assert_eq!(g.state.players[player.index()].life, 21);
    }
}
