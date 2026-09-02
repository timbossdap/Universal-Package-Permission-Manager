import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15
import ".."

Item {
    id: dropdown
    implicitWidth: 200
    implicitHeight: 56

    property string label: "Scheme"
    property var model: [
        "Seafoam",
        "Abyss",
        "Glacier",
        "Midnight",
        "Lavender",
        "Obsidian",
        "Mint",
        "Forest",
        "Sunrise",
        "Ember",
        "Coral",
        "Crimson",
        "Rose Quartz",
        "Velvet",
        "Silver Mist",
        "Gunpowder"
    ]
    property string currentValue: "Seafoam"
    property color notchColor: Material.background

    signal valueSelected(string value)

    function getColor(name) {
        switch (name) {
            // Teal
            case "Seafoam": return "#4DB6AC"
            case "Abyss": return "#00695C"

            // Blue
            case "Glacier": return "#64B5F6"
            case "Midnight": return "#1565C0"

            // Purple
            case "Lavender": return "#BA68C8"
            case "Obsidian": return "#4A148C"

            // Green
            case "Mint": return "#81C784"
            case "Forest": return "#2E7D32"

            // Orange
            case "Sunrise": return "#FFB74D"
            case "Ember": return "#D84315"

            // Red
            case "Coral": return "#E57373"
            case "Crimson": return "#C62828"

            // Pink
            case "Rose Quartz": return "#F06292"
            case "Velvet": return "#AD1457"

            // Grey
            case "Silver Mist": return "#B0BEC5"
            case "Gunpowder": return "#455A64"

            // Fallbacks
            case "Teal":
            default:
                return "#4DB6AC"
        }
    }

    // Main outlined box container
    Rectangle {
        id: boxRect
        anchors.fill: parent
        anchors.topMargin: 6
        radius: 8
        color: "transparent"
        border.color: dropdownPopup.visible ? Material.accent : (boxMouseArea.containsMouse ? Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.6) : "#5a5a6e")
        border.width: dropdownPopup.visible ? 2 : 1.5

        Behavior on border.color { ColorAnimation { duration: 150 } }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 14
            anchors.rightMargin: 12
            spacing: 10

            // Preview circle for current selected color
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: dropdown.getColor(dropdown.currentValue)
                border.color: Qt.rgba(1, 1, 1, 0.4)
                border.width: 1
                Layout.alignment: Qt.AlignVCenter
            }

            Label {
                text: dropdown.currentValue
                color: "white"
                font.pixelSize: 16
                font.bold: true
                Layout.fillWidth: true
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
            }

            MaterialIcon {
                name: dropdownPopup.visible ? "arrow_drop_up" : "arrow_drop_down"
                tint: dropdownPopup.visible ? Material.accent : "#cfcfcf"
                size: 24
            }
        }

        MouseArea {
            id: boxMouseArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                if (dropdownPopup.visible) {
                    dropdownPopup.close()
                } else {
                    dropdownPopup.open()
                }
            }
        }
    }

    // Floating Notched Label
    Rectangle {
        x: 12
        y: 0
        height: labelText.implicitHeight
        width: labelText.implicitWidth + 8
        color: dropdown.notchColor

        Label {
            id: labelText
            anchors.centerIn: parent
            text: dropdown.label
            font.pixelSize: 12
            font.bold: true
            color: dropdownPopup.visible ? Material.accent : "#a0a0b0"

            Behavior on color { ColorAnimation { duration: 150 } }
        }
    }

    // Dropdown Popup
    Popup {
        id: dropdownPopup
        y: dropdown.height + 4
        width: Math.max(dropdown.width, 210)
        padding: 8
        modal: false
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: "#25242a"
            radius: 16
            border.color: Qt.rgba(1, 1, 1, 0.14)
            border.width: 1

            Rectangle {
                anchors.fill: parent
                radius: 16
                color: "transparent"
                border.color: Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.35)
                border.width: 1
            }
        }

        contentItem: ListView {
            id: popupListView
            implicitHeight: Math.min(contentHeight, 280)
            clip: true
            model: dropdown.model
            spacing: 4

            ScrollBar.vertical: ScrollBar {
                policy: popupListView.contentHeight > 280 ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
            }

            delegate: Rectangle {
                id: itemDelegate
                width: popupListView.width - (popupListView.contentHeight > 280 ? 12 : 0)
                height: 40
                radius: 12

                property bool isSelected: modelData === dropdown.currentValue
                property bool isHovered: false

                color: {
                    if (isSelected) {
                        return Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.3)
                    } else if (isHovered) {
                        return Qt.rgba(1, 1, 1, 0.08)
                    } else {
                        return "transparent"
                    }
                }

                Behavior on color { ColorAnimation { duration: 120 } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 8

                    MaterialIcon {
                        name: "check"
                        size: 18
                        tint: Material.accent
                        visible: itemDelegate.isSelected
                        Layout.preferredWidth: 18
                    }

                    Item {
                        visible: !itemDelegate.isSelected
                        Layout.preferredWidth: 18
                        Layout.preferredHeight: 18
                    }

                    // Preview circle for this color item
                    Rectangle {
                        width: 14
                        height: 14
                        radius: 7
                        color: dropdown.getColor(modelData)
                        border.color: Qt.rgba(1, 1, 1, 0.3)
                        border.width: 1
                        Layout.alignment: Qt.AlignVCenter
                    }

                    Label {
                        text: modelData
                        color: itemDelegate.isSelected ? "white" : "#e0e0e0"
                        font.pixelSize: 15
                        font.bold: itemDelegate.isSelected
                        Layout.fillWidth: true
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onEntered: itemDelegate.isHovered = true
                    onExited: itemDelegate.isHovered = false
                    onClicked: {
                        dropdown.currentValue = modelData
                        dropdown.valueSelected(modelData)
                        dropdownPopup.close()
                    }
                }
            }
        }
    }
}
