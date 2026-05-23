# Bluesky CLI
What's different's about this one compared to all the other Bluesky clients? Not much! 

The wider issue is the AT Protocol and Bluesky libraries are still likely to be subject to change over the next year or so. In that spirit, this app is a product of me hacking around the ATP endpoints and just seeing what works and what I could get away with doing. 

**As of version 0.4.4** this app does the job and is secure. I think it makes for a full Bluesky experience. Feel free to get in touch if something is lacking.

## Features in Brief
All the essentials you'd expect from social media:

- Search Bluesky
- Automatic login with keyring
- Turn on (or off) notifications of any type
- See your up-to-date profile as others see it
- See your main timeline feed of accounts you follow
- Open any post with images and they render in your terminal
- Control whether you see replies, quoted posts and reposts of account you follow

## What's Missing (Long-Term)

- Ideally posts with links (destructured from ATP's post "facets") could be navigatied to directly from within the timeline. Very likely this has to be handled outside of ratatui.

- Great feature mentioned on the Bluesky codebase here: [bluesky-social/social-app#10601](https://github.com/bluesky-social/social-app/issues/10601) which would work "login, react, leave" mode. Right now this app sort of works this way (you have to press `R` to hard re-render the timeline) but if it could be an option to totally guarantee you see nothing but a snapshot of your timeline and are protected from a doomscrolling session, that'd be a worth bringing into the settings menu.

- There's no video rendering at the moment. This isn't a priority for me, you may disagree. It'd likely to an external call to `mpv` get it done, which presents its own risks and I'm unsure Bluesky welcome hotlinking videos from their endpoints.

## Installation
Git clone and build this from source. Make sure you have `cargo` installed, then run the cargo build or install command at project root.

If you choose to install globally with `cargo install --path .` then the final app is executable by simply running `bsky`.

## IMPORTANT: How to Uninstall
Please do not just run `cargo uninstall` because this may not entirely remove any PDS config or keyring login credentials you've used with this app.

Run the uninstall script instead from project root: `./uninstall.sh`

Whether or not you run this script, your keyring credentials will always be encrypted throughout.

## ALSO IMPORTANT: Nerd Fonts
Having a nerd font installed on your system is a requirement for the app tabline to fully render. This can go on the todo list if it becomes an issue. But since it's a one-user codebase right now, I've no plans to change it at the moment.

# Original Credit
Credit to Cameron Banga for the original `skyscraper-cli` that this is forked from originally.

# License
MIT Licensed.
