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
    Material.accent: {
        var scheme = settings.colorScheme
        switch (scheme) {
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

            case "Teal":
            default:
                return "#4DB6AC"
        }
    }

    AppSettings {
        id: settings
    }

    // tracking the selected app by its id string (not a numeric index) means
    // we don't have to worry about indices shifting around when the search
    // filter changes what's actually visible in the list
    property string selectedAppId: ""
    property string selectedSource: ""
    property int currentTabIndex: 0
    property string currentPage: "main"

    property var displayedSources: {
        if (settings.showAllTabs == true) {
            return uppm.all_sources
        } else if (uppm.available_sources.length > 0) {
            return uppm.available_sources
        } else {
            return uppm.all_sources
        }
    }

    property bool isCurrentSourceInstalled: {
        if (uppm.available_sources.length === 0) {
            return false
        }
        return uppm.available_sources.indexOf(selectedSource) !== -1
    }

    onDisplayedSourcesChanged: {
        var idx = displayedSources.indexOf(selectedSource)
        if (idx !== -1) {
            currentTabIndex = idx
        } else if (displayedSources.length > 0) {
            currentTabIndex = 0
            selectedSource = displayedSources[0]
            selectedAppId = ""
            if (mainPage) {
                mainPage.searchText = ""
            }
            uppm.select_source(selectedSource)
        }
    }

    // select the first available source on startup so the tab bar and
    // content are in sync from the moment the window appears
    Component.onCompleted: {
        if (displayedSources.length > 0) {
            var firstSource = displayedSources[0]
            mainWindow.selectedSource = firstSource
            mainWindow.currentTabIndex = 0
            uppm.select_source(firstSource)
        }
    }

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
        tabModel: mainWindow.displayedSources
        onOpenMenuRequested: {
            sideMenu.open()
        }
        onTabSelected: function(sourceName) {
            var cur = mainWindow.currentTabIndex
            // find the index of this source in the tab model
            var idx = mainWindow.displayedSources.indexOf(sourceName)
            if (cur == idx && mainWindow.selectedSource == sourceName) {
                return
            }
            mainWindow.selectedSource = sourceName
            mainWindow.currentTabIndex = idx
            mainWindow.selectedAppId = ""
            mainPage.searchText = ""
            uppm.select_source(sourceName)
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
        selectedSource: mainWindow.selectedSource
        isSourceInstalled: mainWindow.isCurrentSourceInstalled
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
