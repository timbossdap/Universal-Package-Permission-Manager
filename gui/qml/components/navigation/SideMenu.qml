import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import ".."

// side menu drawer
Drawer {
    id: sideMenu
    width: 240
    height: {
        if (parent != null) {
            return parent.height
        } else {
            return 600
        }
    }
    edge: Qt.LeftEdge
    modal: true

    property string currentPage: "main"

    signal pageSelected(string page)

    // dark overlay behind the drawer
    Overlay.modal: Rectangle {
        color: "black"
        opacity: 0.3
    }

    background: Rectangle {
        color: Material.background
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 4

        Label {
            text: "Menu"
            font.bold: true
            font.pixelSize: 20
            color: "white"
            Layout.bottomMargin: 14
            Layout.leftMargin: 6
            Layout.fillWidth: true
        }

        // main permission manager page link
        Rectangle {
            id: mainItem
            Layout.fillWidth: true
            Layout.preferredHeight: 52
            radius: 26
            property bool active: sideMenu.currentPage == "main"
            property bool hovered: false
            property bool pressed: false

            color: {
                if (mainItem.active == true) {
                    return Material.accent
                } else if (mainItem.pressed == true) {
                    return Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.18)
                } else if (mainItem.hovered == true) {
                    return Qt.rgba(1, 1, 1, 0.06)
                } else {
                    return "transparent"
                }
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                spacing: 12

                MaterialIcon {
                    name: "home"
                    tint: "white"
                }
                Label {
                    text: "Permission Manager"
                    color: "white"
                    font.pixelSize: 18
                    font.bold: mainItem.active
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                    verticalAlignment: Text.AlignVCenter
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: mainItem.hovered = true
                onExited: mainItem.hovered = false
                onPressed: mainItem.pressed = true
                onReleased: mainItem.pressed = false
                onClicked: {
                    sideMenu.pageSelected("main")
                    sideMenu.close()
                }
            }
        }

        // settings page link
        Rectangle {
            id: settingsItem
            Layout.fillWidth: true
            Layout.preferredHeight: 52
            radius: 26
            property bool active: sideMenu.currentPage == "settings"
            property bool hovered: false
            property bool pressed: false

            color: {
                if (settingsItem.active == true) {
                    return Material.accent
                } else if (settingsItem.pressed == true) {
                    return Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.18)
                } else if (settingsItem.hovered == true) {
                    return Qt.rgba(1, 1, 1, 0.06)
                } else {
                    return "transparent"
                }
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                spacing: 12

                MaterialIcon {
                    name: "settings"
                    tint: "white"
                }
                Label {
                    text: "Settings"
                    color: "white"
                    font.pixelSize: 18
                    font.bold: settingsItem.active
                    Layout.fillWidth: true
                    verticalAlignment: Text.AlignVCenter
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: settingsItem.hovered = true
                onExited: settingsItem.hovered = false
                onPressed: settingsItem.pressed = true
                onReleased: settingsItem.pressed = false
                onClicked: {
                    sideMenu.pageSelected("settings")
                    sideMenu.close()
                }
            }
        }

        // spacer to push about to bottom
        Rectangle {
            width: 1
            height: 1
            color: "transparent"
            Layout.fillHeight: true
        }

        // about page link
        Rectangle {
            id: aboutItem
            Layout.fillWidth: true
            Layout.preferredHeight: 52
            radius: 26
            property bool active: sideMenu.currentPage == "about"
            property bool hovered: false
            property bool pressed: false

            color: {
                if (aboutItem.active == true) {
                    return Material.accent
                } else if (aboutItem.pressed == true) {
                    return Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.18)
                } else if (aboutItem.hovered == true) {
                    return Qt.rgba(1, 1, 1, 0.06)
                } else {
                    return "transparent"
                }
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14
                spacing: 12

                MaterialIcon {
                    name: "info"
                    tint: "white"
                }
                Label {
                    text: "About"
                    color: "white"
                    font.pixelSize: 18
                    font.bold: aboutItem.active
                    Layout.fillWidth: true
                    verticalAlignment: Text.AlignVCenter
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: aboutItem.hovered = true
                onExited: aboutItem.hovered = false
                onPressed: aboutItem.pressed = true
                onReleased: aboutItem.pressed = false
                onClicked: {
                    sideMenu.pageSelected("about")
                    sideMenu.close()
                }
            }
        }
    }
}
