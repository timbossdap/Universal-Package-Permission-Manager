import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    id: mainWindow
    visible: true
    width: 800
    height: 600
    title: "UPPM"

    Material.theme: Material.Dark
    Material.accent: Material.Teal

    AppSettings {
        id: settings
    }

    // tracking the selected app by its id string (not a numeric index) means
    // we don't have to worry about indices shifting around when the search
    // filter changes what's actually visible in the list
    property string selectedAppId: ""
    property int currentTabIndex: 0
    property string currentPage: "main"

    // filter app list based on search text
    property var filteredAppIds: {
        var ids = uppm.app_ids
        var needle = mainPage.searchText.toLowerCase()
        var out = []

        if (needle.length == 0) {
            return ids
        }

        var i = 0
        while (i < ids.length) {
            var id = ids[i]
            var lower = id.toLowerCase()
            var found = lower.indexOf(needle)
            if (found != -1) {
                out.push(id)
            }
            i = i + 1
        }
        return out
    }

    header: TitleBar {
        currentTabIndex: mainWindow.currentTabIndex
        onOpenMenuRequested: {
            sideMenu.open()
        }
        onTabSelected: function(index) {
            var cur = mainWindow.currentTabIndex
            if (cur == index) {
                return
            }
            mainWindow.currentTabIndex = index
            mainWindow.selectedAppId = ""
            mainPage.searchText = ""
            uppm.select_source(index)
        }
    }

    SideMenu {
        id: sideMenu
        currentPage: mainWindow.currentPage
        onPageSelected: function(page) {
            mainWindow.currentPage = page
        }
    }

    MainPage {
        id: mainPage
        visible: {
            if (mainWindow.currentPage == "main") {
                return true
            } else {
                return false
            }
        }
        selectedAppId: mainWindow.selectedAppId
        filteredAppIds: mainWindow.filteredAppIds
        settings: settings
        onAppSelected: function(appId) {
            mainWindow.selectedAppId = appId
            // still need the app's index in the *unfiltered* list,
            // since that's what the rust side's profiles vec uses
            var allIds = uppm.app_ids
            var idx = allIds.indexOf(appId)
            uppm.select_app(idx)
        }
    }

    SettingsPage {
        id: settingsPage
        visible: {
            if (mainWindow.currentPage == "settings") {
                return true
            } else {
                return false
            }
        }
        settings: settings
        onBackRequested: {
            mainWindow.currentPage = "main"
        }
    }

    AboutPage {
        id: aboutPage
        visible: {
            if (mainWindow.currentPage == "about") {
                return true
            } else {
                return false
            }
        }
        onBackRequested: {
            mainWindow.currentPage = "main"
        }
    }
}
