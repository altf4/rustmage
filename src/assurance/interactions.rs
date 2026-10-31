use super::*;

#[test]
fn growth_in_response_saves_a_creature_and_expires_at_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        act(&mut g, Action::Pass);
        cast(&mut g, shock, Some(Target::Card(bear)));
        act(&mut g, Action::Pass);
        cast(&mut g, growth, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(stats(&g, bear), (5, 5));
        assert_eq!(g.object_view(bear).damage, 2);
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
        assert_eq!(g.object_view(bear).damage, 0);
    }
}

#[test]
fn bounced_target_is_a_new_incarnation_and_old_removal_fizzles() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(bear);
    let murder = put(&mut g, p.other(), "Murder", Zone::Hand);
    let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
    act(&mut g, Action::Pass);
    cast(&mut g, murder, Some(Target::Card(bear)));
    act(&mut g, Action::Pass);
    cast(&mut g, bounce, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Hand);
    assert_ne!(g.object_ref(bear), old);
    assert!(
        g.observe(p.other())
            .known_hand
            .contains(&g.object_ref(bear))
    );
    cast(&mut g, bear, None);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_ne!(g.object_ref(bear), old);
    assert_eq!(g.object_view(murder).zone, Zone::Graveyard);
}

#[test]
fn negate_only_counters_noncreature_spells() {
    for creature in [false, true] {
        let mut g = scenario(Player::P0);
        let spell = put(
            &mut g,
            Player::P0,
            if creature { "Bear Cub" } else { "Shock" },
            Zone::Hand,
        );
        let negate = put(&mut g, Player::P1, "Negate", Zone::Hand);
        cast(
            &mut g,
            spell,
            if creature {
                None
            } else {
                Some(Target::Player(Player::P1))
            },
        );
        act(&mut g, Action::Pass);
        let target = Target::Spell(spell);
        if creature {
            assert!(!g.legal_actions().iter().any(
                |a| matches!(a,Action::Cast{card,..}|Action::AnnounceCast{card} if *card==negate)
            ));
        } else {
            cast(&mut g, negate, Some(target));
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(spell).zone,
            if creature {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(g.state.players[1].life, 20);
    }
}

#[test]
fn royal_role_protects_then_ward_remains_after_hexproof_expires() {
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let treatment = put(&mut g, Player::P0, "Royal Treatment", Zone::Hand);
    let murder = put(&mut g, Player::P1, "Murder", Zone::Hand);
    cast(&mut g, treatment, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(stats(&g, bear), (3, 3));
    assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
    act(&mut g, Action::Pass);
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..}if *card==murder&&*t==bear)
    ));
    g.finish_cleanup();
    g.state.players[1].mana = [8; 6];
    g.state.priority = Player::P1;
    assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
    assert_eq!(stats(&g, bear), (3, 3));
    announce(&mut g, murder, &[Target::Card(bear)]);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    choose(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
}

#[test]
fn suppression_aura_disables_mana_and_bounce_restores_printed_abilities() {
    let mut g = scenario(Player::P0);
    let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let cold = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    announce(&mut g, cold, &[Target::Card(elf)]);
    settle(&mut g);
    assert!(g.object_view(elf).tapped);
    assert!(!g.object_view(elf).abilities.unwrap().printed);
    g.state.objects[elf as usize].tapped = false;
    reject(&mut g, Action::TapMana { card: elf });
    let bounce = put(&mut g, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Card(elf)));
    settle(&mut g);
    assert_eq!(g.object_view(cold).zone, Zone::Graveyard);
    cast(&mut g, elf, None);
    settle(&mut g);
    assert!(g.object_view(elf).abilities.is_none_or(|a| a.printed));
    reject(&mut g, Action::TapMana { card: elf }); // summoning sickness after recast
}

#[test]
fn creature_mana_ability_respects_sickness_and_cost() {
    let mut g = scenario(Player::P0);
    let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Hand);
    cast(&mut g, elf, None);
    settle(&mut g);
    reject(&mut g, Action::TapMana { card: elf });
    g.state.objects[elf as usize].summoning_sick = false;
    let before = g.state.players[0].mana[4];
    act(&mut g, Action::TapMana { card: elf });
    assert_eq!(g.state.players[0].mana[4], before + 1);
    assert!(g.object_view(elf).tapped);
}

#[test]
fn arena_upkeep_draw_is_private_and_life_loss_is_public() {
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Phyrexian Arena", Zone::Battlefield);
    let secret = put(&mut g, Player::P0, "Shock", Zone::Library);
    g.begin_step(Step::Upkeep);
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 19);
    assert_eq!(g.object_view(secret).zone, Zone::Hand);
    assert!(g.observe(Player::P1).objects.iter().all(|o| o.id != secret));
    assert!(g.observe(Player::P0).objects.iter().any(|o| o.id == secret));
}

#[test]
fn ajanis_pridemate_triggers_from_actual_lifelink_damage() {
    let mut g = scenario(Player::P0);
    let mate = put(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    let hawk = put(&mut g, Player::P0, "Healer's Hawk", Zone::Battlefield);
    combat(&mut g, hawk, None);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.counter_count(mate, CounterKind::PlusOne), 1);
    assert_eq!(stats(&g, mate), (3, 3));
}

fn combat(g: &mut Game, attacker: CardId, blocker: Option<CardId>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    for _ in 0..8 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
    if let Some(blocker) = blocker {
        act(g, Action::Block { blocker, attacker });
    }
    act(g, Action::FinishBlockers);
    for _ in 0..32 {
        if g.state.step == Step::Main2 {
            return;
        }
        if g.state.pending == Pending::Priority {
            act(g, Action::Pass);
        } else {
            choose(g, |a| !matches!(a, Action::Concede));
        }
    }
    panic!("32-action combat budget exhausted");
}

#[test]
fn vigilance_flying_and_reach_change_legal_combat() {
    let mut g = scenario(Player::P0);
    let angel = put(&mut g, Player::P0, "Serra Angel", Zone::Battlefield);
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let archer = put(&mut g, Player::P1, "Thornweald Archer", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: angel });
    act(&mut g, Action::FinishAttackers);
    assert!(!g.object_view(angel).tapped);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    reject(
        &mut g,
        Action::Block {
            blocker: bear,
            attacker: angel,
        },
    );
    act(
        &mut g,
        Action::Block {
            blocker: archer,
            attacker: angel,
        },
    );
}

#[test]
fn deathtouch_kills_larger_blocker_and_lifelink_uses_actual_damage() {
    let mut g = scenario(Player::P0);
    let hawk = put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    let blocker = put(&mut g, Player::P1, "Serra Angel", Zone::Battlefield);
    combat(&mut g, hawk, Some(blocker));
    assert_eq!(g.object_view(hawk).zone, Zone::Graveyard);
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn first_strike_removes_blocker_before_it_deals_damage() {
    let mut g = scenario(Player::P0);
    let knight = put(&mut g, Player::P0, "Brightblade Stoat", Zone::Battlefield);
    let blocker = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    combat(&mut g, knight, Some(blocker));
    assert_eq!(g.object_view(knight).zone, Zone::Battlefield);
    assert_eq!(g.object_view(knight).damage, 0);
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
}

#[test]
fn doubling_season_replaces_counters_and_tokens() {
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Snakeskin Veil", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 2);
    assert_eq!(stats(&g, bear), (4, 4));
    let token_spell = put(&mut g, Player::P0, "Dragon Fodder", Zone::Hand);
    cast(&mut g, token_spell, None);
    settle(&mut g);
    let tokens = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
        .count();
    assert_eq!(tokens, 4);
}

#[test]
fn damage_replacement_doubles_enemy_damage_only() {
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Twinflame Tyrant", Zone::Battlefield);
    for (target, expected) in [(Player::P1, 16), (Player::P0, 18)] {
        let spell = put(&mut g, Player::P0, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(target)));
        settle(&mut g);
        assert_eq!(g.state.players[target.index()].life, expected);
    }
}

#[test]
fn damage_death_replacement_persists_until_cleanup() {
    for cleanup in [false, true] {
        let mut g = scenario(Player::P0);
        let mammoth = put(&mut g, Player::P1, "Aggressive Mammoth", Zone::Battlefield);
        let fire = put(&mut g, Player::P0, "Fiery Annihilation", Zone::Hand);
        announce(&mut g, fire, &[Target::Card(mammoth)]);
        settle(&mut g);
        assert_eq!(g.object_view(mammoth).damage, 5);
        if cleanup {
            g.finish_cleanup();
            g.state.players[0].mana = [8; 6];
            g.state.active = Player::P0;
            g.state.priority = Player::P0;
            g.state.step = Step::Main1;
        }
        let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(mammoth)));
        settle(&mut g);
        assert_eq!(
            g.object_view(mammoth).zone,
            if cleanup {
                Zone::Graveyard
            } else {
                Zone::Exile
            }
        );
    }
}

#[test]
fn graveyard_cost_reduction_keeps_the_colored_requirement() {
    let mut g = scenario(Player::P0);
    for name in ["Shock", "Giant Growth", "Divination"] {
        put(&mut g, Player::P0, name, Zone::Graveyard);
    }
    put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let terror = put(&mut g, Player::P0, "Tolarian Terror", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 3];
    cast(&mut g, terror, None);
    settle(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(stats(&g, terror), (5, 5));
}

#[test]
fn modal_spell_has_distinct_target_legality_and_food_payload() {
    for mode in 0..3 {
        let mut g = scenario(Player::P0);
        let flyer = put(&mut g, Player::P1, "Healer's Hawk", Zone::Battlefield);
        let ground = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let enchantment = put(&mut g, Player::P1, "Phyrexian Arena", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Pawpatch Formation", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode });
        if mode == 0 {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ground),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(flyer),
                },
            );
        } else if mode == 1 {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enchantment),
                },
            );
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.object_view(flyer).zone,
            if mode == 0 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(enchantment).zone,
            if mode == 1 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        if mode == 2 {
            assert_eq!(g.hand_count(Player::P0), 1);
            assert_eq!(
                g.observe(Player::P0)
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
                    .count(),
                1
            );
        }
    }
}

#[test]
fn bite_reads_current_power_after_a_response_and_does_not_fight_back() {
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let mammoth = put(&mut g, Player::P1, "Aggressive Mammoth", Zone::Battlefield);
    let bite = put(&mut g, Player::P0, "Bite Down", Zone::Hand);
    announce(&mut g, bite, &[Target::Card(bear), Target::Card(mammoth)]);
    let growth = put(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, growth, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(g.object_view(mammoth).damage, 5);
    assert_eq!(g.object_view(bear).damage, 0);
}

#[test]
fn adventure_resolves_to_exile_and_casts_the_same_physical_creature() {
    let mut g = scenario(Player::P0);
    let creature = put(
        &mut g,
        Player::P0,
        "Hollow Scavenger // Bakery Raid",
        Zone::Hand,
    );
    let original = g.object_ref(creature);
    act(&mut g, Action::AnnounceAdventure { card: creature });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Exile);
    assert!(g.can_play_adventure(Player::P0, creature));
    assert_ne!(g.object_ref(creature), original);
    assert_eq!(
        g.observe(Player::P0)
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        1
    );
    act(&mut g, Action::AnnounceExile { card: creature });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(stats(&g, creature), (3, 2));
    assert!(!g.can_play_adventure(Player::P0, creature));
}

#[test]
fn omen_shuffles_physical_card_without_adventure_permission() {
    let mut g = scenario(Player::P0);
    let creature = put(
        &mut g,
        Player::P0,
        "Sagu Wildling // Roost Seek",
        Zone::Hand,
    );
    act(&mut g, Action::AnnounceOmen { card: creature });
    pay(&mut g);
    for _ in 0..6 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    choose(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Library);
    assert!(!g.can_play_adventure(Player::P0, creature));
    assert_eq!(g.hand_count(Player::P0), 1);
}

#[test]
fn tutor_hides_library_choices_then_reveals_only_the_selected_card() {
    let mut g = scenario(Player::P0);
    let wanted = put(&mut g, Player::P0, "Shock", Zone::Library);
    let hidden = put(&mut g, Player::P0, "Bear Cub", Zone::Library);
    let mage = put(&mut g, Player::P0, "Micromancer", Zone::Hand);
    cast(&mut g, mage, None);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    if g.legal_actions()
        .contains(&Action::ChooseOptional { accept: true })
    {
        act(&mut g, Action::ChooseOptional { accept: true });
    }
    assert!(g.observe(Player::P1).search_choice.is_none());
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|o| o.id != wanted && o.id != hidden)
    );
    reject(&mut g, Action::ChooseSearch { card: hidden });
    act(&mut g, Action::ChooseSearch { card: wanted });
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert!(
        g.observe(Player::P1)
            .known_hand
            .contains(&g.object_ref(wanted))
    );
    assert!(g.observe(Player::P1).objects.iter().all(|o| o.id != hidden));
}

#[test]
fn simultaneous_deaths_use_departed_trigger_sources_and_controller() {
    let mut g = scenario(Player::P0);
    let garna = put(
        &mut g,
        Player::P0,
        "Garna, Bloodfist of Keld",
        Zone::Battlefield,
    );
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let sage = put(&mut g, Player::P0, "Infestation Sage", Zone::Battlefield);
    let foe = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let sweep = put(&mut g, Player::P0, "Day of Judgment", Zone::Hand);
    cast(&mut g, sweep, None);
    for _ in 0..24 {
        if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
            break;
        }
        if g.state.pending == Pending::Priority {
            act(&mut g, Action::Pass);
        } else {
            choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        }
    }
    for id in [garna, bear, sage, foe] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.state.players[1].life, 18);
    let tokens: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token())
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!((tokens[0].power, tokens[0].toughness), (1, 1));
    assert!(tokens[0].keywords.contains(Keywords::FLYING));
}

#[test]
fn base_setting_preserves_counters_and_growth_in_layer_order() {
    let mut g = scenario(Player::P0);
    let angel = put(&mut g, Player::P0, "Serra Angel", Zone::Battlefield);
    let counter = put(&mut g, Player::P0, "Snakeskin Veil", Zone::Hand);
    cast(&mut g, counter, Some(Target::Card(angel)));
    settle(&mut g);
    let growth = put(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, growth, Some(Target::Card(angel)));
    settle(&mut g);
    let aura = put(&mut g, Player::P0, "Eaten by Piranhas", Zone::Hand);
    announce(&mut g, aura, &[Target::Card(angel)]);
    settle(&mut g);
    assert_eq!(stats(&g, angel), (5, 5)); // 1/1 base +1 counter +3 growth
    assert!(!g.keywords(angel).contains(Keywords::FLYING));
    g.finish_cleanup();
    assert_eq!(stats(&g, angel), (2, 2));
}

#[test]
fn printed_colored_costs_cannot_be_paid_with_generic_mana() {
    for name in ["Shock", "Giant Growth", "Unsummon", "Murder", "Serra Angel"] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 8];
        reject(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    }
}

#[test]
fn trample_with_deathtouch_assigns_one_lethal_and_the_rest_to_defender() {
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    let blocker = put(&mut g, Player::P1, "Serra Angel", Zone::Battlefield);
    let overrun = put(&mut g, Player::P0, "Overrun", Zone::Hand);
    cast(&mut g, overrun, None);
    settle(&mut g);
    assert_eq!(stats(&g, attacker), (5, 6));
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: attacker });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..6 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(&mut g, Action::Block { blocker, attacker });
    act(&mut g, Action::FinishBlockers);
    for _ in 0..6 {
        if g.state.pending == Pending::Damage {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Damage);
    reject(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(blocker),
            amount: 0,
        },
    );
    act(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(blocker),
            amount: 1,
        },
    );
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(g.object_view(attacker).zone, Zone::Battlefield);
    assert_eq!(g.state.players[1].life, 16);
    assert_eq!(g.state.players[0].life, 25);
}

#[test]
fn indestructible_survives_destroy_and_menace_requires_two_blockers() {
    let mut g = scenario(Player::P0);
    let zet = put(
        &mut g,
        Player::P0,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, murder, Some(Target::Card(zet)));
    settle(&mut g);
    assert_eq!(g.object_view(zet).zone, Zone::Battlefield);
    let attacker = put(&mut g, Player::P0, "Ravine Raider", Zone::Battlefield);
    assert!(g.keywords(attacker).contains(Keywords::MENACE));
    let a = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: attacker });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..6 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::Block {
            blocker: a,
            attacker,
        },
    );
    reject(&mut g, Action::FinishBlockers);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker,
        },
    );
    act(&mut g, Action::FinishBlockers);
}

#[test]
fn bug_hunt_giada_does_not_apply_to_valkyries_return_effect() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Valkyrie's Call", Zone::Battlefield);
    put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut g, p, "Murder", Zone::Hand);
    cast(&mut g, murder, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert!(g.has_subtype(bear, "Angel"));
    assert_eq!(
        g.counter_count(bear, CounterKind::PlusOne),
        1,
        "only Valkyrie's Call's counter applies"
    );
}

#[test]
fn giada_valkyrie_return_separates_entry_replacement_from_angel_trigger() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Valkyrie's Call", Zone::Battlefield);
            put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
            let observer = put(&mut g, p, "Youthful Valkyrie", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let before = g.object_ref(bear);
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            cast(&mut g, murder, Some(Target::Card(bear)));
            act(&mut g, Action::Pass);
            let actor = g.actor().unwrap();
            let index = g
                .legal_actions()
                .iter()
                .position(|action| *action == Action::Pass)
                .unwrap();
            let checkpoint = g.snapshot();
            let mut indexed = g.clone();
            let mut replay = g.clone();
            act(&mut g, Action::Pass);
            indexed
                .apply_index(actor, indexed.revision(), index)
                .unwrap();
            big_helpers::settle(&mut g);
            big_helpers::settle(&mut indexed);
            assert_eq!(g.snapshot(), indexed.snapshot());
            replay.restore(&checkpoint).unwrap();
            act(&mut replay, Action::Pass);
            big_helpers::settle(&mut replay);
            assert_eq!(g.snapshot(), replay.snapshot());
            assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
            assert_ne!(g.object_ref(bear), before);
            assert!(g.has_subtype(bear, "Angel"));
            assert!(g.keywords(bear).contains(Keywords::FLYING));
            let factor = if doubled { 2 } else { 1 };
            assert_eq!(g.counter_count(bear, CounterKind::PlusOne), factor);
            assert_eq!(
                g.counter_count(observer, CounterKind::PlusOne),
                factor,
                "the post-entry Angel type triggers Youthful Valkyrie"
            );
            // Printed Angels still receive Giada's replacement for all three Angels.
            g.state.players[p.index()].mana = [8; 6];
            let angel = put(&mut g, p, "Serra Angel", Zone::Hand);
            cast(&mut g, angel, None);
            big_helpers::settle(&mut g);
            assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 3 * factor);
        }
    }
}

#[test]
fn bug_hunt_giada_counts_skrull_static_types_on_entry() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
    put(&mut g, p, "Bear Cub", Zone::Graveyard);
    put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let skrull = put(&mut g, p, "Undercover Skrull", Zone::Hand);
    cast(&mut g, skrull, None);
    settle(&mut g);
    assert_eq!(g.object_view(skrull).zone, Zone::Battlefield);
    assert!(g.has_subtype(skrull, "Angel"));
    assert_eq!(
        g.counter_count(skrull, CounterKind::PlusOne),
        1,
        "Giada counts the entering Skrull as an Angel"
    );
    assert_eq!(stats(&g, skrull), (4, 4));
}

#[test]
fn giada_skrull_graveyard_threshold_and_owned_branches() {
    for p in [Player::P0, Player::P1] {
        for own in 0..=2 {
            let mut g = scenario(p);
            put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
            for _ in 0..own {
                put(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            for _ in 0..2 {
                put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
            }
            let skrull = put(&mut g, p, "Undercover Skrull", Zone::Hand);
            cast(&mut g, skrull, None);
            act(&mut g, Action::Pass);
            let actor = g.actor().unwrap();
            let index = g
                .legal_actions()
                .iter()
                .position(|a| *a == Action::Pass)
                .unwrap();
            let checkpoint = g.snapshot();
            let mut indexed = g.clone();
            let mut replay = g.clone();
            act(&mut g, Action::Pass);
            indexed
                .apply_index(actor, indexed.revision(), index)
                .unwrap();
            big_helpers::settle(&mut g);
            big_helpers::settle(&mut indexed);
            assert_eq!(g.snapshot(), indexed.snapshot());
            replay.restore(&checkpoint).unwrap();
            act(&mut replay, Action::Pass);
            big_helpers::settle(&mut replay);
            assert_eq!(g.snapshot(), replay.snapshot());
            let eligible = own >= 2;
            assert_eq!(g.has_subtype(skrull, "Angel"), eligible);
            assert_eq!(
                g.counter_count(skrull, CounterKind::PlusOne),
                u64::from(eligible)
            );
            assert_eq!(stats(&g, skrull), if eligible { (4, 4) } else { (1, 1) });
        }
    }
}

#[test]
fn giada_skrull_return_does_not_count_itself_in_graveyard() {
    for p in [Player::P0, Player::P1] {
        for other_creatures in [1, 2] {
            let mut g = scenario(p);
            put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
            for _ in 0..other_creatures {
                put(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            let skrull = put(&mut g, p, "Undercover Skrull", Zone::Graveyard);
            let reanimate = put(&mut g, p, "Zombify", Zone::Hand);
            cast(&mut g, reanimate, Some(Target::Card(skrull)));
            big_helpers::settle(&mut g);
            let eligible = other_creatures >= 2;
            assert_eq!(g.object_view(skrull).zone, Zone::Battlefield);
            assert_eq!(g.has_subtype(skrull, "Angel"), eligible);
            assert_eq!(
                g.counter_count(skrull, CounterKind::PlusOne),
                u64::from(eligible)
            );
        }
    }
}

#[test]
fn giada_still_counts_printed_and_changeling_angels() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        let changeling = put(&mut g, p, "Barkform Harvester", Zone::Hand);
        cast(&mut g, changeling, None);
        big_helpers::settle(&mut g);
        assert!(g.has_subtype(changeling, "Angel"));
        assert_eq!(g.counter_count(changeling, CounterKind::PlusOne), 1);
        g.state.players[p.index()].mana = [8; 6];
        let angel = put(&mut g, p, "Serra Angel", Zone::Hand);
        cast(&mut g, angel, None);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 2);
    }
}
