import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import "../.."

Frame {
    id: permPane

    property string selectedAppId: ""
    property string selectedSource: ""
    property bool isSourceInstalled: true
    property AppSettings settings: null

    SplitView.fillWidth: true
    padding: 0

    background: Rectangle {
        color: "transparent"
        border.color: Material.accent
        border.width: 1
        radius: 10
    }

    ListView {
        id: permList
        anchors.fill: parent
        anchors.margins: 1
        model: uppm.permissions
        clip: true

        delegate: Rectangle {
            id: permDelegate
            width: permList.width
            color: "transparent"

            height: {
                if (permPane.settings != null) {
                    if (permPane.settings.compactRows == true) {
                        return 36
                    }
                }
                return 48
            }

            // check if permission is marked high risk on rust side
            property bool hiRisk: {
                if (permPane.settings == null) {
                    return false
                }
                var highlight = permPane.settings.highlightHighRisk
                if (highlight != true) {
                    return false
                }
                var riskList = uppm.permissions_hi_risk
                var len = riskList.length
                if (index < 0) {
                    return false
                }
                if (index >= len) {
                    return false
                }
                var val = riskList[index]
                if (val == true) {
                    return true
                } else {
                    return false
                }
            }

            Label {
                text: modelData
                color: {
                    if (permDelegate.hiRisk == true) {
                        return "#ff6b6b"
                    } else {
                        return "white"
                    }
                }
                font.bold: permDelegate.hiRisk
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 12
                anchors.rightMargin: 12
            }
        }

        Label {
            anchors.centerIn: parent
            visible: {
                var loading = uppm.loading
                var count = permList.count
                if (loading == false && count == 0) {
                    return true
                } else {
                    return false
                }
            }
            text: {
                if (permPane.isSourceInstalled === false) {
                    return "manager not installed"
                }
                var app = permPane.selectedAppId
                if (app == "") {
                    return "select an app on the left to see its permissions"
                } else {
                    return "no permissions found"
                }
            }
            color: "gray"
        }
    }

    // loading indicator while rust collectors run
    Column {
        anchors.centerIn: parent
        spacing: 12
        visible: uppm.loading

        BusyIndicator {
            anchors.horizontalCenter: parent.horizontalCenter
            running: uppm.loading
        }

        Label {
            text: "scanning installed apps..."
            color: "gray"
        }
    }
}
