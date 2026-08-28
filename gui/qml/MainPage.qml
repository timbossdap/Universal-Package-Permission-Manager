import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// main permission manager view
ColumnLayout {
    id: mainPage
    anchors.fill: parent
    anchors.margins: 8
    spacing: 8

    property string selectedAppId: ""
    property alias searchText: searchBar.searchText
    property var filteredAppIds: []
    property AppSettings settings: null

    signal appSelected(string appId)

    SearchBar {
        id: searchBar
        Layout.fillWidth: true
        targetWidth: appPane.width
        selectedAppId: mainPage.selectedAppId
    }

    SplitView {
        id: listSplit
        Layout.fillWidth: true
        Layout.fillHeight: true
        orientation: Qt.Horizontal

        AppListPanel {
            id: appPane
            filteredAppIds: mainPage.filteredAppIds
            selectedAppId: mainPage.selectedAppId
            settings: mainPage.settings
            windowWidth: mainPage.Window.width

            onAppSelected: function(appId) {
                mainPage.appSelected(appId)
            }
        }

        PermissionPanel {
            id: permPane
            selectedAppId: mainPage.selectedAppId
            settings: mainPage.settings
        }
    }
}
