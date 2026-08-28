import QtQuick 2.15

// renders a material icon from the ttf font
Item {
    id: icon

    property string name: ""
    property color tint: "white"
    property int size: 24

    width: icon.size
    height: icon.size

    FontLoader {
        id: materialFont
        source: "MaterialIcons-Regular.ttf"
    }

    Text {
        anchors.centerIn: parent
        text: icon.name
        color: icon.tint
        font.pixelSize: icon.size
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter

        font.family: {
            if (materialFont.status == FontLoader.Ready) {
                var fname = materialFont.name
                return fname
            } else {
                return "sans-serif"
            }
        }
    }
}
