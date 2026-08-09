use ksni::menu::{CheckmarkItem, MenuItem, StandardItem};
use ksni::Icon;

const ICON_BYTES: &[u8] = include_bytes!("../resources/icon.png");

fn load_icon() -> Vec<Icon> {
    let Ok(img) = image::load_from_memory(ICON_BYTES) else {
        return Vec::new();
    };
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width() as i32, rgba.height() as i32);
    let mut data = Vec::with_capacity(rgba.as_raw().len());
    for px in rgba.pixels() {
        data.push(px[3]);
        data.push(px[0]);
        data.push(px[1]);
        data.push(px[2]);
    }
    vec![Icon {
        width,
        height,
        data,
    }]
}

pub struct WallpiperTray {
    pub paused: bool,
    pub muted: bool,
}

impl ksni::Tray for WallpiperTray {
    fn id(&self) -> String {
        "wallpiper".into()
    }

    fn title(&self) -> String {
        "Wallpiper".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        load_icon()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Open Wallpaper Engine".into(),
                activate: Box::new(|_tray: &mut Self| {
                    crate::launch_ui(Some("-showbrowse"));
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            CheckmarkItem {
                label: "Pause".into(),
                checked: self.paused,
                activate: Box::new(|tray: &mut Self| {
                    tray.paused = !tray.paused;
                    crate::set_paused(tray.paused);
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "Mute".into(),
                checked: self.muted,
                activate: Box::new(|tray: &mut Self| {
                    tray.muted = !tray.muted;
                    crate::set_muted(tray.muted);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_tray: &mut Self| {
                    crate::cleanup();
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
