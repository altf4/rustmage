//! Fixed layer-five colors in actual and borrowed speculative worlds.
use crate::ecl_scenarios::{cast, named, settle, setup};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingMove, FundingState};
use rustmage_core::engine::*;
fn applied<'g>(world: &FundingState<'g>, step: &FundingMove) -> FundingState<'g> {
    let mut child = None;
    world.visit_applied(step, &mut |next| {
        child = Some(next.clone());
        true
    });
    child.expect("the complete mana transition is legal")
}
fn land_output(world: &FundingState<'_>, goal: &FundingGoal, land: CardId) -> [u128; 6] {
    let step = world
        .moves(goal)
        .into_iter()
        .find(|step| world.resources[step.resource].id == land)
        .unwrap();
    applied(world, &step).mana
}
#[test]
fn ecl_green_fixed_color_timeline_matches_funding_aura_order_expiry_and_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_color_effects(
            &[bear],
            31,
            rustmage_core::cards::AnimationDuration::Permanent,
        );
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: Some((
                g.object_ref(bear),
                AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: evidence::cost(0).filter,
                },
            )),
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(world.colors(bear), 31);
        assert!(world.guard_available(ActivationGuard::ControlColorsAtLeast(5), bear, p));
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).colors, 17);
        g.set_color_effects(
            &[bear],
            2,
            rustmage_core::cards::AnimationDuration::Permanent,
        );
        assert_eq!(g.object_view(bear).colors, 2);
        let mut world = FundingState::from_game(&g, &goal);
        assert_eq!(world.colors(bear), 2);
        assert!(!world.guard_available(ActivationGuard::ControlColorsAtLeast(5), bear, p));
        let index = world.resources.iter().position(|r| r.id == bear).unwrap();
        world.resources[index].object.zone = Zone::Graveyard;
        assert_eq!(
            world.colors(bear),
            Card::from_name("Bear Cub").unwrap().colors()
        );
        g.set_color_effects(
            &[bear],
            31,
            rustmage_core::cards::AnimationDuration::EndOfTurn,
        );
        assert_eq!(g.object_view(bear).colors, 31);
        crate::ecl_scenarios::next_turn(&mut g);
        assert_eq!(g.object_view(bear).colors, 2);
        let original = g.object_ref(bear);
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(g.object_ref(bear), original);
        assert_eq!(g.object_view(bear).colors, 16);
        assert!(
            !g.observe(p)
                .animations
                .iter()
                .any(|row| row.object == original)
        );
        // Mutating a branch's fixed color leaves its parent and restored state intact.
        let snapshot = g.snapshot();
        let mut child = g.clone();
        child.set_color_effects(
            &[bear],
            1,
            rustmage_core::cards::AnimationDuration::Permanent,
        );
        assert_eq!(child.object_view(bear).colors, 1);
        assert_eq!(g.object_view(bear).colors, 16);
        child.restore(&snapshot).unwrap();
        assert_eq!(child.snapshot(), g.snapshot());
    }
}

#[test]
fn ecl_green_fixed_colors_distinguish_otherwise_identical_funding_resource_keys() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Forest", Zone::Battlefield);
        let second = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        g.set_color_effects(
            &[first],
            2,
            rustmage_core::cards::AnimationDuration::Permanent,
        );
        g.set_color_effects(
            &[second],
            8,
            rustmage_core::cards::AnimationDuration::Permanent,
        );
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: None,
        };
        let mut left = FundingState::from_game(&g, &goal);
        let mut right = left.clone();
        left.resources
            .iter_mut()
            .find(|r| r.id == first)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        right
            .resources
            .iter_mut()
            .find(|r| r.id == second)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_ne!(
            ability_world::controlled_colors(&left, p),
            ability_world::controlled_colors(&right, p)
        );
        assert!(left.key(&goal) != right.key(&goal));
    }
}

#[test]
fn ecl_green_wickermaw_funding_color_overrides_are_owned_keyed_and_feed_dynamic_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let wicker = named(&mut g, p, "Foraging Wickermaw", Zone::Battlefield);
        let bloom = named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        g.state.objects[bloom as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let goal = FundingGoal {
            actor: p,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            source: None,
            reserve: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let moves = world.moves(&goal);
        let blue_move = moves
            .iter()
            .find(|step| {
                world.resources[step.resource].id == wicker && step.production == [0, 1, 0, 0, 0, 0]
            })
            .unwrap();
        let red_move = moves
            .iter()
            .find(|step| {
                world.resources[step.resource].id == wicker && step.production == [0, 0, 0, 1, 0, 0]
            })
            .unwrap();
        let mut blue = applied(&world, blue_move);
        let mut red = applied(&world, red_move);
        assert_eq!(world.colors(wicker), 0);
        assert_eq!(blue.colors(wicker), 2);
        assert_eq!(red.colors(wicker), 8);
        assert_eq!(g.object_view(wicker).colors, 0);
        assert!(
            blue.moves(&goal)
                .iter()
                .any(|step| blue.resources[step.resource].id == bloom
                    && step.production == [0, 1, 0, 0, 1, 0])
        );
        assert!(
            red.moves(&goal)
                .iter()
                .any(|step| red.resources[step.resource].id == bloom
                    && step.production == [0, 0, 0, 1, 1, 0])
        );
        blue.mana = [0; 6];
        red.mana = [0; 6];
        assert!(blue.key(&goal) != red.key(&goal));
    }
}

#[test]
fn ecl_green_choice_stat_mana_counts_other_creatures_in_owned_worlds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        let definition = ManaProduction::ChoiceStat {
            colors: 2 | 16,
            amount: &StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: true,
                cards_only: false,
            }),
        };
        fn collect(
            world: &impl CharacteristicWorld,
            source: CardId,
            definition: ManaProduction,
        ) -> Vec<Mana> {
            let mut result = Vec::new();
            definition.in_world(world, source).visit(|mana| {
                result.push(mana);
                false
            });
            result
        }
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            source: None,
            reserve: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(
            collect(&g, source, definition),
            vec![[0, 1, 0, 0, 0, 0], [0, 0, 0, 0, 1, 0]]
        );
        assert_eq!(
            collect(&world, source, definition),
            collect(&g, source, definition)
        );
        let mut branch = world.clone();
        branch
            .resources
            .iter_mut()
            .find(|resource| resource.id == other)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_eq!(collect(&branch, source, definition), vec![[0; 6]]);
        assert_eq!(
            collect(&world, source, definition),
            collect(&g, source, definition)
        );
    }
}

#[test]
fn ecl_green_growth_bonus_funding_is_owned_and_unrestricted_beside_restricted_main_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let aura = crate::ecl_scenarios::enter_growth(&mut g, p, land, 1);
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 1, 0, 0, 1, 0],
            generic: 0,
            source: None,
            reserve: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert_eq!(land_output(&world, &goal, land), [0, 1, 0, 0, 1, 0]);
        assert_eq!(world.colors(land), 2);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| world.resources[step.resource].id == land)
            .unwrap();
        assert_eq!(step.production, [0, 0, 0, 0, 1, 0]);
        let child = applied(&world, &step);
        assert_eq!(child.mana, [0, 1, 0, 0, 1, 0]);
        assert_eq!(world.mana, [0; 6]);
        assert!(world.possible_color_upper()[1] >= 1);
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|resource| resource.id == aura)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_eq!(land_output(&departed, &goal, land), [0, 0, 0, 0, 1, 0]);
        assert_eq!(departed.colors(land), 0);
        assert_eq!(land_output(&world, &goal, land), [0, 1, 0, 0, 1, 0]);
    }
}

#[test]
fn ecl_green_behold_owned_funding_preserves_live_or_hand_witness_but_allows_tapping() {
    for p in [Player::P0, Player::P1] {
        for hand in [false, true] {
            let mut g = setup(p);
            let witness = named(
                &mut g,
                p,
                "Llanowar Elves",
                if hand { Zone::Hand } else { Zone::Battlefield },
            );
            g.state.objects[witness as usize].summoning_sick = false;
            let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
            let spec = g.state.objects[source as usize]
                .card
                .additional_cost()
                .unwrap();
            let goal = FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                source: None,
                reserve: Some((g.object_ref(witness), spec)),
            };
            let world = FundingState::from_game(&g, &goal);
            assert!(world.obligations(&goal));
            let mut tapped = world.clone();
            tapped
                .resources
                .iter_mut()
                .find(|resource| resource.id == witness)
                .unwrap()
                .object
                .tapped = true;
            assert!(tapped.obligations(&goal));
            let mut departed = world.clone();
            departed
                .resources
                .iter_mut()
                .find(|resource| resource.id == witness)
                .unwrap()
                .object
                .zone = Zone::Graveyard;
            assert!(!departed.obligations(&goal));
            assert!(world.obligations(&goal));
            if !hand {
                assert!(!world.guard_available(
                    ActivationGuard::GraveyardSubtype("Elf"),
                    witness,
                    p
                ));
                assert!(departed.guard_available(
                    ActivationGuard::GraveyardSubtype("Elf"),
                    witness,
                    p
                ));
            }
            let mut stale = world.clone();
            stale
                .resources
                .iter_mut()
                .find(|resource| resource.id == witness)
                .unwrap()
                .object
                .generation += 1;
            assert!(!stale.obligations(&goal));
        }
        let mut g = setup(p);
        let token = named(
            &mut g,
            p,
            "@token/shapeshifter_1_1_colorless_changeling",
            Zone::Graveyard,
        );
        let guard = ActivationGuard::GraveyardSubtype("Elf");
        assert!(!g.activation_guard_available(guard, token, p));
        let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
        let spec = g.state.objects[source as usize]
            .card
            .additional_cost()
            .unwrap();
        let battlefield_token = named(
            &mut g,
            p,
            "@token/shapeshifter_1_1_colorless_changeling",
            Zone::Battlefield,
        );
        assert!(g.additional_candidate(
            spec,
            p,
            g.object_ref(source),
            g.object_ref(battlefield_token)
        ));
        g.move_object(battlefield_token, Zone::Hand);
        assert!(!g.additional_candidate(
            spec,
            p,
            g.object_ref(source),
            g.object_ref(battlefield_token)
        ));
        assert!(!g.activation_guard_available(guard, token, p));
        assert!(g.state.objects[token as usize].is_token());
    }
}

#[test]
fn ecl_green_behold_many_provisional_reserves_are_owned_and_distinguish_interchangeable_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creatures: Vec<_> = (0..3)
            .map(|_| {
                let id = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
                g.state.objects[id as usize].summoning_sick = false;
                id
            })
            .collect();
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 1,
            source: None,
            reserve: None,
        };
        let spec = AdditionalCost {
            kind: EffectChoiceKind::BeholdMany(2),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Elf"),
            },
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.cost_group = Some((
            spec,
            creatures[..2].iter().map(|&id| g.object_ref(id)).collect(),
        ));
        assert!(world.obligations(&goal));
        let moves = world.moves(&goal);
        let first = moves
            .iter()
            .find(|step| world.resources[step.resource].id == creatures[0])
            .unwrap();
        let third = moves
            .iter()
            .find(|step| world.resources[step.resource].id == creatures[2])
            .unwrap();
        let mut reserved = applied(&world, first);
        let ordinary = applied(&world, third);
        assert!(reserved.obligations(&goal));
        assert_eq!(reserved.mana, ordinary.mana);
        assert!(reserved.key(&goal) != ordinary.key(&goal));
        let second = reserved
            .moves(&goal)
            .into_iter()
            .find(|step| reserved.resources[step.resource].id == creatures[1])
            .unwrap();
        reserved = applied(&reserved, &second);
        assert!(reserved.obligations(&goal));
        assert_eq!(reserved.mana, [0, 0, 0, 0, 2, 0]);
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|resource| resource.id == creatures[0])
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(!departed.obligations(&goal));
        assert!(world.obligations(&goal));
        departed
            .cost_group
            .as_mut()
            .unwrap()
            .1
            .push(g.object_ref(creatures[2]));
        assert_eq!(world.cost_group.as_ref().unwrap().1.len(), 2);
    }
}
