import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import ".."

// search bar row
RowLayout {
    id: searchBar
    spacing: 8

    property alias searchText: searchInput.text
    property real targetWidth: 200
    property string selectedAppId: ""

    Item {
        id: searchContainer
        Layout.preferredWidth: searchBar.targetWidth
        Layout.preferredHeight: 46

        // Pill outline border
        Rectangle {
            id: searchBg
            anchors.fill: parent
            anchors.topMargin: 6
            radius: height / 2
            color: Qt.rgba(1, 1, 1, 0.04)
            border.color: searchInput.activeFocus ? Material.accent : Qt.rgba(1, 1, 1, 0.22)
            border.width: searchInput.activeFocus ? 2 : 1

            Behavior on border.color { ColorAnimation { duration: 150 } }

            MouseArea {
                anchors.fill: parent
                onClicked: searchInput.forceActiveFocus()
            }
        }

        // Notch cutout opening a hole in the top pill border
        Rectangle {
            id: notchMask
            x: 16
            y: 6 - height / 2
            height: 6
            width: floatingLabel.implicitWidth + 8
            color: Material.background
            visible: floatingLabel.visible
        }

        // Floating label sitting in the hole
        Label {
            id: floatingLabel
            x: 20
            anchors.verticalCenter: notchMask.verticalCenter
            text: "search apps..."
            font.pixelSize: 11
            font.bold: true
            color: searchInput.activeFocus ? Material.accent : "#888888"
            visible: searchInput.activeFocus || searchInput.text.length > 0

            Behavior on color { ColorAnimation { duration: 150 } }
        }

        // Search text input
        TextInput {
            id: searchInput
            anchors.fill: parent
            anchors.topMargin: 6
            anchors.leftMargin: 16
            anchors.rightMargin: 36
            verticalAlignment: TextInput.AlignVCenter
            color: "white"
            font.pixelSize: 14
            selectByMouse: true
            clip: true

            // Resting placeholder text when unfocused and empty
            Label {
                anchors.fill: parent
                verticalAlignment: Text.AlignVCenter
                text: "search apps..."
                color: "#757575"
                font.pixelSize: 14
                visible: !searchInput.activeFocus && searchInput.text.length === 0
            }
        }

        // Clear button (X)
        Rectangle {
            id: clearBtn
            width: 24
            height: 24
            radius: 12
            color: "transparent"
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: searchBg.verticalCenter

            visible: searchInput.text.length > 0

            MaterialIcon {
                anchors.centerIn: parent
                name: "close"
                tint: Material.accent
                size: 16
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    searchInput.text = ""
                    searchInput.forceActiveFocus()
                }
            }
        }
    }

    // Spacer so the pill goes to the right
    Rectangle {
        Layout.fillWidth: true
        height: 1
        color: "transparent"
    }

    // Badge for currently selected app
    Rectangle {
        id: selectedPill
        visible: searchBar.selectedAppId !== ""
        radius: height / 2
        Layout.preferredWidth: pillLabel.implicitWidth + 28
        Layout.preferredHeight: 32
        color: Material.accent

        Label {
            id: pillLabel
            anchors.centerIn: parent
            text: searchBar.selectedAppId
            color: "white"
            font.pixelSize: 13
        }
    }
}
