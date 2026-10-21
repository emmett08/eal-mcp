"""Small, provider-independent host policy for EAL-CONTEXT-001.

This module does not call a model or turn byte lengths into token measurements.
The request counter passed to ``prepare`` must cover the complete request,
including tools, protected instructions, reference index and current turn.
"""
from __future__ import annotations

from copy import deepcopy
from dataclasses import asdict, dataclass, field
import json
from typing import Callable


DEFAULT_LIMITS = (16384, 1024)
RELOAD_LADDER = ((4096, 1024), DEFAULT_LIMITS, (65536, 4096))


def compact_bytes(value: object) -> int:
    """UTF-8 JSON bytes: a machinery counter, never provider/model tokens."""
    return len(json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode())


class ContextOverflow(RuntimeError):
    pass


class RetrievalExhausted(RuntimeError):
    pass


@dataclass(frozen=True)
class Reference:
    entry_id: str
    claim: str
    assessment_id: str
    source_digest: str
    context_fingerprint: str
    method_registry_fingerprint: str
    assessed_at: str

    @property
    def key(self) -> tuple[str, str, str]:
        return self.entry_id, self.assessment_id, self.claim


@dataclass
class Block:
    messages: list[dict]
    reference: Reference | None = None
    packet: bool = False


@dataclass
class Turn:
    number: int
    blocks: list[Block] = field(default_factory=list)
    completed: bool = False


class History:
    """Store whole turn/exchange blocks; restoration always requires a call.

    At the start of turn 5, a packet first exposed in turn 1 has survived four
    completed turns. B/C replace that whole exchange with a metadata reference.
    Ordinary user/final messages are kept until the common input ceiling needs
    emergency removal of their complete turn. A never performs packet-age
    eviction and never keeps packets in an out-of-context restoration cache.
    """

    def __init__(self, condition: str, brief: str, glossary: dict, tools: list):
        if condition not in ("A", "B", "C"):
            raise ValueError("condition must be A, B or C")
        self.condition = condition
        self._protected = [
            {"role": "system", "content": brief},
            {"role": "user", "content": json.dumps({"identifier_glossary": glossary}, sort_keys=True)},
        ]
        self._tools = deepcopy(tools)
        self.turns: list[Turn] = []
        self.index: dict[tuple[str, str, str], Reference] = {}
        self._cache: dict[tuple[str, str, str], Block] = {}
        self._levels: dict[tuple[int, tuple[str, str, str]], int] = {}
        self.events: list[dict] = []
        self.logical_lookups = 0
        self.cache_hits = 0
        self.server_reloads = 0
        self._next_turn = 1

    @property
    def current(self) -> Turn:
        if not self.turns or self.turns[-1].completed:
            raise RuntimeError("no current turn")
        return self.turns[-1]

    def begin_turn(self, user_content: str) -> int:
        if self.turns and not self.turns[-1].completed:
            raise RuntimeError("finish the current turn first")
        turn = Turn(self._next_turn, [Block([{"role": "user", "content": user_content}])])
        self._next_turn += 1
        self.turns.append(turn)
        if self.condition != "A":
            for old in self.turns[:-1]:
                if turn.number - old.number < 4:
                    continue
                for position, block in enumerate(old.blocks):
                    if not block.packet:
                        continue
                    assert block.reference is not None
                    old.blocks[position] = Block([{
                        "role": "user",
                        "content": json.dumps({"host_reference": asdict(block.reference)}, sort_keys=True),
                    }], reference=block.reference)
                    self.events.append({"type": "packet_eviction", "at_turn": turn.number,
                                        "exposure_turn": old.number,
                                        "reference": asdict(block.reference)})
        return turn.number

    def add_packet(self, reference: Reference, messages: list[dict],
                   limits: tuple[int, int] = DEFAULT_LIMITS) -> None:
        """Append one whole tool exchange; B caches exposed default exchanges."""
        if self.condition in ("A", "B") and limits != DEFAULT_LIMITS:
            raise ValueError("A/B can expose only the default packet budget")
        # Check complete function-call/result pairing without parsing semantics.
        calls = [call["id"] for message in messages
                 for call in message.get("tool_calls", [])]
        results = [message["tool_call_id"] for message in messages if message.get("role") == "tool"]
        if not calls or len(set(calls)) != len(calls) or calls != results:
            raise ValueError("packet requires a complete ordered tool-call/result exchange")
        if reference.key in self.index and self.index[reference.key] != reference:
            raise ValueError("an assessment reference cannot change identity or original time")
        block = Block(deepcopy(messages), reference=reference, packet=True)
        self.current.blocks.append(block)
        self.index[reference.key] = reference
        if self.condition == "B":
            cached = self._cache.get(reference.key)
            # Tool-call IDs can change when replaying a cached body. Cache the
            # first exposed whole exchange and return it byte-for-byte to caller.
            if cached is None:
                self._cache[reference.key] = deepcopy(block)

    def add_messages(self, messages: list[dict]) -> None:
        self.current.blocks.append(Block(deepcopy(messages)))

    def finish_turn(self, answer: str) -> None:
        self.add_messages([{"role": "assistant", "content": answer}])
        self.current.completed = True

    def request(self) -> dict:
        references = [asdict(reference) for reference in self.index.values()]
        messages = deepcopy(self._protected)
        messages.append({"role": "user", "content": json.dumps({"reference_index": references}, sort_keys=True)})
        messages.extend(deepcopy(message) for turn in self.turns
                        for block in turn.blocks for message in block.messages)
        return {"messages": messages, "tools": deepcopy(self._tools)}

    def prepare(self, counter: Callable[[dict], int], cap: int) -> dict:
        """Apply a common full-request input cap, removing complete old turns."""
        if isinstance(cap, bool) or not isinstance(cap, int) or cap <= 0:
            raise ValueError("cap must be a positive integer")
        self.current  # Preparing only makes sense inside a user turn.
        while True:
            request = self.request()
            count = counter(request)
            if isinstance(count, bool) or not isinstance(count, int) or count < 0:
                raise ValueError("request counter must return a nonnegative integer")
            if count <= cap:
                return request
            candidate = next((turn for turn in self.turns if turn.completed), None)
            if candidate is None:
                self.events.append({"type": "context_overflow", "at_turn": self.current.number,
                                    "count": count, "cap": cap})
                raise ContextOverflow("protected brief/index/tools and current turn exceed the input cap")
            self.turns.remove(candidate)
            self.events.append({"type": "emergency_turn_eviction", "at_turn": self.current.number,
                                "removed_turn": candidate.number, "count_before": count, "cap": cap})

    def cache_lookup(self, reference: Reference) -> list[dict]:
        if self.condition != "B":
            raise PermissionError("host cache lookup belongs only to condition B")
        self.current
        self.logical_lookups += 1
        block = self._cache.get(reference.key)
        if block is None or block.reference != reference:
            self.events.append({"type": "cache_miss", "at_turn": self.current.number,
                                "reference": asdict(reference)})
            raise KeyError(reference.key)
        self.cache_hits += 1
        self.events.append({"type": "cache_hit", "at_turn": self.current.number,
                            "reference": asdict(reference)})
        return deepcopy(block.messages)

    def reload(self, reference: Reference, retrieve: Callable[[Reference, tuple[int, int]], object]) -> object:
        if self.condition != "C":
            raise PermissionError("bounded historical service retrieval belongs only to C")
        if self.index.get(reference.key) != reference:
            raise KeyError("only previously exposed, exact assessment identities may be restored")
        key = self.current.number, reference.key
        level = self._levels.get(key, 0)
        if level >= len(RELOAD_LADDER):
            raise RetrievalExhausted("all three budget levels were requested this turn")
        limits = RELOAD_LADDER[level]
        self._levels[key] = level + 1  # A failed call still consumes its level.
        self.logical_lookups += 1
        self.server_reloads += 1
        self.events.append({"type": "service_reload", "at_turn": self.current.number,
                            "reference": asdict(reference), "limits": list(limits)})
        return retrieve(reference, limits)


@dataclass
class TurnBudget:
    started_monotonic: float
    deadline_seconds: float = 120.0
    max_cycles: int = 6
    cycles: int = 0

    def before_cycle(self, now_monotonic: float) -> None:
        if now_monotonic - self.started_monotonic >= self.deadline_seconds:
            raise TimeoutError("original user-turn deadline exhausted")
        if self.cycles >= self.max_cycles:
            raise RetrievalExhausted("user-turn model/tool cycle limit exhausted")
        self.cycles += 1


def normalise_usage(raw: dict | None, *, output_includes_reasoning: bool) -> dict:
    """Normalise an already provider-adapted usage object without imputation.

    Cached input is included in input. Reasoning is added exactly once when
    provider output excludes it. Unknown mandatory fields keep the total null.
    Endpoint-specific adapters and instrument reconciliation remain necessary.
    """
    names = ("input_tokens", "output_tokens", "cached_input_tokens", "reasoning_tokens")
    readings = {name: None if raw is None else raw.get(name) for name in names}
    for value in readings.values():
        if value is not None and (isinstance(value, bool) or not isinstance(value, int) or value < 0):
            raise ValueError("usage readings must be nonnegative integers or null")
    cached, input_ = readings["cached_input_tokens"], readings["input_tokens"]
    if cached is not None and input_ is not None and cached > input_:
        raise ValueError("cached input must be a subset of input")
    parts = [input_, readings["output_tokens"]]
    if not output_includes_reasoning:
        parts.append(readings["reasoning_tokens"])
    total = None if any(part is None for part in parts) else sum(parts)
    return {"usage_raw": deepcopy(raw), **readings, "total_tokens": total,
            "usage_missing_reason": "missing_required_usage" if total is None else None}


def deadline_restricted_latency(observed_seconds: float | None, *, completed: bool,
                                deadline_seconds: float = 120.0) -> float:
    if observed_seconds is not None and observed_seconds < 0:
        raise ValueError("latency cannot be negative")
    return min(observed_seconds, deadline_seconds) if completed and observed_seconds is not None else deadline_seconds
