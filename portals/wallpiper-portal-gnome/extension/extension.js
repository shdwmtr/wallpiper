import Wallpiper from 'gi://Wallpiper?version=1.0';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

export default class WallpiperExtension extends Extension {
    enable() {
        try {
            Wallpiper.portal_start(global.backend, global.window_group);
            console.log('[wallpiper-gnome] portal started');
        } catch (e) {
            console.error(`[wallpiper-gnome] portal_start() failed: ${e.message}`);
        }
    }
    disable() {
        Wallpiper.portal_stop();
    }
}
