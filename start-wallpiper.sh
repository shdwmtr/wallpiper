make build-core
make build-hyprland

export WALLPIPER_PROTON_BIN="/home/shdw/.local/share/Steam/steamapps/common/Proton 11.0/proton"
export WALLPIPER_PORTAL=hyprland
export WALLPIPER_WPE_IPC_DUMP_FILE=$HOME/file.log

exec ./target/release/wallpiper-daemon
