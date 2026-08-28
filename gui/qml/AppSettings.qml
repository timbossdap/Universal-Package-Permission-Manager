import QtQuick 2.15
import Qt.labs.settings 1.1

// persists to disk
Settings {
    id: settings

    property real savedLeftBoxWidth: -1
    property bool highlightHighRisk: true
    property bool compactRows: false
}
