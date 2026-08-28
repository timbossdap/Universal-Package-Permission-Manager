import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.15

// about page
ColumnLayout {
    id: aboutPage
    anchors.fill: parent
    anchors.margins: 8
    spacing: 16

    signal backRequested()

    RowLayout {
        Layout.fillWidth: true
        spacing: 8

        ToolButton {
            contentItem: MaterialIcon {
                anchors.centerIn: parent
                name: "arrow_back"
                tint: "white"
            }
            onClicked: aboutPage.backRequested()
        }

        Label {
            text: "About"
            font.pixelSize: 24
            font.bold: true
        }

        Item { Layout.fillWidth: true }
    }

    Frame {
        Layout.fillWidth: true
        Layout.fillHeight: true
        padding: 24

        background: Rectangle {
            color: "transparent"
            border.color: Material.accent
            border.width: 1
            radius: 10
        }

        ColumnLayout {
            width: parent.width
            spacing: 20

            RowLayout {
                Layout.fillWidth: true
                spacing: 16

                Rectangle {
                    width: 56
                    height: 56
                    radius: 14
                    color: Material.accent

                    MaterialIcon {
                        anchors.centerIn: parent
                        name: "home"
                        tint: "white"
                        size: 32
                    }
                }

                ColumnLayout {
                    spacing: 2
                    Label {
                        text: "Universal Package Permission Manager"
                        font.pixelSize: 20
                        font.bold: true
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                    Label {
                        text: "UPPM · version 0.1.0"
                        color: "gray"
                        font.pixelSize: 14
                    }
                }
            }

            Label {
                text: "An all-in-one solution to view and manage your permissions from different package managers such as Flatpak and Pacman, through a modern minimalist GUI built with Rust and QML, with the beautiful Material themeing."
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                font.pixelSize: 15
            }

            ColumnLayout {
                spacing: 6
                Layout.topMargin: 8

                Label {
                    text: "Limitations"
                    font.pixelSize: 16
                    font.bold: true
                }
                Label {
                    text: "Pacman does not expose an extensive list of what packages use what permissions, which makes it harder to fetch accurate permission data for apps installed that way."
                    color: "gray"
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                    font.pixelSize: 14
                }
            }

            Item { Layout.fillHeight: true }
        }
    }
}
