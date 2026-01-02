"""Tests for AsyncBehaviorSubject."""

import pytest

from modal_dictation_exploration.state.async_behavior_subject import (
    AsyncBehaviorSubject,
)


def test_initial_value_is_set():
    """value returns the initial value passed to constructor."""
    subject = AsyncBehaviorSubject(42)

    assert subject.value == 42


def test_next_updates_value():
    """Calling next() updates value."""
    subject = AsyncBehaviorSubject(42)

    subject.next(100)

    assert subject.value == 100


@pytest.mark.asyncio
async def test_subscribe_yields_initial_value():
    """First yielded value is the current value."""
    subject = AsyncBehaviorSubject(42)

    subscriber = subject.subscribe()
    first_value = await anext(subscriber)

    assert first_value == 42


@pytest.mark.asyncio
async def test_subscribe_yields_values_from_next():
    """Values pushed via next() are yielded to subscriber."""
    subject = AsyncBehaviorSubject(0)

    subscriber = subject.subscribe()
    await anext(subscriber)
    subject.next(42)
    value = await anext(subscriber)

    assert value == 42


@pytest.mark.asyncio
async def test_multiple_subscribers_receive_same_values():
    """All active subscribers receive the same updates."""
    subject = AsyncBehaviorSubject(0)

    subscriber1 = subject.subscribe()
    subscriber2 = subject.subscribe()
    value1 = await anext(subscriber1)
    value2 = await anext(subscriber2)

    assert value1 == 0
    assert value2 == 0

    subject.next(42)
    value1 = await anext(subscriber1)
    value2 = await anext(subscriber2)

    assert value1 == 42
    assert value2 == 42


@pytest.mark.asyncio
async def test_subscriber_cleanup_on_iteration_stop():
    """Queue is removed from _queues when subscriber stops."""
    subject = AsyncBehaviorSubject(0)
    assert len(subject._queues) == 0

    subscriber = subject.subscribe()
    await anext(subscriber)
    assert len(subject._queues) == 1

    await subscriber.aclose()
    assert len(subject._queues) == 0
