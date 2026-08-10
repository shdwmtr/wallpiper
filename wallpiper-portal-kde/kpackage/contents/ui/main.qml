import QtQuick
import org.kde.plasma.plasmoid
import dev.wallpiper.capture as WallpiperCapture

WallpaperItem {
    id: root

    WallpiperCapture.CaptureItem {
        id: captureItem
        anchors.fill: parent
    }

    Rectangle {
        visible: captureItem.debugEnabled
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 12
        width: 210
        height: statsColumn.implicitHeight + 16
        radius: 6
        color: "#c8141418"

        Column {
            id: statsColumn
            anchors.fill: parent
            anchors.margins: 8
            spacing: 4

            Text {
                color: "white"
                font.family: "monospace"
                text: "D " + captureItem.displayFps
            }
            Text {
                color: "white"
                font.family: "monospace"
                text: "C " + captureItem.captureFps
            }
            Text {
                color: "white"
                font.family: "monospace"
                text: "F " + captureItem.lastFrameMs.toFixed(1) + " / " + captureItem.peakFrameMs.toFixed(1)
            }
        }
    }
}
