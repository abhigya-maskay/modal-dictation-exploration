from collections.abc import Callable

from PIL.Image import Image

class Icon:
    def __init__(
        self,
        name: str,
        icon: Image | None = None,
        title: str | None = None,
        menu: Menu | None = None,
    ) -> None: ...
    def run(self) -> None: ...
    def stop(self) -> None: ...

class Menu:
    def __init__(self, *items: MenuItem) -> None: ...

class MenuItem:
    def __init__(
        self,
        text: str,
        action: Callable[[Icon, MenuItem], None],
    ) -> None: ...
