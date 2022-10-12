# krustyfy

<p align="center">
  <img src="https://raw.githubusercontent.com/abigaliz/krustyfy/master/krustify.png">
</p>


Unobtrusive notification daemon made in Rust and Qt.

Notifications **can't be interacted with** (unless you keep Left Alt key pressed) and **mouse input goes right through them** :)


https://user-images.githubusercontent.com/112440538/188256590-9793e49d-8265-4d85-a5f7-c2c3f3ed01bd.mp4

## Configuration

Most settings can be changed directly from the **res/themes/{current theme}/template.ui** config file. From the layout of the notification itself to settings like duration, monitor, shadow color, etc. More settings comming soon. :)

<p align="center">
  <img src="https://user-images.githubusercontent.com/112440538/188322780-06a043c8-4b3f-449d-9853-3154f8788b0b.png">
</p>

Take into account that some widgets MUST exist in the template.ui file, otherwise it'll crash. Modify it wisely.



## Theming

Themes are located in the **res/themes** subdirectory. Each folder corresponds to a theme, which must have a **template.ui** file. Themes are loaded when the app is launched, and can be changed from the system tray:

![themes](https://user-images.githubusercontent.com/112440538/190928867-c006a63a-97ee-4eb5-8e2a-ccf012671547.png)

Currently we have two built in themes: **default** and **compact**, and they can be changed, and even modified, while the app is running:



https://user-images.githubusercontent.com/112440538/190928912-c352c2ad-a002-4d9a-aed8-0429989bc1d5.mp4



## Name

**K**: Because it's made in Qt, so it works nice with KDE.

**Rust**: Because it's made in rust.

**Krusty**: Because it's made by a dumb sad clown.

**Krustyfy**: Because it just had to have the "ify" suffix, as in "notify".

## How to contribute and help the project
I don't know, I never thought I'd get this far. Also since I'm just learning about how to code in rust it's probably full of bad practices and awful code. :)

## Building

Tested with Debian 11 netinst + KDE

### Requirements
1. Install Rustup (https://rustup.rs/)
2. Install required dev packages: `#apt-get install qt5-qmake qtbase5-dev cmake build-essential pkg-config qttools5-dev`

### Clone and build
1. Clone from git `$git clone https://github.com/abigaliz/krustyfy.git`
2. `cd krustyfy`
3. `cargo build --release`

### Running it
1. Disable KDE Notifications from the system tray by going to the System Tray Settings and marking Notifications as Disabled:
![image](https://user-images.githubusercontent.com/112440538/195397565-2d15242f-be1e-40ba-b7f1-4b1b9e6c0457.png)
2. Log out of the current session.
3. Run krustyfy from `$HOME/krustyfy/target/release/krustyfy`

You can also set it to run on startup from KDE System Settings:

![image](https://user-images.githubusercontent.com/112440538/195398592-cc36fac4-95a0-4633-9b5b-22852101f138.png)

Remember to set up the work path to `$HOME/krustyfy/target/release/`, otherwise it won't be able to access the notification templates folder.


## Usage

By pressing **Left Alt key** you freeze all notifications (new notifications still come in, but start frozen) and you're able to click on them to interact.

Otherwise, they are semi-transparent and get blurry and even less opaque when your cursor is over it. Also you click through them, so if a notification spanws just when you were about to click, you don't have to worry; the click will be processed as if the notification was nothing at all, nothing at all, nothing at all.
