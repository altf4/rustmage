"""The CI runner must propagate failures and retain progress before completion."""
import io
import json
import unittest
from scripts.python_tests import TimedResult


class PythonRunnerTests(unittest.TestCase):
    def run_case(self, case):
        events = io.StringIO()
        result = unittest.TextTestRunner(stream=io.StringIO(),
            resultclass=lambda *args: TimedResult(*args, events=events)).run(case)
        return result, [json.loads(line) for line in events.getvalue().splitlines()]

    def test_failure_is_reported_in_result_and_timing(self):
        class Failing(unittest.TestCase):
            def runTest(self):
                self.fail('expected test failure')
        result, events = self.run_case(Failing())
        self.assertFalse(result.wasSuccessful())
        self.assertEqual([e['event'] for e in events], ['start', 'finish'])
        self.assertTrue(events[-1]['failed'])
        self.assertGreaterEqual(events[-1]['seconds'], 0)

    def test_subtest_failure_is_not_recorded_as_success(self):
        class Failing(unittest.TestCase):
            def runTest(self):
                with self.subTest(case='bad'):
                    self.fail('expected subtest failure')
        result, events = self.run_case(Failing())
        self.assertFalse(result.wasSuccessful())
        self.assertTrue(events[-1]['failed'])

    def test_success_records_test_identity(self):
        class Passing(unittest.TestCase):
            def runTest(self):
                pass
        case = Passing()
        result, events = self.run_case(case)
        self.assertTrue(result.wasSuccessful())
        self.assertFalse(events[-1]['failed'])
        self.assertEqual(events[0]['test'], case.id())
