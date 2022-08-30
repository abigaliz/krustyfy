# krustyfy
Unobtrusive notification daemon made in Rust and Qt.

Notifications **can't be interacted with** (unless you keep Left Alt key pressed) and **mouse input goes right through them** :)

![Sorry :(](https://raw.githubusercontent.com/abigaliz/krustyfy/master/krustify.png)

## Name

**K**: Because it's made in Qt, so it works nice with KDE.

**Rust**: Because it's made in rust.

**Krusty**: Because it's made by a dumb sad clown.

**Krustyfy**: Because it just had to have the "ify" suffix, as in "notify".

## How to contribute and help the project
I don't know, I never thought I'd get this far. Also since I'm just learning about how to code in rust it's probably full of bad practices and awful code. :)

## Usage

By pressing **Left Alt key** you freeze all notifications (new notifications still come in, but start frozen) and you're able to click on them to interact.

Otherwise, they are semi-transparent and get blurry and even less opaque when your cursor is over it. Also you click through them, so if a notification spanws just when you were about to click, you don't have to worry; the click will be processed as if the notification was nothing at all, nothing at all, nothing at all.
