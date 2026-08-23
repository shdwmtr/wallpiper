/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

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
