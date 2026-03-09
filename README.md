# Pulsar

Pulsar is a Linux/Steam Deck-first mod manager for No Man's Sky built with Rust + Tauri v2 and integrated with Nexus Mods.

## Attribution

PulsarMM is an independent project that builds on ideas and prior implementation work from:

- [Syzzle07](https://github.com/Syzzle07/)/[SingularityMM](https://github.com/Syzzle07/SingularityMM/)
- [Syzzle07](https://github.com/Syzzle07/)/[NMS-Mod-Manager](https://github.com/Syzzle07/NMS-Mod-Manager/)

This project builds on community learnings and prior work while taking a different product direction focused on Linux, SteamOS, Steam Deck, and Flatpak delivery.

Pulsar is independently maintained and is not an official continuation by the original maintainer.

Pulsar is a fan-made project and is not affiliated with Hello Games or Nexus Mods.

## Features

*   **Automatic Game Detection:** Finds your Steam, GOG or Gamepass PC installation of No Man's Sky automatically.
*   **Mod Management:** Easily enable, disable, download, install and set the priority of your mods.
*   **Mod Update Check:** Easily check for updates for your installed mods.
*   **Drag & Drop Installation:** Install mods by simply dropping `.zip`, `.rar` or `.7z` files onto the application.
*   **Nexus Mods Integration with SSO:** Link the Manager with your Nexus Account through the Single Sign-On and download mods using the "Mod Manager Download" button or if you have a Premium Account you can browse and download mods directly through the Manager itself.
*   **Profiles:** Includes option to save mod profiles for different play styles.

## Dependencies

- Linux/Steam Deck: WebKitGTK runtime used by Tauri (provided by system or Flatpak runtime)
- Windows: WebView2 runtime (usually preinstalled on modern Windows)
