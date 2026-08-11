set shell := ["bash", "-euo", "pipefail", "-c"]

gnome_build_dir := "target/gnome"
kde_build_dir := "target/kde"

default:
    @just --list

build: build-cargo build-interpose build-gnome build-kde

build-cargo:
    cargo build --release --workspace

build-interpose:
    make -C rt-translation-layer

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

test:
    cargo test --workspace

check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

clean:
    rm -rf target
