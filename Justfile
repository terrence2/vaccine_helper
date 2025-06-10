format:
    cargo fmt --all

build: format check
    cargo build

run: build
    cargo run

check:
    cargo clippy

clean-config:
    rm ~/.local/share/vaccinehelper/app.ron

vibe:
    nix-shell -p nodejs_24 --command "npm install @anthropic-ai/claude-code"
    nix-shell -p nodejs_24 --command "./node_modules/.bin/claude"
