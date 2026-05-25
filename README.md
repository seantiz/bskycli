# Bluesky CLI
What's different's about this one compared to all the other Bluesky clients? Not much! 

This app is a product of me hacking around the ATP endpoints and just seeing what works and what I could get away with doing. 

**As of version 0.4.4** this app does the job and gives a completely secure Bluesky experience as long as you're using a PTY terminal like kitty. 

## Features in Brief
All the essentials you'd expect:

- Search Bluesky
- Automatic login with keyring
- Turn on (or off) notifications of any type
- See your up-to-date profile as others see it
- See your main timeline feed of accounts you follow
- Open any post with images and they render in your terminal
- Control whether you see replies, quoted posts and reposts of account you follow

### What's Missing (Long-Term)

- Posts with embedded links can only launch to your browser when they're prefixed with `http`. Lacking support for `at`, `gemini` and shortened web links.

- There's no video support. This isn't a priority for me, you may disagree. It'd likely take an external call to `mpv` and I'm unsure Bluesky welcome hotlinking videos from their endpoints. Until there's a clear confirmation from their side, I'm leaving this alone.

- Great idea mentioned on the Bluesky codebase here as an anti-doomscrolling feature: [bluesky-social/social-app#10601](https://github.com/bluesky-social/social-app/issues/10601) ... Right now the app sort of works this way (you have to press `R` to hard re-render the timeline but could do with revisiting. 

## Installation
Git clone and build this from source. Make sure you have `cargo` installed, then run the cargo build or install command at project root.

If you choose to install globally with `cargo install --path .` then the final app is executable by simply running `bsky`.

## IMPORTANT: How to Uninstall
Please do not just run `cargo uninstall` because this may not entirely remove any PDS config or keyring login credentials you've used with this app.

Run the uninstall script instead from project root: `./uninstall.sh`

Whether or not you run this script, your keyring credentials will always be encrypted throughout.

## ALSO IMPORTANT: Nerd Fonts
You need nerd fonts installed on your system for the app tabline to fully render. This can go on the todo list if it becomes an issue. But since it's a one-user codebase right now, I've no plans here.

---

### Original Credit
Credit to Cameron Banga for the original `skyscraper-cli` that this is forked from originally.

### License
MIT Licensed.
