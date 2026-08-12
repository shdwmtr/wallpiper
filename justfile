set shell := ["bash", "-euo", "pipefail", "-c"]

gnome_build_dir := "target/gnome"
kde_build_dir := "target/kde"

default:
    @just --list

build: build-cargo build-interpose build-gnome build-kde

build-core: build-cargo build-interpose

build-cargo:
    cargo build --release --workspace

build-interpose:
    make -C rt-translation-layer

build-hyprland:
    cargo build --release --manifest-path wallpiper-portal-hyprland/Cargo.toml --target-dir target

build-i3:
    cargo build --release --manifest-path wallpiper-portal-i3/Cargo.toml --target-dir target

build-sway:
    cargo build --release --manifest-path wallpiper-portal-sway/Cargo.toml --target-dir target

build-gnome:
    meson setup {{gnome_build_dir}} wallpiper-portal-gnome/native --reconfigure
    ninja -C {{gnome_build_dir}}

build-kde:
    cmake -S wallpiper-portal-kde/native -B {{kde_build_dir}} -DCMAKE_BUILD_TYPE=Release
    cmake --build {{kde_build_dir}} --parallel

install-gnome: build-gnome
    sudo meson install -C {{gnome_build_dir}}
    sudo ldconfig
    mkdir -p "$HOME/.local/share/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev"
    cp -r wallpiper-portal-gnome/extension/. "$HOME/.local/share/gnome-shell/extensions/wallpiper-gnome@wallpiper.dev/"
    gnome-extensions enable wallpiper-gnome@wallpiper.dev

install-kde:
    ./wallpiper-portal-kde/install.sh

install-wallpiperd: build-cargo
    mkdir -p "$HOME/.local/lib/wallpiper"
    install -m 755 target/release/wallpiperd "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/wallpiperctl "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/libwallpiper-preload.so "$HOME/.local/lib/wallpiper/"
    install -m 755 target/release/libVkLayer_wallpiper_capture.so "$HOME/.local/lib/wallpiper/"
    for portal in hyprland i3 sway; do bin="target/release/wallpiper-portal-$portal"; [ -f "$bin" ] && install -m 755 "$bin" "$HOME/.local/lib/wallpiper/" || true; done
    mkdir -p "$HOME/.local/bin"
    ln -sf "$HOME/.local/lib/wallpiper/wallpiperd" "$HOME/.local/bin/wallpiperd"
    ln -sf "$HOME/.local/lib/wallpiper/wallpiperctl" "$HOME/.local/bin/wallpiperctl"
    @echo "installed to $HOME/.local/lib/wallpiper, symlinked at $HOME/.local/bin/{wallpiperd,wallpiperctl}"
    @echo "make sure $HOME/.local/bin is on your PATH"

test:
    cargo test --workspace

check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

clean:
    rm -rf target
