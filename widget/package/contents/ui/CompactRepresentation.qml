import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Item {
    id: compactRoot

    // تحويل البايت إلى ميجابت للعرض السريع
    function formatSpeed(bps) {
        if (!bps)
            return "0.0";

        return ((bps * 8) / 1e+06).toFixed(1);
    }

    Layout.minimumWidth: Kirigami.Units.iconSizes.smallMedium
    Layout.minimumHeight: Kirigami.Units.iconSizes.smallMedium

    RowLayout {
        anchors.fill: parent
        spacing: Kirigami.Units.smallSpacing

        Kirigami.Icon {
            source: "network-workgroup"
            Layout.preferredWidth: Kirigami.Units.iconSizes.smallMedium
            Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
        }

        ColumnLayout {
            spacing: 0
            visible: root.totalDl > 0 || root.totalUl > 0

            Text {
                text: "↓ " + formatSpeed(root.totalDl)
                color: Kirigami.Theme.positiveTextColor
                font.pixelSize: 10
            }

            Text {
                text: "↑ " + formatSpeed(root.totalUl)
                color: Kirigami.Theme.negativeTextColor
                font.pixelSize: 10
            }

        }

    }

}
