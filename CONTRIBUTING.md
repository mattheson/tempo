# Contributing

Contributions to Tempo are welcome!

I'm currently a student still, so my inexperience might be present in many parts of the codebase. If you're an experienced developer (especially if you have experience with Rust/Tauri/React), your help would be very valuable!

## Ways to Help
### Testing
The best way you can help out with Tempo right now is to **use it**.

Tempo is probably a buggy mess. It's a top priority of mine to make sure stuff actually works.

Contributions involving writing tests or helping set up CI/CD are also very welcome!

### Logo/Branding
It would be awesome if someone wants to help out with creating a logo or branding for Tempo.

### Frontend Stuff
Tempo uses TypeScript, React and Tailwind for the frontend. I learned how to use the latter two in the process of developing Tempo, so there's probably a lot of grotesque code throughout the frontend.

Contributions which clean up Tempo's frontend are welcome. There are lots of spots where components are copied/pasted, stuff could be much more modular. Also, adding dark mode would be nice.

Also, I don't know if my usage of Zustand is very appropriate. I put a lot of state in the Zustand store. It works for now :)

For the time being, I think it's best to focus on iteratively improving Tempo's UI. I'd be open to doing a full redesign at some point. If someone wants to make a Figma design, feel free to do so and I'll check it out! I'm open to switching over to Svelte on a redesign. A redesign is not a priority right now though.

### Rust Stuff
Tempo uses the latest stable version of Rust.

I'm extremely interested in creating a library which parses project files from various DAWs. It would be very nice to be able to parse project files into some intermediate format (maybe something like DAWproject). This would allow Tempo to support other DAWs and allow for lots of interesting features in the future. I would like to build out this library before adding support for other DAWs to Tempo.

There are lots of TODOs throughout the codebase, feel free to make any PRs to address them.

## Legal Stuff
You cannot violate the EULA of any DAW while writing your contribution, and the code itself cannot violate the EULA of any DAW. **You need to acknowledge you haven't done either in your pull requests.**

Ableton explicitly states that reverse engineering, disassembling or tampering with the Ableton executable or application data violates their EULA.
I interpret this meaning that, in the case of Ableton's macOS bundle, everything inside of `Ableton Live [10/11/12/etc.].app` is off-limits.

It's possible that my scanning of Ableton's plugin database is a gray area.
Ableton's plugin database is an artifact produced by the application, so I think it's probably alright.
Ableton developers: Please feel free to reach out to me if you have concerns about this.

Clean-room reverse engineering proprietary file formats is legal in the United States. You should check if this is legal in your jurisdiction.

All contributions are assumed to be placed under the existing licenses used in this codebase.

## Getting Started
These steps outline how to get started with running Tempo in dev mode:

1. Install rustup and npm
2. Clone the repository
3. Run `npm install`
4. Run `npm run tauri dev -- -- -- -d [path which will be used as Tempo's data directory]` (you need to provide a directory to use as Tempo's data directory when running `tauri dev`)