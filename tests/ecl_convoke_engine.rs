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

#[test]
fn hidden_opposing_library_provider_cannot_change_public_cast_choices() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    for p in [Player::P0, Player::P1] {
        let mut hidden = scenario(p);
        let witness = put(&mut hidden, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut hidden, p, "Arbiter of Woe", Zone::Hand);
        let unknown = put(&mut hidden, p.other(), "Short Bow", Zone::Library);
        let mut ordinary = hidden.clone();
        ordinary.state.objects[unknown as usize].card = Card::Forest;
        let same_public_choice = |left: &Game, right: &Game| {
            assert_eq!(
                serde_json::to_value(left.observe(p)).unwrap(),
                serde_json::to_value(right.observe(p)).unwrap()
            );
            assert_eq!(left.legal_actions(), right.legal_actions());
        };
        same_public_choice(&hidden, &ordinary);
        for game in [&mut hidden, &mut ordinary] {
            act(game, Action::AnnounceCast { card: spell });
        }
        same_public_choice(&hidden, &ordinary);
        assert!(matches!(
            hidden.state.pending,
            Pending::AdditionalCost { .. }
        ));
        for game in [&mut hidden, &mut ordinary] {
            let object = game.object_ref(witness);
            act(game, Action::ChooseAdditionalCost { object });
        }
        same_public_choice(&hidden, &ordinary);
        for game in [&mut hidden, &mut ordinary] {
            pay(game);
        }
        same_public_choice(&hidden, &ordinary);
        assert_eq!(hidden.object_view(witness).zone, Zone::Graveyard);
    }
}

#[test]
fn sacrificing_public_suppressor_or_exiler_restores_convoke_for_the_same_cast() {
    const CREATURE_GRANT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..GRANT
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CREATURE_GRANT)]);
    for linked_return in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let grant = put(
            &mut g,
            p,
            "Grizzly Bears",
            if linked_return {
                Zone::Exile
            } else {
                Zone::Battlefield
            },
        );
        let binder = put(&mut g, p, "Tishana's Tidebinder", Zone::Battlefield);
        let mut donors = vec![(grant, ConvokeContribution::Generic)];
        for _ in 0..2 {
            donors.push((
                put(&mut g, p, "Vampire Nighthawk", Zone::Battlefield),
                ConvokeContribution::Black,
            ));
        }
        for _ in 0..3 {
            donors.push((
                put(&mut g, p, "Bear Cub", Zone::Battlefield),
                ConvokeContribution::Generic,
            ));
        }
        if linked_return {
            g.state.exile_links.push(ExileLink {
                source: g.object_ref(binder),
                exiled: g.object_ref(grant),
                return_zone: Zone::Battlefield,
                ability_origin: None,
            });
        } else {
            // Arrange Tidebinder's already-resolved exact-source duration; the
            // actual announcement, sacrifice, restored grant and payment follow.
            let timestamp = g.next_role_timestamp();
            let effect = SourceLinkedEffectView {
                source: g.object_ref(binder),
                recipient: g.object_ref(grant),
                controller: p,
                kind: SourceLinkedEffectKind::RemoveAbilities,
                timestamp,
            };
            g.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(effect);
        }
        let spell = put(&mut g, p, "Arbiter of Woe", Zone::Hand);
        let definition = g.state.objects[spell as usize].card;
        g.state.players[p.index()].mana = [0; 6];
        assert!(!convoke::available(&g, p, definition));
        assert!(convoke::potential(&g, definition));
        act(&mut g, Action::AnnounceCast { card: spell });
        assert!(
            g.cast_cost_world(g.state.announcing.as_ref().unwrap())
                .cost_completion()
        );
        act(&mut g, Action::BeginOrderedCosts);
        let clauses = &g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses;
        let mana = clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Mana)
            .unwrap() as u16;
        let additional = clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Additional)
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: mana });
        act(&mut g, Action::ChooseCastCost { clause: additional });
        let object = g.object_ref(binder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        assert_eq!(g.object_view(binder).zone, Zone::Graveyard);
        assert_eq!(g.object_view(grant).zone, Zone::Battlefield);
        assert!(convoke::available(&g, p, definition));
        act(&mut g, Action::ChooseCastCost { clause: mana });
        for &(id, contribution) in &donors {
            let object = g.object_ref(id);
            act(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        act(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert!(donors.iter().all(|(id, _)| g.object_view(*id).tapped));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn additional_cost_cannot_sacrifice_the_only_convoke_grant_before_mana() {
    const CREATURE_GRANT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..GRANT
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CREATURE_GRANT)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let grant = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
    }
    for _ in 0..4 {
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
    }
    let spell = put(&mut g, p, "Arbiter of Woe", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::BeginConvoke);
    let additional = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause: additional });
    let object = g.object_ref(grant);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    assert_eq!(g.object_view(grant).zone, Zone::Battlefield);
}
