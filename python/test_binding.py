import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from rustmage import EngineError, Game


class BindingTests(unittest.TestCase):
    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def vital_index(self, obs):
        own={o['id']:o for o in obs['objects']}
        def score(a):
            kind=a['kind']
            if kind=='AnnounceCast':
                return 85 if own[a['card']]['card'] in ("Ajani's Pridemate",'Agent Venom') else 80
            return {'Keep':100,'PlayLand':90,'PayCosts':100,
                    'TapMana':70 if obs['decision']=='pay_costs' else -5,
                    'Activate':60,'AnnounceActivate':60,'PutTrigger':60,'Sacrifice':60,
                    'FinishAttackers':40,'FinishBlockers':40,'Pass':20,
                    'Mulligan':-100,'Concede':-1000}.get(kind,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def resolve_vital_top(self, game, viewer):
        for _ in range(2):
            obs=game.observe(viewer)
            obs=game.observe(obs['actor'])
            game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass'))
        return game.observe(viewer)

    def test_life_gain_trigger_and_public_history_clone_restore(self):
        deck=['Plains']*24+["Ajani's Pridemate"]*12+['Cat Collector']*12+['Brightblade Stoat']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                if obs['outcome'] is not None: self.fail('game ended before gain trigger')
                obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=="Ajani's Pridemate" and 'TriggeredAbility' in obs['stack'][-1]['kind']: break
                obs=game.step(obs,self.vital_index(obs))
            else: self.fail('no life-gain trigger reached')
            source=obs['stack'][-1];viewer=source['controller'];p=0 if viewer=='P0' else 1
            self.assertEqual(obs['schema'],21)
            self.assertGreater(obs['players'][p]['life_gain_events_this_turn'],0)
            other='P1' if viewer=='P0' else 'P0'
            self.assertEqual(game.observe(other)['players'][p]['life_gain_events_this_turn'],obs['players'][p]['life_gain_events_this_turn'])
            card=source['card'];before=next(o['counters']['plus_one'] for o in obs['objects'] if o['id']==card)
            saved=game.snapshot()
            with game.clone() as branch: expected=self.resolve_vital_top(branch,viewer)
            actual=self.resolve_vital_top(game,viewer)
            self.assertEqual(actual,expected)
            self.assertEqual(next(o['counters']['plus_one'] for o in actual['objects'] if o['id']==card),before+1)
            game.restore(saved)
            replay=self.resolve_vital_top(game,viewer)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_death_trigger_native_continuation_clones_and_restores(self):
        deck=['Swamp']*24+['Agent Venom']*12+['Voracious Vermin']*12+['Abyssal Gorestalker']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(5000):
                if obs['outcome'] is not None: self.fail('game ended before death trigger')
                obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Agent Venom' and 'TriggeredAbility' in obs['stack'][-1]['kind']: break
                obs=game.step(obs,self.vital_index(obs))
            else: self.fail('no death trigger reached')
            source=obs['stack'][-1];viewer=source['controller'];p=0 if viewer=='P0' else 1
            self.assertTrue(any(o['zone']=='Graveyard' for o in obs['objects']))
            saved=game.snapshot()
            with game.clone() as branch: expected=self.resolve_vital_top(branch,viewer)
            actual=self.resolve_vital_top(game,viewer)
            self.assertEqual(actual,expected)
            self.assertEqual(actual['players'][p]['library_count'],obs['players'][p]['library_count']-1)
            self.assertEqual(actual['players'][p]['life'],obs['players'][p]['life']-1)
            game.restore(saved)
            replay=self.resolve_vital_top(game,viewer)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_private_discard_payment_paid_or_declined_clones_and_restores(self):
        deck=["Mountain"]*30+["Discerning Peddler"]*30
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(1000):
                obs=game.observe(obs["actor"])
                if obs["decision"]=="effect_choice": break
                index=max(range(len(obs["actions"])),key=lambda i:
                    {"Keep":100,"PlayLand":90,"AnnounceCast":80,"PayCosts":100,
                     "TapMana":70 if obs["decision"]=="pay_costs" else -5,"Pass":20,
                     "FinishAttackers":40,"FinishBlockers":40,"Mulligan":-100,"Concede":-1000}
                    .get(obs["actions"][i]["kind"],0))
                obs=game.step(obs,index)
            else: self.fail("no discard cost reached")
            actor=obs["actor"]; player=0 if actor=="P0" else 1; other="P1" if actor=="P0" else "P0"
            self.assertEqual(obs["effect_choice"]["kind"],"Discard")
            self.assertTrue(obs["effect_choice"]["optional"])
            self.assertIsNone(game.observe(other)["effect_choice"])
            self.assertEqual(game.observe(other)["actions"],[])
            index=next(i for i,a in enumerate(obs["actions"]) if a["kind"]=="Discard")
            card=obs["actions"][index]["card"];saved=game.snapshot()
            with game.clone() as branch: expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index)
            self.assertEqual(actual,expected)
            self.assertEqual(actual["players"][player]["library_count"],obs["players"][player]["library_count"]-1)
            self.assertTrue(any(o["id"]==card and o["zone"]=="Graveyard" for o in game.observe(other)["objects"]))
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,"StaleRevision"): game.step(obs,index)
            replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"));self.assertEqual(replay,actual)
            game.restore(saved);current=game.observe(actor)
            decline=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="DeclineEffectPayment")
            declined=game.step(current,decline)
            self.assertEqual(declined["players"][player]["library_count"],obs["players"][player]["library_count"])
            self.assertTrue(any(o["id"]==card and o["zone"]=="Hand" for o in declined["objects"]))
            game.drop_snapshot(saved)

    def test_public_apnap_sacrifices_stay_on_battlefield_until_simultaneous_commit(self):
        deck=["Swamp"]*30+["Agents of HYDRA"]*20+["Abyssal Gorestalker"]*10
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(5000):
                if obs["outcome"] is not None: self.fail("game ended before simultaneous sacrifice")
                obs=game.observe(obs["actor"])
                choice=obs["effect_choice"]
                if choice and not choice["optional"] and min(choice["required"])>=2: break
                objects={o["id"]:o for o in obs["objects"]}
                def score(a):
                    if a["kind"]=="AnnounceCast": return 85 if objects[a["card"]]["card"]=="Abyssal Gorestalker" else 80
                    return {"Keep":100,"PlayLand":90,"PayCosts":100,
                            "TapMana":70 if obs["decision"]=="pay_costs" else -5,
                            "Sacrifice":60,"PutTrigger":60,"Discard":30,"Pass":20,
                            "FinishAttackers":40,"FinishBlockers":40,"Mulligan":-100,"Concede":-1000}.get(a["kind"],0)
                obs=game.step(obs,max(range(len(obs["actions"])),key=lambda i:score(obs["actions"][i])))
            else: self.fail("no simultaneous sacrifice reached")
            active=obs["active"];player=0 if active=="P0" else 1;other="P1" if active=="P0" else "P0"
            self.assertEqual(obs["actor"],active)
            selected=[]
            for _ in range(2):
                index=next(i for i,a in enumerate(obs["actions"]) if a["kind"]=="Sacrifice")
                selected.append(obs["actions"][index]["object"])
                obs=game.step(obs,index)
                self.assertEqual(game.observe(other)["effect_choice"]["selected"][player],selected)
                self.assertTrue(all(any(o["id"]==ref["card"] and o["zone"]=="Battlefield" for o in obs["objects"]) for ref in selected))
            self.assertEqual(obs["actor"],other)
            saved=game.snapshot()
            def finish(handle):
                for _ in range(3):
                    current=handle.observe(other)
                    if current["effect_choice"] is None: return current
                    current=handle.observe(current["actor"])
                    index=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="Sacrifice")
                    handle.step(current,index)
                self.fail("sacrifice did not commit")
            with game.clone() as branch: expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected)
            self.assertTrue(all(any(o["id"]==ref["card"] and o["zone"]=="Graveyard" for o in actual["objects"]) for ref in selected))
            game.restore(saved);replay=finish(game)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_survival_step_trigger_history_and_source_tapped_clone_restore(self):
        deck=["Forest"]*30+["Cautious Survivor"]*30
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(1600):
                if obs["outcome"] is not None: self.fail("game ended before survival trigger")
                obs=game.observe(obs["actor"])
                actor=obs["actor"]
                if obs["step"]=="Main2" and obs["decision"]=="priority" and any(x["source_card"]=="Cautious Survivor" for x in obs["stack"]):
                    break
                index=max(range(len(obs["actions"])),key=lambda i:
                    {"Keep":100,"PlayLand":90,"AnnounceCast":80,"PayCosts":100,
                     "TapMana":70 if obs["decision"]=="pay_costs" else -5,
                     "PutTrigger":70,"Attack":60,"Block":-5,"Pass":20,
                     "Discard":30,"Mulligan":-100,"Concede":-1000}.get(obs["actions"][i]["kind"],0))
                obs=game.step(obs,index)
            else: self.fail("no survival trigger reached")
            player=0 if actor=="P0" else 1
            self.assertTrue(obs["players"][player]["attacked_this_turn"])
            self.assertTrue(obs["stack"][-1]["source_tapped"])
            other="P1" if actor=="P0" else "P0"
            self.assertTrue(game.observe(other)["players"][player]["attacked_this_turn"])
            saved=game.snapshot()
            def resolve(handle):
                for _ in range(2):
                    current=handle.observe(actor)
                    current=handle.observe(current["actor"])
                    index=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="Pass")
                    handle.step(current,index)
                return handle.observe(actor)
            with game.clone() as branch: expected=resolve(branch)
            actual=resolve(game)
            self.assertEqual(actual,expected)
            self.assertEqual(actual["players"][player]["life"],obs["players"][player]["life"]+2)
            game.restore(saved)
            replay=resolve(game)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"))
            self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_completed_cast_counts_and_triggers_survive_native_clone_restore(self):
        deck = ["Mountain"] * 24 + ["Firebrand Archer"] * 24 + ["Boltwave"] * 12
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1200):
                obs = game.observe(obs["actor"])
                if obs["outcome"] is not None:
                    self.fail("game ended before a cast-trigger check")
                actor = obs["actor"]
                player = 0 if actor == "P0" else 1
                own = {o["id"]: o for o in obs["objects"] if o["owner"] == actor}
                def score(a):
                    kind = a["kind"]
                    if kind == "AnnounceCast":
                        return 85 if own[a["card"]]["card"] == "Firebrand Archer" else 80
                    return {"Keep":100, "PlayLand":90, "PayCosts":100,
                            "TapMana":70 if obs["decision"] == "pay_costs" else -5,
                            "PutTrigger":60, "FinishAttackers":60, "FinishBlockers":60,
                            "Pass":50, "Mulligan":-100, "Concede":-1000}.get(kind, 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                action = obs["actions"][index]
                if (action["kind"] == "PayCosts" and obs["announcing"]["source_card"] == "Boltwave"
                        and any(o["card"] == "Firebrand Archer" and o["zone"] == "Battlefield" for o in own.values())):
                    before = obs["players"][player]["spells_cast_this_turn"]
                    saved = game.snapshot()
                    with game.clone() as branch:
                        expected = branch.step(branch.observe(actor), index)
                    actual = game.step(obs, index)
                    self.assertEqual(actual, expected)
                    self.assertEqual(actual["players"][player]["spells_cast_this_turn"], before + 1)
                    other = "P1" if actor == "P0" else "P0"
                    self.assertEqual(game.observe(other)["players"][player]["spells_cast_this_turn"], before + 1)
                    self.assertEqual(actual["stack"][0]["source_card"], "Boltwave")
                    self.assertTrue(any(x["source_card"] == "Firebrand Archer" for x in actual["stack"] + actual["waiting_triggers"]))
                    game.restore(saved)
                    replay = game.step(game.observe(actor), index)
                    self.assertGreater(replay.pop("revision"), actual.pop("revision"))
                    self.assertEqual(replay, actual)
                    game.drop_snapshot(saved)
                    break
                obs = game.step(obs, index)
            else:
                self.fail("no cast trigger exercised")

    def test_legend_choices_public_masks_and_native_clone_restore(self):
        deck = ["Swamp"] * 30 + ["Yargle, Glutton of Urborg"] * 30
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(3000):
                obs = game.observe(obs["actor"])
                if obs["decision"] == "legend_rule":
                    break
                # No attacks: keep both legends alive long enough to exercise the rule.
                index = max(range(len(obs["actions"])), key=lambda i:
                    {"Keep": 100, "PlayLand": 90, "AnnounceCast": 80, "Cast": 80, "PayCosts": 100,
                     "TapMana": 70 if obs["decision"] == "pay_costs" else -5,
                     "FinishAttackers": 60, "FinishBlockers": 60,
                     "Pass": 50, "Attack": -50, "Mulligan": -100, "Concede": -1000}
                    .get(obs["actions"][i]["kind"], 0))
                obs = game.step(obs, index)
            else:
                self.fail("no legend decision reached")
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            self.assertEqual(obs["legend_choices"], game.observe(other)["legend_choices"])
            self.assertEqual({a["kind"] for a in obs["actions"]}, {"KeepLegend", "Concede"})
            self.assertEqual(game.observe(other)["actions"], [])
            index = next(i for i,a in enumerate(obs["actions"]) if a["kind"] == "KeepLegend")
            kept = obs["actions"][index]["object"]["card"]
            saved = game.snapshot()
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            self.assertEqual(next(o for o in actual["objects"] if o["id"] == kept)["zone"], "Battlefield")
            self.assertEqual(actual["legend_choices"], [])
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            self.assertGreater(replay.pop("revision"), actual.pop("revision"))
            self.assertEqual(replay, actual)
            game.drop_snapshot(saved)

    def test_private_views_clone_restore_and_stale_indices(self):
        with self.game() as game:
            observation = game.observe("P0")
            coverage = json.loads((Path(__file__).resolve().parents[1] / "data/standard/coverage.json").read_text())
            self.assertEqual(observation["catalogue"], coverage["catalogue_version"])
            self.assertEqual(observation["schema"], 21)
            self.assertEqual(len(observation["objects"]), 7)
            self.assertTrue(all(o["owner"] == "P0" for o in observation["objects"]))
            self.assertEqual(game.observe("P1")["actions"], [])
            saved = game.snapshot()
            with game.clone() as branch:
                keep = next(i for i, a in enumerate(observation["actions"]) if a["kind"] == "Keep")
                branch.step(observation, keep)
                self.assertEqual(game.observe("P0"), observation)
                self.assertEqual(branch.observe("P1")["actor"], "P1")
            game.restore(saved)
            restored = game.observe("P0")
            self.assertGreater(restored.pop("revision"), observation["revision"])
            before = dict(observation)
            before.pop("revision")
            self.assertEqual(restored, before)
            with self.assertRaisesRegex(EngineError, "StaleRevision"):
                game.step(observation, 1)
            game.drop_snapshot(saved)
            with self.assertRaisesRegex(EngineError, "unknown snapshot"):
                game.restore(saved)

    def test_distinct_target_groups_power_damage_and_partial_restore(self):
        deck = ["Forest"] * 24 + ["Grizzly Bears"] * 18 + ["Quarrel"] * 18
        with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1800):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["targeting_group"] == 0 and obs["announcing"]["source_card"] == "Quarrel":
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": 79,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, index)
            else:
                self.fail("no distinct target declaration reached")
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            selected = []
            for group in (0, 1):
                self.assertEqual(obs["targeting_group"], group)
                hidden = game.observe(other)
                self.assertIsNone(hidden["targeting_group"])
                self.assertEqual(hidden["announcing"]["targets"], [])
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "ChooseTarget")
                chosen = obs["actions"][index]["target"]["Card"]
                obj = next(o for o in obs["objects"] if o["id"] == chosen)
                self.assertEqual(obj["owner"], actor if group == 0 else other)
                selected.append(chosen)
                saved = game.snapshot()
                with game.clone() as branch:
                    expected = branch.step(branch.observe(actor), index)
                actual = game.step(obs, index)
                self.assertEqual(actual, expected)
                game.restore(saved)
                replay = game.step(game.observe(actor), index)
                comparison = dict(replay)
                comparison.pop("revision")
                actual.pop("revision")
                self.assertEqual(actual, comparison)
                game.drop_snapshot(saved)
                obs = replay
            for _ in range(20):
                kinds = [a["kind"] for a in obs["actions"]]
                if "PayCosts" in kinds:
                    obs = game.step(obs, kinds.index("PayCosts"))
                    break
                obs = game.step(obs, kinds.index("TapMana"))
            else:
                self.fail("no completed group payment")
            item = next(i for i in obs["stack"] if i["source_card"] == "Quarrel")
            self.assertEqual([b["group"] for b in item["targets"]], [0, 1])
            for _ in range(2):
                obs = game.observe(obs["actor"])
                obs = game.step(obs, next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Pass"))
            view = game.observe(actor)
            source = next(o for o in view["objects"] if o["id"] == selected[0])
            victim = next(o for o in view["objects"] if o["id"] == selected[1])
            self.assertEqual(source["zone"], "Battlefield")
            self.assertEqual(source["damage"], 0)
            self.assertEqual(victim["zone"], "Graveyard")

    def test_modal_cast_mode_targets_payment_and_resolution_replay(self):
        fixtures = [(["Mountain"] * 24 + ["Abrade"] * 18 + ["Hill Giant"] * 18, "Abrade", 0),
                    (["Island"] * 36 + ["Spellgyre"] * 24, "Spellgyre", 1)]
        for deck, name, mode in fixtures:
            with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
                obs = game.observe("P0")
                for _ in range(1800):
                    self.assertIsNone(obs["outcome"])
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if obs["decision"] == "spell_mode":
                        break
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                        return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": 79,
                                "Mulligan": -100, "Concede": -1000}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                    obs = game.step(obs, index)
                else:
                    self.fail("no mode selection reached: " + name)
                actor = obs["actor"]
                other = "P1" if actor == "P0" else "P0"
                self.assertEqual(obs["announcing"]["source_card"], name)
                self.assertIsNone(obs["announcing"]["mode"])
                self.assertTrue(obs["mode_choices"])
                self.assertFalse(game.observe(other)["mode_choices"])
                self.assertFalse(game.observe(other)["actions"])
                saw_target = False
                saw_payment = False
                for _ in range(30):
                    kinds = [a["kind"] for a in obs["actions"]]
                    if "ChooseMode" in kinds:
                        index = next(i for i, a in enumerate(obs["actions"])
                                     if a["kind"] == "ChooseMode" and a["mode"] == mode)
                    elif "ChooseTarget" in kinds:
                        index = kinds.index("ChooseTarget")
                        saw_target = True
                    elif "PayCosts" in kinds:
                        index = kinds.index("PayCosts")
                        saw_payment = True
                    elif "TapMana" in kinds:
                        index = kinds.index("TapMana")
                    else:
                        break
                    saved = game.snapshot()
                    with game.clone() as branch:
                        expected = branch.step(branch.observe(actor), index)
                    actual = game.step(obs, index)
                    self.assertEqual(actual, expected)
                    game.restore(saved)
                    replay = game.step(game.observe(actor), index)
                    comparison = dict(replay)
                    comparison.pop("revision")
                    actual.pop("revision")
                    self.assertEqual(actual, comparison)
                    game.drop_snapshot(saved)
                    obs = replay
                self.assertTrue(saw_payment)
                self.assertEqual(saw_target, name == "Abrade")
                item = next(item for item in obs["stack"] if item["source_card"] == name)
                self.assertEqual(item["mode"], mode)
                for _ in range(2):
                    obs = game.observe(obs["actor"])
                    index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Pass")
                    obs = game.step(obs, index)
                if name == "Spellgyre":
                    obs = game.observe(actor)
                    self.assertIsNotNone(obs["library_choice"])
                    self.assertEqual(obs["resolving"]["mode"], 1)
                    self.assertIsNone(game.observe(other)["library_choice"])

    def test_search_private_selection_public_destination_and_clone_restore(self):
        for source in ("Evolving Wilds", "World Map"):
            deck = ["Forest"] * 36 + [source] * 24
            with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
                obs = game.observe("P0")
                for _ in range(1200):
                    self.assertIsNone(obs["outcome"])
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if obs["search_choice"] is not None:
                        break
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                        return {"Keep": 100, "PlayLand": 90, "Cast": 80, "Activate": 70,
                                "AnnounceCast": -20, "AnnounceActivate": -20,
                                "Mulligan": -100, "Concede": -1000}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                    obs = game.step(obs, index)
                else:
                    self.fail("no search reached: " + source)
                actor = obs["actor"]
                other = "P1" if actor == "P0" else "P0"
                choice = obs["search_choice"]
                self.assertEqual(choice["filter"], "BasicLand")
                self.assertTrue(choice["cards"])
                self.assertTrue(all(o["zone"] == "Library" for o in choice["cards"]))
                hidden = game.observe(other)
                self.assertIsNone(hidden["search_choice"])
                self.assertFalse(hidden["actions"])
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "ChooseSearch")
                selected = obs["actions"][index]["card"]
                obs = game.step(obs, index)
                self.assertEqual(obs["search_choice"]["selected"], [selected])
                self.assertIsNone(game.observe(other)["search_choice"])
                self.assertFalse(any(o["id"] == selected for o in game.observe(other)["objects"]))
                saved = game.snapshot()
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "FinishSearch")
                with game.clone() as branch:
                    expected = branch.step(branch.observe(actor), index)
                actual = game.step(obs, index)
                self.assertEqual(actual, expected)
                obj = next(o for o in game.observe(other)["objects"] if o["id"] == selected)
                if source == "World Map":
                    self.assertEqual(obj["zone"], "Hand")
                    self.assertIn(obj["reference"], actual["known_hand"])
                else:
                    self.assertEqual(obj["zone"], "Battlefield")
                    self.assertTrue(obj["tapped"])
                game.restore(saved)
                replay = game.step(game.observe(actor), index)
                replay.pop("revision")
                actual.pop("revision")
                self.assertEqual(actual, replay)
                game.drop_snapshot(saved)

    def test_nonbasic_land_entry_private_surveil_and_mana_replay(self):
        deck = ["Commercial District"] * 60
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            seen_entry = False
            for _ in range(100):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["library_choice"] is not None:
                    break
                index = max(range(len(obs["actions"])), key=lambda i:
                    {"Keep": 100, "PlayLand": 90, "Mulligan": -100, "Concede": -1000}
                    .get(obs["actions"][i]["kind"], 0))
                action = obs["actions"][index]
                if action["kind"] == "PlayLand":
                    actor = obs["actor"]
                    obs = game.step(obs, index)
                    land = next(o for o in obs["objects"] if o["id"] == action["card"])
                    self.assertTrue(land["tapped"])
                    self.assertFalse(any(a.get("card") == land["id"] and a["kind"] == "ActivateMana"
                                         for a in obs["actions"]))
                    seen_entry = True
                else:
                    obs = game.step(obs, index)
            else:
                self.fail("no land surveil choice reached")
            self.assertTrue(seen_entry)
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            self.assertTrue(obs["library_choice"]["surveil"])
            self.assertIsNone(game.observe(other)["library_choice"])
            saved = game.snapshot()
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Graveyard")
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            replay.pop("revision")
            actual.pop("revision")
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)
            obs = game.observe(actor)
            # Reach the next untap step without tapping away the available source.
            for _ in range(200):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                actions = [(i, a) for i, a in enumerate(obs["actions"])
                           if a["kind"] == "ActivateMana"]
                if actions:
                    break
                index = next(i for i, a in enumerate(obs["actions"])
                             if a["kind"] in ("Pass", "FinishAttackers", "FinishBlockers", "Keep",
                                              "Discard", "Bottom", "ArrangeLibrary"))
                obs = game.step(obs, index)
            else:
                self.fail("land never untapped")
            self.assertEqual({tuple(a["production"]) for _, a in actions},
                             {(0, 0, 0, 1, 0, 0), (0, 0, 0, 0, 1, 0)})
            saved = game.snapshot()
            index = actions[0][0]
            with game.clone() as branch:
                expected = branch.step(branch.observe(obs["actor"]), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            self.assertEqual(sum(actual["players"][0 if obs["actor"] == "P0" else 1]["mana"]), 1)
            game.restore(saved)
            replay = game.step(game.observe(obs["actor"]), index)
            replay.pop("revision")
            actual.pop("revision")
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)

    def test_scry_private_library_cards_and_partial_choice_restore(self):
        deck = ["Plains"] * 36 + ["Wakandan Drone Flock"] * 24
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(600):
                self.assertIsNone(obs["outcome"], "game ended before the scry fixture")
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["library_choice"] is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, index)
            else:
                self.fail("no scry choice reached")
            choice = obs["library_choice"]
            self.assertFalse(choice["surveil"])
            self.assertEqual(len(choice["remaining"]), 2)
            self.assertTrue(all(o["zone"] == "Library" for o in choice["remaining"]))
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            hidden = game.observe(other)
            self.assertIsNone(hidden["library_choice"])
            self.assertEqual(hidden["actions"], [])
            visible_ids = {o["id"] for o in hidden["objects"]}
            self.assertTrue(all(o["id"] not in visible_ids for o in choice["remaining"]))
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Bottom")
            obs = game.step(obs, index)
            self.assertEqual(len(obs["library_choice"]["bottom"]), 1)
            selected = obs["library_choice"]["bottom"][0]
            self.assertEqual(selected, next(o for o in choice["remaining"] if o["id"] == selected["id"]))
            self.assertEqual(selected["zone"], "Library")
            self.assertIsNone(game.observe(other)["library_choice"])
            saved = game.snapshot()
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Top")
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            self.assertGreater(replay.pop("revision"), actual.pop("revision"))
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)

    def test_derived_anthem_stats_clone_and_restore(self):
        deck = ["Plains"] * 20 + ["Forest"] * 20 + ["Anthem of Champions"] * 10 + ["Grizzly Bears"] * 10
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1200):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                creatures = {o["id"]: (o["power"], o["toughness"]) for o in objects.values()
                             if o["zone"] == "Battlefield" and o["owner"] == obs["actor"] and o["card"] == "Grizzly Bears"}
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Anthem of Champions"), None)
                if index is not None and creatures:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("anthem/creature setup not reached")
            actor = obs["actor"]
            def cast_and_resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe(actor)["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe(actor)
            saved = game.snapshot()
            with game.clone() as branch:
                expected = cast_and_resolve(branch, branch.observe(actor))
            after = cast_and_resolve(game, obs)
            self.assertEqual(after, expected)
            by_id = {o["id"]: o for o in after["objects"]}
            for card, (power, toughness) in creatures.items():
                self.assertEqual((by_id[card]["power"], by_id[card]["toughness"]), (power + 1, toughness + 1))
            game.restore(saved)
            again = cast_and_resolve(game, game.observe(actor))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_effective_keyword_gains_replay_through_native_library(self):
        deck = ["Forest"] * 36 + ["Grizzly Bears"] * 12 + ["Overprotect"] * 12
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1000):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Overprotect"), None)
                if index is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("keyword fixture not reached")
            actor = obs["actor"]
            target = obs["actions"][index]["target"]["Card"]
            before = objects[target]
            def resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe(actor)["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe(actor)
            saved = game.snapshot()
            with game.clone() as branch:
                expected = resolve(branch, branch.observe(actor))
            after = resolve(game, obs)
            self.assertEqual(after, expected)
            protected = next(o for o in after["objects"] if o["id"] == target)
            self.assertEqual(protected["keywords"] & (256 + 4096 + 8192), 256 + 4096 + 8192)
            self.assertEqual((protected["power"], protected["toughness"]), (before["power"] + 3, before["toughness"] + 3))
            game.restore(saved)
            again = resolve(game, game.observe(actor))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_menace_partial_blocks_require_completion_and_restore(self):
        decks = [["Swamp"] * 36 + ["Ravine Raider"] * 24, ["Forest"] * 36 + ["Grizzly Bears"] * 24]
        with Game(decks, seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1500):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Block" and obs["actor"] == "P1"), None)
                if index is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20, "Attack": 60,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("menace block fixture not reached")
            attacker = obs["actions"][index]["attacker"]
            obs = game.step(obs, index)
            self.assertFalse(any(a["kind"] == "FinishBlockers" for a in obs["actions"]))
            self.assertEqual(game.observe("P0")["blocks"], [])
            saved = game.snapshot()
            second = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Block" and a["attacker"] == attacker)
            def complete(engine, observation):
                current = engine.step(observation, second)
                finish = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "FinishBlockers")
                return engine.step(current, finish)
            with game.clone() as branch:
                expected = complete(branch, branch.observe("P1"))
            after = complete(game, obs)
            self.assertEqual(after, expected)
            self.assertEqual(len(after["blocks"]), 2)
            game.restore(saved)
            again = complete(game, game.observe("P1"))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_public_linked_exile_returns_and_replays_through_library(self):
        decks = [["Plains"] * 32 + ["Banishing Light"] * 12 + ["Disenchant"] * 16,
                 ["Forest"] * 36 + ["Grizzly Bears"] * 24]
        with Game(decks, seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1600):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                source = obs["exile_links"][0]["source"]["card"] if obs["exile_links"] else None
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Disenchant" and a["target"] == {"Card": source}), None)
                if index is not None and obs["actor"] == "P0":
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20, "ChooseTarget": 70,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("linked exile fixture not reached")
            link = next(link for link in obs["exile_links"] if link["source"]["card"] == source)
            target = link["exiled"]["card"]
            self.assertEqual(link["return_zone"], "Battlefield")
            self.assertEqual(objects[target]["zone"], "Exile")
            self.assertEqual(game.observe("P1")["exile_links"], obs["exile_links"])
            saved = game.snapshot()
            def resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe("P0")["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe("P0")
            with game.clone() as branch:
                expected = resolve(branch, branch.observe("P0"))
            after = resolve(game, obs)
            self.assertEqual(after, expected)
            returned = next(o for o in after["objects"] if o["id"] == target)
            self.assertEqual((returned["zone"], returned["owner"]), ("Battlefield", "P1"))
            self.assertNotEqual(returned["reference"], link["exiled"])
            self.assertFalse(any(other["source"]["card"] == source for other in after["exile_links"]))
            game.restore(saved)
            again = resolve(game, game.observe("P0"))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_input_and_lifecycle(self):
        with self.assertRaises(EngineError):
            Game([["BlackLotus"] * 60] * 2)
        with self.assertRaises(EngineError):
            Game([["LightningBolt"] * 60] * 2)
        game = self.game()
        with self.assertRaises(EngineError):
            game.request("unknown")
        game.close()
        game.close()
        with self.assertRaisesRegex(EngineError, "closed"):
            game.observe("P0")

    def test_standard_admission_distinguishes_legality_and_rule_coverage(self):
        deck = ["Plains"] * 56 + ["Savannah Lions"] * 4
        with Game([deck, deck], deck_rules="Standard") as game:
            self.assertEqual(game.observe("P0")["players"][0]["library_count"], 53)
        with self.assertRaisesRegex(EngineError, "not Standard legal"):
            Game([["Mountain"] * 56 + ["Lightning Bolt"] * 4] * 2, deck_rules="Standard")
        with self.assertRaisesRegex(EngineError, "rules not implemented"):
            Game([["Hare Apparent"] * 60] * 2, deck_rules="Standard")

    def test_decoded_nul_in_invalid_card_is_an_error_without_aborting_host(self):
        # A regression in this boundary can abort the interpreter, so exercise it
        # in a child and assert normal exit rather than crashing the whole suite.
        result = subprocess.run([sys.executable, "-c", """
from rustmage import EngineError, Game
try:
    Game([["bad\\x00"] * 60] * 2)
except EngineError as error:
    assert "unknown card" in str(error)
else:
    raise AssertionError("invalid card accepted")
"""], env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parent)}, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_token_metadata_and_pending_resolution_replay_through_shared_library(self):
        deck = ["Mountain"] * 10 + ["Dragon Fodder"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(200):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                kind = obs["actions"][index]["kind"]
                obs = game.step(obs, index)
                if kind == "Cast":
                    break
                self.assertIsNone(obs["outcome"])
            else:
                self.fail("token spell was never cast")
            saved = game.snapshot()
            def resolve(handle):
                for _ in range(2):
                    current = handle.observe("P0")
                    current = handle.observe(current["actor"])
                    index = next(i for i, action in enumerate(current["actions"]) if action["kind"] == "Pass")
                    handle.step(current, index)
                result = handle.observe("P0")
                result.pop("revision")
                return result
            with game.clone() as branch:
                expected = resolve(branch)
            actual = resolve(game)
            self.assertEqual(actual, expected)
            tokens = [o for o in actual["objects"] if o["token"] is not None]
            self.assertEqual(len(tokens), 2)
            self.assertTrue(all(o["token"] == {"name": "Goblin Token", "subtypes": "Goblin", "colors": 8}
                                for o in tokens))
            self.assertTrue(all(o["summoning_sick"] for o in tokens))
            self.assertTrue(all(o["card"].startswith("@token/") for o in tokens))
            game.restore(saved)
            self.assertEqual(resolve(game), expected)
            with self.assertRaisesRegex(EngineError, "tokens cannot"):
                Game([[tokens[0]["card"]] * 60] * 2, deck_rules="Freeform")

    def test_activated_self_sacrifice_has_independent_stack_identity_and_replay(self):
        deck = ["Mountain"] * 10 + ["Fanatical Firebrand"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(100):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] == "Main1" else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80,
                            "Concede": -1000, "Mulligan": -100}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                kind = obs["actions"][index]["kind"]
                game.step(obs, index)
                if kind == "Cast":
                    break
            else:
                self.fail("Firebrand never cast")
            def pass_twice():
                for _ in range(2):
                    current = game.observe("P0")
                    current = game.observe(current["actor"])
                    game.step(current, next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass"))
            pass_twice()
            obs = game.observe("P0")
            obs = game.observe(obs["actor"])
            opponent = "P1" if obs["actor"] == "P0" else "P0"
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "Activate" and a["target"] == {"Player": opponent})
            action = obs["actions"][index]
            self.assertEqual(action["counter_payment"], {"plus_one": 0, "minus_one": 0, "stun": 0})
            source = next(o for o in obs["objects"] if o["id"] == action["card"])
            saved = game.snapshot()
            game.step(obs, index)
            pending = game.observe("P0")
            self.assertEqual(pending["stack"][-1]["kind"], {"ActivatedAbility": {"ability": 0}})
            self.assertEqual(pending["stack"][-1]["object"], source["reference"])
            self.assertEqual(next(o for o in pending["objects"] if o["id"] == action["card"])["zone"], "Graveyard")
            pass_twice()
            expected = game.observe("P0")
            expected.pop("revision")
            self.assertEqual(expected["players"][0 if opponent == "P0" else 1]["life"], 19)
            game.restore(saved)
            current = game.observe(obs["actor"])
            game.step(current, current["actions"].index(action))
            pass_twice()
            actual = game.observe("P0")
            actual.pop("revision")
            self.assertEqual(actual, expected)

    def test_treasure_mana_is_immediate_and_six_slot_pool_replays(self):
        deck = ["Mountain"] * 16 + ["Plundering Pirate"] * 4
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(1000):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                choices = [(i,a) for i,a in enumerate(obs["actions"])
                           if a["kind"] == "ActivateMana" and a["production"] == [0,0,0,0,1,0]]
                if choices:
                    index, action = choices[0]
                    break
                def score(a):
                    if a["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] == "Main1" else -5
                    return {"Keep":100,"PlayLand":90,"Cast":80,"Concede":-1000,"Mulligan":-100}.get(a["kind"],0)
                game.step(obs, max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i])))
            else:
                self.fail("Treasure mana ability never available")
            actor = obs["actor"]
            self.assertTrue(all(len(p["mana"]) == 6 for p in obs["players"]))
            self.assertEqual(next(o for o in obs["objects"] if o["id"] == action["card"])["token"]["name"], "Treasure Token")
            before_mana = obs["players"][0 if actor == "P0" else 1]["mana"][4]
            saved = game.snapshot()
            game.step(obs, index)
            expected = game.observe(actor)
            expected.pop("revision")
            self.assertEqual(expected["stack"], obs["stack"])
            self.assertEqual(expected["actor"], actor)
            self.assertEqual(expected["players"][0 if actor == "P0" else 1]["mana"][4], before_mana + 1)
            self.assertFalse(any(o["id"] == action["card"] for o in expected["objects"]))
            game.restore(saved)
            current = game.observe(actor)
            game.step(current, current["actions"].index(action))
            actual = game.observe(actor)
            actual.pop("revision")
            self.assertEqual(actual, expected)

    def test_unfunded_cast_targets_then_funds_in_owned_payment_window(self):
        deck = ["Mountain"] * 10 + ["Lightning Bolt"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(200):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                index = next((i for i,a in enumerate(obs["actions"]) if a["kind"] == "AnnounceCast"), None)
                if index is not None:
                    game.step(obs,index)
                    break
                def score(a):
                    return {"Keep":100,"PlayLand":90,"Concede":-1000,"Mulligan":-100,"TapMana":-5}.get(a["kind"],0)
                game.step(obs,max(range(len(obs["actions"])),key=lambda i: score(obs["actions"][i])))
            else:
                self.fail("unfunded announcement never offered")
            actor = obs["actor"]
            opponent = "P1" if actor == "P0" else "P0"
            current=game.observe(actor)
            index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "ChooseTarget" and a["target"] == {"Player":opponent})
            game.step(current,index)
            current=game.observe(actor)
            self.assertEqual(current["decision"],"pay_costs")
            self.assertEqual(current["payment_request"]["required"],[0,0,0,1,0,0])
            self.assertIsNone(game.observe(opponent)["payment_request"])
            self.assertEqual(game.observe(opponent)["actions"],[])
            saved=game.snapshot()
            def finish(handle):
                current=handle.observe(actor)
                index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "TapMana")
                handle.step(current,index)
                current=handle.observe(actor)
                index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "PayCosts")
                handle.step(current,index)
                self.assertEqual(len(handle.observe(actor)["stack"]),1)
                for _ in range(2):
                    current=handle.observe("P0")
                    current=handle.observe(current["actor"])
                    handle.step(current,next(i for i,a in enumerate(current["actions"]) if a["kind"] == "Pass"))
                result=handle.observe(actor)
                result.pop("revision")
                return result
            with game.clone() as branch:
                expected=finish(branch)
            self.assertEqual(finish(game),expected)
            self.assertEqual(expected["players"][0 if opponent=="P0" else 1]["life"],17)
            game.restore(saved)
            self.assertEqual(finish(game),expected)

    def test_parallel_independent_games(self):
        def run(seed):
            with self.game() as game:
                for _ in range(100):
                    # Observation calls are read-only; independent handles have no global RNG.
                    game.observe("P0")
                return json.dumps(game.observe("P0"), sort_keys=True)
        with concurrent.futures.ThreadPoolExecutor(4) as pool:
            outputs = list(pool.map(run, range(8)))
        self.assertEqual(len(set(outputs)), 1)

    def test_complete_selfplay_through_shared_library(self):
        # The complete control loop stays in this process. Actor changes require
        # a fresh observation for the new player, never a union of both hands.
        with self.game() as game:
            observation = game.observe("P0")
            cast_count = 0
            for _ in range(5000):
                if observation["outcome"] is not None:
                    break
                actor = observation["actor"]
                if observation["viewer"] != actor:
                    observation = game.observe(actor)
                opponent = "P1" if actor == "P0" else "P0"

                def score(action):
                    kind = action["kind"]
                    if kind == "Cast":
                        return 80 if action["target"] == {"Player": opponent} else -10
                    return {"Keep": 100, "PlayLand": 90, "TapMana": 20,
                            "Discard": 40, "Concede": -1000, "Mulligan": -100}.get(kind, 0)

                index = max(range(len(observation["actions"])), key=lambda i: score(observation["actions"][i]))
                cast_count += observation["actions"][index]["kind"] == "Cast"
                observation = game.step(observation, index)
            else:
                self.fail("self-play did not end naturally")
            self.assertGreaterEqual(cast_count, 7)
            self.assertIn("Winner", observation["outcome"])
            self.assertEqual(game.observe("P0")["outcome"], game.observe("P1")["outcome"])

    def test_standard_keyword_combat_through_shared_library(self):
        white = ["Plains"] * 28
        for name in ["Savannah Lions", "Healer's Hawk", "Bishop's Soldier", "Brightblade Stoat",
                     "Leonin Skyhunter", "Serra Angel", "Shrike Force", "Iron Giant"]:
            white.extend([name] * 4)
        green = ["Forest"] * 24
        for name in ["Ankle Biter", "Dragon Sniper", "Magnigoth Sentry", "Colossadactyl",
                     "Tajuru Pathwarden", "Galewind Moose", "Gigantosaurus", "Bear Cub", "Jibbirik Omnivore"]:
            green.extend([name] * 4)
        seen_steps = set()
        with Game([white, green], seed=2, deck_rules="Standard") as game:
            obs = game.observe("P0")
            for _ in range(10000):
                if obs["outcome"] is not None:
                    break
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                seen_steps.add(obs["step"])
                def score(action):
                    if action["kind"] == "AssignDamage":
                        self.assertIn("target", action)
                        if obs["damage_assigned"]:
                            self.assertIn("source", obs["damage_assigned"][0])
                        return 50 + action["amount"]
                    if action["kind"] == "TapMana":
                        return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                    return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                            "Cast": 80, "Attack": 60, "Block": 60,
                            "Discard": 50}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                obs = game.step(obs, index)
            else:
                self.fail("Standard keyword game did not end")
            self.assertIn("FirstStrikeDamage", seen_steps)
            self.assertIn("CombatDamage", seen_steps)
            self.assertIn("Winner", obs["outcome"])

    def test_standard_removal_and_object_identity_through_shared_library(self):
        removal = {"Unsummon", "Fell", "Murder", "Bilbo's Deadly Slice"}
        blue_black = ["Island"] * 12 + ["Swamp"] * 12
        for name in sorted(removal) + ["Vampire Nighthawk", "Hullcarver", "Monoist Sentry",
                                        "Squirrelanoids", "Highborn Vampire"]:
            blue_black.extend([name] * 4)
        green = ["Forest"] * 24
        for name in ["Ankle Biter", "Dragon Sniper", "Magnigoth Sentry", "Colossadactyl",
                     "Tajuru Pathwarden", "Galewind Moose", "Gigantosaurus", "Bear Cub", "Jibbirik Omnivore"]:
            green.extend([name] * 4)
        replayed = False
        with Game([blue_black, green], seed=9, deck_rules="Standard") as game:
            obs = game.observe("P0")
            for _ in range(10000):
                if obs["outcome"] is not None:
                    break
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {obj["id"]: obj for obj in obs["objects"]}
                for obj in objects.values():
                    self.assertEqual(obj["reference"]["card"], obj["id"])
                for spell in obs["stack"]:
                    self.assertEqual(spell["object"], objects[spell["card"]]["reference"])
                def score(action):
                    kind = action["kind"]
                    if kind == "Cast" and action["target"] is not None:
                        target = objects[action["target"]["Card"]]
                        return 85 if target["owner"] != obs["actor"] else -10
                    if kind in ("AnnounceCast", "AnnounceActivate", "AnnounceMana"):
                        return -30  # This fixture checks the original atomic Cast replay.
                    if kind == "TapMana":
                        return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                    if kind == "AssignDamage":
                        return 50 + action["amount"]
                    return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                            "Cast": 80, "Attack": 60, "Block": 60, "Discard": 50}.get(kind, 0)
                index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                action = obs["actions"][index]
                check_replay = (not replayed and action["kind"] == "Cast"
                                and objects[action["card"]]["card"] in removal)
                if check_replay:
                    saved = game.snapshot()
                    after = game.step(obs, index)
                    game.restore(saved)
                    restored = game.observe(obs["viewer"])
                    again = game.step(restored, index)
                    self.assertGreater(again.pop("revision"), after.pop("revision"))
                    self.assertEqual(again, after)
                    game.drop_snapshot(saved)
                    replayed = True
                    obs = game.observe(restored["viewer"])
                else:
                    obs = game.step(obs, index)
            else:
                self.fail("Standard removal game did not end")
            self.assertTrue(replayed)
            self.assertIn("Winner", obs["outcome"])

    def test_standard_trigger_programs_and_pending_choice_replay(self):
        white_blue = ["Plains"] * 12 + ["Island"] * 12
        for name in ["Helpful Hunter", "Inspiring Overseer", "Bellowing Crier", "Buzz Bots",
                     "Summit Sentinel", "Icewind Elemental", "Lifecreed Duo", "Rook Turret", "Cloudblazer"]:
            white_blue.extend([name] * 4)
        black_green = ["Swamp"] * 12 + ["Forest"] * 12
        for name in ["Bogwater Lumaret", "Burglar Rat", "Hecteyes", "Vampire Spawn", "Glidedive Duo",
                     "Virulent Emissary", "Eumidian Terrabotanist", "Pelakka Wurm", "Primordial Pachyderm"]:
            black_green.extend([name] * 4)
        replayed = set()
        seen_trigger = False
        for seed in range(5):
            with Game([white_blue, black_green], seed=seed, deck_rules="Standard") as game:
                obs = game.observe("P0")
                for _ in range(10000):
                    if obs["outcome"] is not None:
                        break
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if any(item["kind"] != "Spell" for item in obs["stack"]):
                        seen_trigger = True
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                        if kind == "AssignDamage": return 50 + action["amount"]
                        if kind == "ChooseOptional": return 50 if action["accept"] else 0
                        return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                                "Cast": 80, "Attack": 60, "Block": 60, "Discard": 50,
                                "PutTrigger": 50}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                    decision = obs["decision"]
                    if decision in {"order_triggers", "optional_effect", "effect_discard"} and decision not in replayed:
                        saved = game.snapshot()
                        with game.clone() as branch:
                            expected = branch.step(obs, index)
                        first = game.step(obs, index)
                        self.assertEqual(first, expected)
                        game.restore(saved)
                        restored = game.observe(obs["viewer"])
                        again = game.step(restored, index)
                        self.assertGreater(again.pop("revision"), expected.pop("revision"))
                        self.assertEqual(again, expected)
                        game.drop_snapshot(saved)
                        replayed.add(decision)
                        obs = game.observe(restored["viewer"])
                    else:
                        obs = game.step(obs, index)
                else:
                    self.fail("Standard trigger game did not finish")
                self.assertIn("Winner", obs["outcome"])
            if len(replayed) == 3:
                break
        self.assertTrue(seen_trigger)
        self.assertEqual(replayed, {"order_triggers", "optional_effect", "effect_discard"})

    def test_standard_target_announcements_and_pending_target_replay(self):
        white_black = ["Plains"] * 12 + ["Swamp"] * 12
        for name in ["Fancy Footwork", "Thistledown Players", "Helpful Hunter", "Inspiring Overseer",
                     "Emerge from the Cocoon", "Skeleton Archer", "Vault Plunderer", "Zombify", "Burglar Rat"]:
            white_black.extend([name] * 4)
        blue_green = ["Island"] * 12 + ["Forest"] * 12
        for name in ["Reclamation Sage", "Iron Giant", "Buzz Bots", "Rook Turret", "Quick Study", "Refute",
                     "Aegis Turtle", "Tajuru Pathwarden", "Magnigoth Sentry"]:
            blue_green.extend([name] * 4)
        replayed = set()
        for seed in range(5):
            with Game([white_black, blue_green], seed=seed, deck_rules="Standard") as game:
                obs = game.observe("P0")
                for _ in range(10000):
                    if obs["outcome"] is not None: break
                    if obs["viewer"] != obs["actor"]: obs = game.observe(obs["actor"])
                    objects = {obj["id"]: obj for obj in obs["objects"]}
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana": return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                        if kind == "AssignDamage": return 50 + action["amount"]
                        if kind == "ChooseOptional": return 50 if action["accept"] else 0
                        if kind == "ChooseTarget":
                            source = obs["announcing"]["source_card"]
                            beneficial = source in {"Fancy Footwork", "Thistledown Players", "Vault Plunderer"}
                            target = action["target"]
                            own = (target.get("Player") == obs["actor"] if "Player" in target
                                   else objects[target["Card"]]["owner"] == obs["actor"])
                            return 70 if own == beneficial else -10
                        if kind == "Cast" and action["target"] is not None:
                            target = action["target"]
                            if "Spell" in target: return -10
                            return 80
                        return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                                "Cast": 80, "BeginCast": 80, "Attack": 60, "Block": 60, "Discard": 50,
                                "PutTrigger": 50}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                    item = obs["announcing"]
                    replay_kind = None if item is None else ("spell" if item["kind"] == "Spell" else "trigger")
                    if obs["decision"] == "targets" and replay_kind not in replayed:
                        saved = game.snapshot()
                        with game.clone() as branch: expected = branch.step(obs, index)
                        self.assertEqual(game.step(obs, index), expected)
                        game.restore(saved)
                        restored = game.observe(obs["viewer"])
                        again = game.step(restored, index)
                        self.assertGreater(again.pop("revision"), expected.pop("revision"))
                        self.assertEqual(again, expected)
                        game.drop_snapshot(saved)
                        replayed.add(replay_kind)
                        obs = game.observe(restored["viewer"])
                    else: obs = game.step(obs, index)
                else: self.fail("Standard targeting game did not end")
                self.assertIn("Winner", obs["outcome"])
            if len(replayed) == 2: break
        self.assertEqual(replayed, {"spell", "trigger"})


if __name__ == "__main__":
    unittest.main()
