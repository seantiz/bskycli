# Bluesky CLI client
I needed Bluesky from the terminal so I spun this one off `skyscraper-cli`.

I can make a brief note on how to handle new create sessions (because Lexicon XRPC demands they're made frequently) if anything is unclear.

## Installation
Currently you can only build this from source. Git clone it, make sure you have `cargo` installed and run it at project root to build or install.

If you choose to install this bsky app globally with `cargo install --path .` then the final app is executable by running `bsky` to open it.

## IMPORTANT: How to Uninstall
Please do not just run `cargo uninstall` because this may not entirely remove any PDS config or keyring login credentials you've used with this app.

Run the uninstall script instead from project root: `./uninstall.sh`

Whether or not you run this script, your keyring credentials will always be encrypted throughout.

# Original Credit
Credit to Cameron Banga for the original `skyscraper-cli` that this is forked from - I just needed a client to build off of where I could later use the kitty graphics protocol and a less involved way of retrieving credentials.

# License
MIT Licensed.
