"""Protect the boundary between ordinary regressions and complete-set acceptance."""
import unittest
from scripts.validate import comprehensive_test_passed, native_test_commands


class ValidationTiers(unittest.TestCase):
    def test_zero_matches_ignored_or_different_test_cannot_establish_comprehensive_pass(self):
        summary='test result: ok. {} passed; 0 failed; {} ignored; 0 measured; 1 filtered out; finished in 0.00s\n'
        name='test tmt_entire_pool_and_theme_acceptance ... '
        self.assertTrue(comprehensive_test_passed(name+'ok\n'+summary.format(1,0)))
        self.assertFalse(comprehensive_test_passed(summary.format(0,0)))
        self.assertFalse(comprehensive_test_passed(name+'ignored\n'+summary.format(0,1)))
        self.assertFalse(comprehensive_test_passed('test another_test ... ok\n'+summary.format(1,0)))
        self.assertFalse(comprehensive_test_passed(name+'FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;\n'))

    def test_comprehensive_adds_the_exact_full_matrix_without_replacing_regressions(self):
        for release in [False, True]:
            with self.subTest(release=release):
                ordinary = native_test_commands(release, 'regression')
                complete = native_test_commands(release, 'comprehensive')
                self.assertEqual(len(ordinary), 1)
                self.assertEqual(complete[:1], ordinary)
                command = complete[1]
                self.assertEqual(command[command.index('--test') + 1], 'tmt_acceptance')
                self.assertEqual(command[-5:], [
                    'tmt_entire_pool_and_theme_acceptance', '--', '--exact', '--ignored', '--nocapture'])
                self.assertNotIn('--include-ignored', command)
                self.assertNotIn('tmt_optimized_throughput', command)

    def test_regression_has_no_runtime_filter_or_target_omission(self):
        optimized = native_test_commands(True, 'regression')[0]
        self.assertEqual(optimized[:4], ['cargo', 'test', '--profile', 'release-test'])
        self.assertEqual(set(optimized[4:]), {'--lib', '--bins', '--tests', '--benches'})
        self.assertEqual(native_test_commands(False, 'regression'), [['cargo', 'test', '--all-targets']])


if __name__ == '__main__':
    unittest.main()
