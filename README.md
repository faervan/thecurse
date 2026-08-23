## Building

### Alpine
```

apk add --no-cache \
	git \
	rustup \
	pkgconf \
	clang \
	libxkbcommon-dev \
	libxkbcommon-static \
    wayland-static \
    alsa-lib-dev
rustup-init -y
. "$HOME/.cargo/env"
git clone https://github.com/faervan/thecurse.git
cd thecurse
cargo build --release -p thecurse_game --no-default-features
```

# Credits
The color palette at [`assets/textures/palette-duel-1x.png`](assets/textures/palette-duel-1x.png) is taken from [https://lospec.com/palette-list/duel].
