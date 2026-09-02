import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import "../.."

Frame {
    id: appPane

    property var filteredAppIds: []
    property string selectedAppId: ""
    property string selectedSource: ""
    property bool isSourceInstalled: true
    property AppSettings settings: null
    property real windowWidth: 800

    signal appSelected(string appId)

    // 1/3 of the page until user says otherwise with handelbar drag
    SplitView.preferredWidth: {
        var saved = 0
        if (appPane.settings != null) {
            saved = appPane.settings.savedLeftBoxWidth
        }
        if (saved > 0) {
            return saved
        } else {
            var third = appPane.windowWidth / 3
            return third
        }
    }
    SplitView.minimumWidth: 160
    padding: 0

    onWidthChanged: {
        if (appPane.settings != null) {
            appPane.settings.savedLeftBoxWidth = width
        }
    }

    background: Rectangle {
        color: "transparent"
        border.color: Material.accent
        border.width: 1
        radius: 10
    }

    ListView {
        id: appList
        anchors.fill: parent
        anchors.margins: 8
        spacing: 6
        model: appPane.filteredAppIds
        clip: true

        delegate: Rectangle {
            id: appDelegate
            width: appList.width
            height: {
                if (appPane.settings != null) {
                    if (appPane.settings.compactRows == true) {
                        return 36
                    }
                }
                return 48
            }
            radius: height / 2

            property bool isSelected: modelData == appPane.selectedAppId
            property bool isHovered: false
            property bool isPressed: false

            color: {
                if (isSelected == true) {
                    return Material.accent
                } else if (isPressed == true) {
                    return Qt.rgba(Material.accentColor.r, Material.accentColor.g, Material.accentColor.b, 0.18)
                } else if (isHovered == true) {
                    return Qt.rgba(1, 1, 1, 0.06)
                } else {
                    return Qt.rgba(1, 1, 1, 0.04)
                }
            }

            Behavior on color { ColorAnimation { duration: 120 } }

            Label {
                text: modelData
                color: "white"
                font.bold: appDelegate.isSelected
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 16
                anchors.rightMargin: 16
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: appDelegate.isHovered = true
                onExited: appDelegate.isHovered = false
                onPressed: appDelegate.isPressed = true
                onReleased: appDelegate.isPressed = false
                onClicked: {
                    var id = modelData
                    appPane.appSelected(id)
                }
            }
        }

        Label {
            anchors.centerIn: parent
            visible: !uppm.loading && !appPane.isSourceInstalled
            text: "manager not installed"
            color: "gray"
        }
    }
}
