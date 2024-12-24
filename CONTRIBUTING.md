# Contributing

Contributions to Tempo are welcome!

If you're an experienced developer (especially if you have experience with Rust/Tauri), your help would be very valuable!

## Ways to Help
### Testing
The best way you can help out with Tempo right now is to **use it**.

Tempo is probably a buggy mess. It's a top priority of mine to make sure stuff actually works.

Contributions involving writing tests or helping set up CI/CD are also very welcome!

### Logo/Branding
It would be awesome if someone wants to help out with creating a logo or branding for Tempo.

### Rust Stuff
Tempo uses the latest stable version of Rust.

I'm extremely interested in creating a library which parses project files from various DAWs. It would be very nice to be able to parse project files into some intermediate format (maybe something like DAWproject). This would allow Tempo to support other DAWs and allow for lots of interesting features in the future. I would like to build out this library before adding support for other DAWs to Tempo.

There are lots of TODOs throughout the codebase, feel free to make any PRs to address them.

## Legal Stuff
You cannot violate the EULA of any DAW while writing your contribution, and the code itself cannot violate the EULA of any DAW. **You need to acknowledge you haven't done either in your pull requests.**

Ableton explicitly states that reverse engineering, disassembling or tampering with the Ableton executable or application data violates their EULA.
I interpret this meaning that, in the case of Ableton's macOS bundle, everything inside of `Ableton Live [10/11/12/etc.].app` is off-limits.

Clean-room reverse engineering proprietary file formats is legal in the United States. You should check if this is legal in your jurisdiction.

All contributions are assumed to be placed under the existing licenses used in this codebase.
