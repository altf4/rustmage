"""Paid split destinations and full genuine bodies through the public library."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'examples'))
import standard_split_search as split
from rustmage import Game
# The runner replaces Game.__init__ with an explicit immutable-library constructor.
# A None argument here preserves the normal binding default for ordinary local use.
class SplitSearchTests(split.SplitStandard):
 def test_paid_omen_zero_one_two_choices_both_seats(self):
  for found in range(3):
   for p in ['P0','P1']:
    with self.subTest(found=found,seat=p):self.scenario('omen',p,2690+found,library=None,terminal=False,found=found)
 def test_paid_troop_attack_then_funded_tap_sacrifice_search_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(seat=p):self.scenario('troop',p,2693,library=None,terminal=False)
 def test_paid_bloomvine_self_own_other_opponent_dragon_then_attack_both_seats(self):
  for p in ['P0','P1']:
   with self.subTest(seat=p):self.scenario('normal',p,2694,library=None,terminal=False)
if __name__=='__main__':
 import unittest
 unittest.main()
