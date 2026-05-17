# Bluesky CLI client
I needed Bluesky from the terminal so I spun this one off `skyscraper-cli`. 

NOTE: This program is me hacking around the lexicon schema and endpoints of the AT Protocol, so this isn't a stable bsky client. It does the job and is secure, but that's about all it can say for itself right now.

I also stripped out `clap` parsing, so if you want a cli that's more aligned with Unix standards I'd choose to download the original `skyscraper` cli.

## Installation
You can build this from source right now. Git clone it, make sure you have `cargo` installed and run at the build or install command at project root.

If you choose to install globally with `cargo install --path .` then the final app is executable by running `bsky`.

## IMPORTANT: How to Uninstall
Please do not just run `cargo uninstall` because this may not entirely remove any PDS config or keyring login credentials you've used with this app.

Run the uninstall script instead from project root: `./uninstall.sh`

Whether or not you run this script, your keyring credentials will always be encrypted throughout.

# Original Credit
Credit to Cameron Banga for the original `skyscraper-cli` that this is forked from originally.

# License
MIT Licensed.
