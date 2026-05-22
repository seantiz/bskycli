# Bluesky CLI
What's different's about this one compared to all the other Bluesky clients? Not much! 

The wider issue is the AT Protocol and Bluesky libraries are still likely to be subject to change over the next year or so. In that spirit, this app is a product of me hacking around the ATP endpoints and just seeing what works and what I could get away with doing. 

**As of version 0.4.0** this app does the job and is secure. I think it makes for a full Bluesky experience. Feel free to get in touch if something is lacking.

## Features in Brief
All the essentials you'd expect from social media:

- Search Bluesky
- Automatic login with keyring
- Turn on (or off) notifications of any type
- See your up-to-date profile as others see it
- See your main timeline feed of accounts you follow
- Open any post with images and they render in your terminal
- Control whether you see replies, quoted posts and reposts of account you follow

## TODOs
If you're a kitty terminal user and you know the graphics protocol, help is welcome on closing the gaps below.

### Immediate
- Bug: Sometimes the image rendering will block navigating through a thread.

### Short Term
- Quoted posts aren't fully rendered. I haven't decided if KGP or Ratatui is better off handling this case.
- It'd be great if posts with links could have their links loaded directly from within the app. This is definitely something to wire through kitty.

### Longer Term
- There's no video rendering yet. This isn't a priority for me, you may disagree. It'd likely to outward calls to `mpv` to get it done. Whether or not to handle it inline is another aspect.

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
