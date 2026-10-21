"""Six synthetic machinery checks; no model or assessor calibration claim."""
import unittest

from history import (ContextOverflow, History, Reference, RELOAD_LADDER,
                     RetrievalExhausted, TurnBudget, compact_bytes,
                     deadline_restricted_latency, normalise_usage)


def reference(revision="0"):
    return Reference("entry", "ready", "assessment-" + revision, "source-" + revision,
                     "context", "methods", "2040-01-01T00:00:0" + revision + "Z")


def exchange(call_id="call-1", content="synthetic packet"):
    return [{"role": "assistant", "tool_calls": [{"id": call_id, "type": "function", "function": {
        "name": "eal_assess_known", "arguments": '{"entry_id":"entry","claim":"ready"}'}}]},
        {"role": "tool", "tool_call_id": call_id, "content": content}]


class MachineryCalibration(unittest.TestCase):
    def history(self, condition="C"):
        return History(condition, "immutable brief", {"ready": "release criterion"}, [{"name": "fixed tool"}])

    def test_01_packet_age_eviction_preserves_complete_exchange_and_ordinary_turn(self):
        for condition in "ABC":
            with self.subTest(condition=condition):
                host = self.history(condition)
                for turn in range(1, 5):
                    host.begin_turn("user " + str(turn))
                    if turn == 1:
                        host.add_packet(reference(), exchange())
                    host.finish_turn("answer " + str(turn))
                self.assertTrue(any(block.packet for block in host.turns[0].blocks))
                host.begin_turn("user 5")
                messages = host.request()["messages"]
                self.assertIn({"role": "user", "content": "user 1"}, messages)
                self.assertIn({"role": "assistant", "content": "answer 1"}, messages)
                remaining_calls = [m for m in messages if m.get("tool_calls")]
                remaining_results = [m for m in messages if m["role"] == "tool"]
                self.assertEqual(len(remaining_calls), 1 if condition == "A" else 0)
                self.assertEqual(len(remaining_results), len(remaining_calls))

    def test_02_known_omission_requires_explicit_calls_and_exhausts_ladder(self):
        host = self.history()
        host.begin_turn("need omitted detail")
        host.add_packet(reference(), exchange(content='{"summary_complete":false}'))
        calls = []

        def retrieve(ref, limits):
            calls.append((ref, limits))
            return {"summary_complete": False, "omitted": {"arguments": 2}}

        self.assertEqual(calls, [])  # The host never auto-restores missing detail.
        for _ in RELOAD_LADDER:
            self.assertFalse(host.reload(reference(), retrieve)["summary_complete"])
        self.assertEqual([limits for _, limits in calls], list(RELOAD_LADDER))
        with self.assertRaises(RetrievalExhausted):
            host.reload(reference(), retrieve)
        self.assertEqual(len(calls), 3)
        host.finish_turn("qualified synthetic placeholder, not a scored answer")
        host.begin_turn("next turn")
        host.reload(reference(), retrieve)
        self.assertEqual(calls[-1][1], RELOAD_LADDER[0])
        with self.assertRaises(PermissionError):
            self.history("A").cache_lookup(reference())
        broken = self.history()
        broken.begin_turn("failed retrieval is still a request")
        broken.add_packet(reference(), exchange())

        def failing(ref, limits):
            raise OSError("synthetic transport failure")

        for _ in RELOAD_LADDER:
            with self.assertRaises(OSError):
                broken.reload(reference(), failing)
        with self.assertRaises(RetrievalExhausted):
            broken.reload(reference(), failing)
        self.assertEqual(broken.server_reloads, 3)

    def test_03_revision_cache_and_index_preserve_original_identity_without_semantic_summary(self):
        for condition in ("A", "B"):
            restricted = self.history(condition)
            restricted.begin_turn("original default packet policy")
            with self.assertRaises(ValueError):
                restricted.add_packet(reference(), exchange(), RELOAD_LADDER[0])
        host = self.history("B")
        original = exchange(content="historical semantic payload")
        host.begin_turn("original")
        host.add_packet(reference(), original)
        host.finish_turn("answer")
        host.begin_turn("revision")
        host.add_packet(reference("1"), exchange("call-2", "revised payload"))
        self.assertEqual(host.cache_lookup(reference()), original)
        returned = host.cache_lookup(reference())
        returned[-1]["content"] = "mutation"
        self.assertEqual(host.cache_lookup(reference()), original)
        index_message = host.request()["messages"][2]["content"]
        self.assertNotIn("historical semantic payload", index_message)
        self.assertNotIn("revised payload", index_message)
        self.assertIn(reference().assessed_at, index_message)
        self.assertEqual(len(host.index), 2)
        self.assertEqual(self.history("A")._cache, {})
        self.assertEqual(self.history("C")._cache, {})

    def test_04_emergency_eviction_preserves_protected_index_and_current_turn(self):
        host = self.history("A")
        host.begin_turn("old question")
        host.add_packet(reference(), exchange(content="x" * 10000))
        host.finish_turn("old answer")
        host.begin_turn("current question")
        before = host.request()
        request = host.prepare(compact_bytes, compact_bytes(before) - 5000)
        self.assertIn("immutable brief", str(request))
        self.assertIn("current question", str(request))
        self.assertIn("reference_index", str(request))
        self.assertNotIn("old answer", str(request))
        self.assertNotIn("old question", str(request))
        self.assertNotIn("tool_call_id", str(request))
        with self.assertRaises(ContextOverflow):
            host.prepare(compact_bytes, 1)
        with self.assertRaises(ValueError):
            host.prepare(lambda request: None, 1000)

    def test_05_timeout_and_cycle_exhaustion_keep_failure_penalty(self):
        budget = TurnBudget(0.0)
        for _ in range(6):
            budget.before_cycle(1.0)
        with self.assertRaises(RetrievalExhausted):
            budget.before_cycle(2.0)
        timed = TurnBudget(10.0)
        with self.assertRaises(TimeoutError):
            timed.before_cycle(130.0)
        self.assertEqual(deadline_restricted_latency(None, completed=False), 120)
        self.assertEqual(deadline_restricted_latency(2.0, completed=False), 120)
        self.assertEqual(deadline_restricted_latency(2.0, completed=True), 2)

    def test_06_missing_telemetry_stays_missing_and_cached_reasoning_fields_do_not_double_count(self):
        missing = normalise_usage(None, output_includes_reasoning=True)
        self.assertIsNone(missing["total_tokens"])
        self.assertIsNone(missing["input_tokens"])
        zero = normalise_usage({"input_tokens": 0, "output_tokens": 0}, output_includes_reasoning=True)
        self.assertEqual(zero["total_tokens"], 0)
        usage = {"input_tokens": 100, "output_tokens": 20, "cached_input_tokens": 60, "reasoning_tokens": 5}
        self.assertEqual(normalise_usage(usage, output_includes_reasoning=True)["total_tokens"], 120)
        self.assertEqual(normalise_usage(usage, output_includes_reasoning=False)["total_tokens"], 125)
        self.assertIsNone(normalise_usage({"input_tokens": 100}, output_includes_reasoning=True)["total_tokens"])
        with self.assertRaises(ValueError):
            normalise_usage({"input_tokens": 1, "cached_input_tokens": 2}, output_includes_reasoning=True)


if __name__ == "__main__":
    unittest.main()
