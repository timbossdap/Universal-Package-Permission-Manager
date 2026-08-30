import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

// settings page
ColumnLayout {
    id: settingsPage
    anchors.fill: parent
    anchors.margins: 8
    spacing: 16

    property AppSettings settings: null

    signal backRequested()

    RowLayout {
        Layout.fillWidth: true
        spacing: 8

        // back button
        Rectangle {
            id: backBtn
            width: 40
            height: 40
            color: "transparent"
            radius: 20

            MaterialIcon {
                anchors.centerIn: parent
                name: "arrow_back"
                tint: "white"
            }

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    settingsPage.backRequested()
                }
            }
        }

        Label {
            text: "Settings"
            font.pixelSize: 24
            font.bold: true
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: "transparent" }
    }

    Frame {
        Layout.fillWidth: true
        Layout.fillHeight: true
        padding: 20

        background: Rectangle {
            color: "transparent"
            border.color: Material.accent
            border.width: 1
            radius: 10
        }

        ColumnLayout {
            width: parent.width
            spacing: 18

            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: "Highlight high-risk permissions"; font.pixelSize: 17 }
                    Label {
                        text: "Flag hardware and network access in the permissions list"
                        color: "gray"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
                Switch {
                    checked: {
                        if (settingsPage.settings != null) {
                            return settingsPage.settings.highlightHighRisk
                        } else {
                            return true
                        }
                    }
                    onToggled: {
                        if (settingsPage.settings != null) {
                            settingsPage.settings.highlightHighRisk = checked
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: "Auto-refresh on launch"; font.pixelSize: 17 }
                    Label {
                        text: "Rescan Flatpak, Pacman, and Homebrew every time UPPM opens"
                        color: "gray"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
                // saved to rust preference file on toggle
                Switch {
                    checked: uppm.auto_refresh_on_launch
                    onToggled: {
                        var val = checked
                        uppm.set_auto_refresh_on_launch(val)
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: "Compact list rows"; font.pixelSize: 17 }
                    Label {
                        text: "Tighter spacing in the app and permission lists"
                        color: "gray"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
                Switch {
                    checked: {
                        if (settingsPage.settings != null) {
                            return settingsPage.settings.compactRows
                        } else {
                            return false
                        }
                    }
                    onToggled: {
                        if (settingsPage.settings != null) {
                            settingsPage.settings.compactRows = checked
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: "Show all package manager tabs"; font.pixelSize: 17 }
                    Label {
                        text: "Display tabs for all supported package managers even if not installed"
                        color: "gray"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
                Switch {
                    checked: {
                        if (settingsPage.settings != null) {
                            return settingsPage.settings.showAllTabs
                        } else {
                            return false
                        }
                    }
                    onToggled: {
                        if (settingsPage.settings != null) {
                            settingsPage.settings.showAllTabs = checked
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: "Color scheme"; font.pixelSize: 17 }
                    Label {
                        text: "Change the accent color theme of the interface"
                        color: "gray"
                        font.pixelSize: 14
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
                OutlinedDropdown {
                    label: "Scheme"
                    currentValue: {
                        if (settingsPage.settings != null && settingsPage.settings.colorScheme) {
                            return settingsPage.settings.colorScheme
                        } else {
                            return "Seafoam"
                        }
                    }
                    model: [
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
                    notchColor: Material.background
                    onValueSelected: function(val) {
                        if (settingsPage.settings != null) {
                            settingsPage.settings.colorScheme = val
                        }
                    }
                }
            }

            Rectangle { Layout.fillHeight: true; width: 1; color: "transparent" }
        }
    }
}
