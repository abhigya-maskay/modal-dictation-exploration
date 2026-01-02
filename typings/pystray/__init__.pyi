from collections.abc import Callable

from PIL.Image import Image

type MenuItemAction = Callable[[Icon, MenuItem], None] | Menu | None

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
        action: MenuItemAction,
        checked: Callable[[MenuItem], bool] | None = None,
        enabled: bool | Callable[[MenuItem], bool] = True,
    ) -> None: ...
