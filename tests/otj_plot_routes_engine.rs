//! Real casts and private choices which produce the same exact plotted designation.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const STACK_PLOT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::ExileTargetSpellPlotted],
    ),
    ..CardRules::EMPTY
};

const RESOLUTION_PLOT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::MulticoloredInstantSorcery,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCastPlotOnResolution],
    }],
    ..CardRules::EMPTY
};

const LOOK_PLOT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 3,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::ExcludingKinds(2),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Hand,
            },
            look: LibraryLook::Fixed(3),
            destination: LibraryPickDestination::ExilePlotted,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn advance_to(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("plot scenario did not reach the expected decision");
}

#[test]
fn stack_exile_plots_an_uncounterable_card_for_its_owner_but_never_a_copy() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &STACK_PLOT)]);
    let caster = Player::P0;
    let owner = caster.other();
    for copy in [false, true] {
        let mut game = scenario(caster);
        let spell = put(&mut game, owner, "Sphinx of the Final Word", Zone::Exile);
        game.grant_exile_play_permission(caster, game.object_ref(spell), false);
        act(&mut game, Action::AnnounceExile { card: spell });
        pay(&mut game);
        let template = game.state.stack.last().unwrap().clone();
        assert_eq!(template.controller, caster);
        assert!(
            game.observe(owner)
                .uncounterable_spell_ids
                .contains(&template.id)
        );
        let target = if copy {
            let packet = game.allocate_spell_copy(&template, caster);
            let id = packet.id;
            game.state.stack.push(packet);
            Target::Stack(id)
        } else {
            Target::Spell(spell)
        };

        act(&mut game, Action::Pass);
        let exiler = put(&mut game, owner, "Shock", Zone::Hand);
        cast(&mut game, exiler, Some(target));
        settle(&mut game);
        if copy {
            assert!(game.observe(owner).plotted_cards.is_empty());
            assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        } else {
            let view = game.observe(owner);
            assert_eq!(view.plotted_cards.len(), 1);
            assert_eq!(view.plotted_cards[0].owner, owner);
            assert_eq!(view.plotted_cards[0].object, game.object_ref(spell));
            assert_eq!(game.object_view(spell).zone, Zone::Exile);
            assert!(!game.has_plotted_permission(owner, spell));
            game.state.turn += 1;
            assert!(game.has_plotted_permission(owner, spell));
            assert!(!game.has_plotted_permission(caster, spell));
        }
    }
}

#[test]
fn the_resolving_player_can_choose_plot_or_another_exile_replacement() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &RESOLUTION_PLOT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let helix = put(&mut game, player, "Lightning Helix", Zone::Hand);
    cast(&mut game, helix, Some(Target::Player(player.other())));
    advance_to(&mut game, |game| {
        matches!(game.state.pending, Pending::ZoneReplacement { .. })
    });
    let choice = game.observe(player).zone_replacement_choice.unwrap();
    assert_eq!(choice.choices.len(), 2);
    assert!(choice.choices.contains(&ZoneReplacementKind::Plot));
    assert!(choice.choices.contains(&ZoneReplacementKind::Exile));
    assert!(
        game.observe(player.other())
            .zone_replacement_choice
            .is_none()
    );

    let checkpoint = game.snapshot();
    let mut plain = game.clone();
    let mut indexed = game.clone();
    let plot = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Plot,
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == plot)
        .unwrap();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, plot);
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(
        &mut plain,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Exile,
        },
    );
    for branch in [&mut game, &mut plain] {
        assert_eq!(branch.object_view(helix).zone, Zone::Exile);
        assert_eq!(branch.state.players[player.index()].life, 23);
        assert_eq!(branch.state.players[player.other().index()].life, 17);
        assert!(branch.observe(player).plot_on_resolution.is_empty());
    }
    assert_eq!(
        game.observe(player).plotted_cards[0].object,
        game.object_ref(helix)
    );
    assert!(plain.observe(player).plotted_cards.is_empty());
    indexed.restore(&checkpoint).unwrap();
    assert!(
        indexed
            .legal_actions()
            .contains(&Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::Exile
            })
    );
}

#[test]
fn resolving_plot_never_marks_nonhand_monocolored_or_unsuccessful_spells() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &RESOLUTION_PLOT)]);
    let player = Player::P0;
    for case in ["counter", "illegal", "exile", "monocolored"] {
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let victim = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let spell = put(
            &mut game,
            player,
            if case == "monocolored" {
                "Shock"
            } else {
                "Lightning Helix"
            },
            if case == "exile" {
                Zone::Exile
            } else {
                Zone::Hand
            },
        );
        if case == "exile" {
            game.grant_exile_play_permission(player, game.object_ref(spell), false);
            act(&mut game, Action::AnnounceExile { card: spell });
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
            pay(&mut game);
        } else {
            cast(&mut game, spell, Some(Target::Card(victim)));
        }

        if matches!(case, "counter" | "illegal") {
            advance_to(&mut game, |game| {
                !game.observe(player).plot_on_resolution.is_empty()
            });
            assert_eq!(
                game.observe(player).plot_on_resolution,
                vec![game.object_ref(spell)]
            );
        }
        if case == "illegal" {
            game.move_object(victim, Zone::Hand);
        } else if case == "counter" {
            let negate = put(&mut game, player.other(), "Negate", Zone::Hand);
            advance_to(&mut game, |game| game.actor() == Some(player.other()));
            cast(&mut game, negate, Some(Target::Spell(spell)));
        }
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard, "{case}");
        assert!(game.observe(player).plotted_cards.is_empty(), "{case}");
        assert!(game.observe(player).plot_on_resolution.is_empty(), "{case}");
    }
}

#[test]
fn private_library_choice_plots_only_the_selected_card_and_moves_the_rest_to_hand() {
    let _rules = fixture_rules::activate(&[(card("Quick Study"), &LOOK_PLOT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let unchosen = put(&mut game, player, "Island", Zone::Library);
    let nonland = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, spell, None);
    advance_to(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectLibrarySelection { .. })
    });
    let selection = game.observe(player).library_selection.unwrap();
    assert_eq!(selection.candidates, vec![game.object_ref(nonland)]);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: land });

    let mut decline = game.clone();
    act(&mut decline, Action::FinishLibrarySelection);
    assert!(decline.observe(player).plotted_cards.is_empty());
    assert_eq!(decline.object_view(nonland).zone, Zone::Hand);
    act(&mut game, Action::SelectLibrary { card: nonland });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert_eq!(game.object_view(nonland).zone, Zone::Exile);
    assert_eq!(
        game.observe(player.other()).plotted_cards[0].object,
        game.object_ref(nonland)
    );
    for id in [land, unchosen] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
    assert!(!game.has_plotted_permission(player, nonland));
    game.state.turn += 1;
    assert!(game.has_plotted_permission(player, nonland));
}
