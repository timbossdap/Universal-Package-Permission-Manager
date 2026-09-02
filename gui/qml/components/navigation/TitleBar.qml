import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import ".."

// top toolbar header
ToolBar {
    id: titleBar

    property int currentTabIndex: 0
    // the list of available sources comes from the Rust bridge - only
    // package managers that are actually installed show up as tabs
    property var tabModel: uppm.available_sources

    signal openMenuRequested()
    signal tabSelected(string sourceName)

    background: Rectangle {
        color: "#121212"
    }

    implicitHeight: 72

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 12
        spacing: 12

        // custom menu button using rectangle and mousearea instead of toolbutton
        Rectangle {
            id: menuBtn
            width: 40
            height: 40
            color: isHover ? "#2a2a2a" : "transparent"
            radius: 20
            property bool isHover: false

            MaterialIcon {
                anchors.centerIn: parent
                name: "menu"
                tint: "white"
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: {
                    menuBtn.isHover = true
                }
                onExited: {
                    menuBtn.isHover = false
                }
                onClicked: {
                    titleBar.openMenuRequested()
                }
            }
        }

        // tab bar track
        Rectangle {
            id: tabBarTrack
            Layout.fillWidth: true
            Layout.preferredHeight: 56
            Layout.fillHeight: false
            Layout.alignment: Qt.AlignVCenter
            radius: height / 2
            color: Qt.rgba(1, 1, 1, 0.06)

            // sliding active tab pill
            Rectangle {
                id: tabHighlight
                radius: height / 2
                color: Material.accent
                y: tabRow.y
                height: tabRow.height

                // manual position calculation: x position is the sum of all
                // tab widths before the current index
                x: {
                    var count = tabRepeater.count
                    if (count == 0) {
                        return tabRow.x
                    }
                    var idx = titleBar.currentTabIndex
                    var totalX = tabRow.x
                    var i = 0
                    while (i < idx && i < count) {
                        var item = tabRepeater.itemAt(i)
                        if (item != null) {
                            totalX += item.width + tabRow.spacing
                        }
                        i = i + 1
                    }
                    return totalX
                }

                // width of the currently selected tab
                width: {
                    var count = tabRepeater.count
                    if (count > 0) {
                        var idx = titleBar.currentTabIndex
                        if (idx >= 0 && idx < count) {
                            var currentItem = tabRepeater.itemAt(idx)
                            if (currentItem != null) {
                                return currentItem.width
                            }
                        }
                    }
                    return 0
                }

                Behavior on x { NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
                Behavior on width { NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
            }

            Row {
                id: tabRow
                x: 6
                anchors.verticalCenter: parent.verticalCenter
                height: parent.height - 12
                spacing: 6

                Repeater {
                    id: tabRepeater
                    model: tabModel

                    delegate: Item {
                        id: tabItem
                        height: tabRow.height
                        width: tabLabel.implicitWidth + 44
                        property bool selected: index == titleBar.currentTabIndex

                        Label {
                            id: tabLabel
                            anchors.centerIn: parent
                            text: modelData
                            color: {
                                if (tabItem.selected == true) {
                                    return "white"
                                } else {
                                    return "#9aa5b1"
                                }
                            }
                            font.pixelSize: 18
                            font.bold: tabItem.selected

                            Behavior on color { ColorAnimation { duration: 220 } }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                var cur = titleBar.currentTabIndex
                                if (cur == index) {
                                    return
                                } else {
                                    titleBar.currentTabIndex = index
                                    titleBar.tabSelected(modelData)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
