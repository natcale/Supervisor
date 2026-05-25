
Supervisor is licensed under the [MIT License](LICENSE). It uses and integrates with the following third-party software and services.

Supervisor bundles or depends on many open-source libraries. Runtime dependencies include:

[Tauri](https://tauri.app/): MIT or Apache-2.0

[Next.js](https://nextjs.org/): MIT

[React](https://react.dev/): MIT

[reqwest](https://github.com/seanmonstar/reqwest): MIT or Apache-2.0

[zip](https://github.com/zip-rs/zip2): MIT

See `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock` for the full dependency tree.

## LOOT

Bethesda plugin sorting can use [LOOT](https://loot.github.io/) when you configure `loot.exe` in Settings. LOOT is GPL-3.0. Supervisor invokes LOOT as an external executable; it is not linked into the Supervisor binary.

## Nexus Mods

Supervisor uses the [Nexus Mods API](https://app.swaggerhub.com/apis-docs/NexusMods/nexus-mods_public_api_params_in_form_data/1.0) for downloads and metadata. Use of the API is subject to [Nexus Mods Terms of Service](https://help.nexusmods.com/article/20-terms-of-service).

Supervisor identifies itself with the `Application-Name: Supervisor` HTTP header.

## Vortex

Supervisor can import Vortex `.collection` manifest files. The Vortex **logo** in the Collections UI is used for attribution only; Vortex is a separate product (GPL) by Nexus Mods. Supervisor does not include Vortex code.

## Steam

Game cover art may be loaded from Steam CDN URLs during game detection. Use is subject to [Steam Subscriber Agreement](https://store.steampowered.com/subscriber_agreement/).

## Trademarks

Steam, Nexus Mods, Vortex, Epic Games, GOG, and other names are trademarks of their respective owners. Supervisor is not affiliated with or endorsed by them.
