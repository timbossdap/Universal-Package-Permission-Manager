import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Window {
    visible: true
    width: 800
    height: 600
    title: "UPPM"
    color: "black"

    property int selectedIndex: -1
    
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        TabBar {
            id: sourceTabs
            Layout.fillWidth: true
            TabButton { text: "Flatpak" }
            TabButton { text: "Pacman" }
            onCurrentIndexChanged: {
                selectedIndex = -1
                uppm.select_source(currentIndex)
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 1

            ListView {
                id: appList
                Layout.preferredWidth: parent.width / 2
                Layout.fillHeight: true
                model: uppm.app_ids
                clip: true

                delegate: ItemDelegate {
                    width: appList.width
                    highlighted: index === selectedIndex
                    text: (index + 1) + ". " + modelData
                    onClicked: {
                        selectedIndex = index
                        uppm.select_app(index)
                    }
                }
            }

            ListView {
                id: permList
                Layout.preferredWidth: parent.width / 2
                Layout.fillHeight: true
                model: uppm.permissions
                clip: true

                delegate: ItemDelegate {
                    width: permList.width
                    text: modelData
                }

                Label {
                    anchors.centerIn: parent
                    visible: permList.count === 0
                    text: "select an app on the left to see its permissions"
                    color: "gray"
                }
            }
        }
    }
}
