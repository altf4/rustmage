//! Dynamic creature convoke grants are evaluated in the actual payment world.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ConvokeCreatureSpells],
    }],
    ..CardRules::EMPTY
};
#[test]
fn dynamic_creature_convoke_pays_with_summoning_sick_creatures_and_rejects_noncreatures() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[a as usize].summoning_sick = true;
        g.state.objects[b as usize].summoning_sick = true;
        let spell = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::BeginConvoke);
        let object = g.object_ref(a);
        act(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: convoke::ConvokeContribution::Green,
            },
        );
        let object = g.object_ref(b);
        act(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: convoke::ConvokeContribution::Generic,
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        assert!(g.state.objects[a as usize].tapped && g.state.objects[b as usize].tapped);
        let noncreature = put(&mut g, p, "Divination", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: noncreature });
    }
}
#[test]
fn departed_or_ability_removed_grant_cannot_pay_creature_spell() {
    const BLANK: CardRules = CardRules {
        effect: Effect::Permanent,
        aura: Some(AuraSpec {
            kinds: 0,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords::NONE,
            },
        }),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &GRANT),
        (Card::from_name("Pacifism").unwrap(), &BLANK),
    ]);
    for removed in [false, true] {
        let mut g = scenario(Player::P0);
        let grant = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        g.state.players[0].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        if removed {
            let aura = put(&mut g, Player::P1, "Pacifism", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(grant));
            reject(&mut g, Action::AnnounceCast { card: spell });
            g.move_object(aura, Zone::Graveyard);
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell })
            );
        } else {
            g.move_object(grant, Zone::Graveyard);
            reject(&mut g, Action::AnnounceCast { card: spell });
        }
    }
}
