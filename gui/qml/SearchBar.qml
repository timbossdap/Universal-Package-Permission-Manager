import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

// search bar row
RowLayout {
    id: searchBar
    spacing: 8

    property alias searchText: searchField.text
    property real targetWidth: 200
    property string selectedAppId: ""

    // search input - using a regular rectangle instead of textfield style
    TextField {
        id: searchField
        Layout.preferredWidth: searchBar.targetWidth
        Layout.fillHeight: false
        placeholderText: "search apps..."
        rightPadding: 32
        leftPadding: 16

        background: Rectangle {
            radius: height / 2
            color: Qt.rgba(1, 1, 1, 0.06)
            border.color: Material.accent
            border.width: 1
        }

        // clear button - put it inside the textfield manually
        Rectangle {
            id: clearBtn
            width: 20
            height: 20
            radius: 10
            color: "transparent"
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter

            visible: {
                var txt = searchField.text
                var len = txt.length
                if (len > 0) {
                    return true
                } else {
                    return false
                }
            }

            MaterialIcon {
                anchors.centerIn: parent
                name: "close"
                tint: Material.accent
                size: 16
            }

            MouseArea {
                anchors.fill: parent
                anchors.margins: -8
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    searchField.text = ""
                }
            }
        }
    }

    // spacer so the pill goes to the right
    Rectangle {
        Layout.fillWidth: true
        height: 1
        color: "transparent"
    }

    // badge for currently selected app
    Rectangle {
        id: selectedPill
        visible: {
            var app = searchBar.selectedAppId
            if (app != "") {
                return true
            } else {
                return false
            }
        }
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
