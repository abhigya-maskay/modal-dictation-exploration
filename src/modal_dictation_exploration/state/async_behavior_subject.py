import asyncio
import threading
from collections.abc import AsyncGenerator


class AsyncBehaviorSubject[T]:
    """A lightweight async broadcast primitive with current-value semantics.

    Holds a current value that new subscribers receive immediately upon subscribing.
    All subscribers receive updates when `next()` is called.

    Attributes:
        value: The current value of the subject.
    """

    value: T
    _queues: list[asyncio.Queue[T]]
    _lock: threading.Lock

    def __init__(self, initial: T) -> None:
        """Initialize the subject with a starting value.

        Args:
            initial: The initial value that subscribers will receive immediately.
        """
        self.value = initial
        self._queues = []
        self._lock = threading.Lock()

    async def subscribe(self) -> AsyncGenerator[T, None]:
        """Subscribe to value updates.

        Yields the current value immediately, then yields each subsequent
        value pushed via `next()`. The subscription is automatically cleaned
        up when the caller stops iterating.

        Yields:
            The current value, followed by all future values.
        """
        queue: asyncio.Queue[T] = asyncio.Queue()
        with self._lock:
            initial = self.value
            self._queues.append(queue)
        try:
            yield initial
            while True:
                value = await queue.get()
                yield value
        finally:
            with self._lock:
                self._queues.remove(queue)

    def next(self, value: T) -> None:
        """Push a new value to all subscribers.

        Updates the current value and notifies all active subscribers.

        Args:
            value: The new value to broadcast.
        """
        with self._lock:
            self.value = value
            for queue in self._queues:
                queue.put_nowait(value)
