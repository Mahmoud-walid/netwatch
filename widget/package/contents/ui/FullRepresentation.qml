import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.plasma.components as PlasmaComponents

Item {
    id: fullRoot

    function formatSpeed(bps) {
        if (!bps)
            return "0.00 Mbps";

        return ((bps * 8) / 1e+06).toFixed(2) + " Mbps";
    }

    function formatBytes(bytes) {
        if (bytes === 0)
            return "0 B";

        var k = 1024;
        var sizes = ["B", "KB", "MB", "GB", "TB"];
        var i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    Layout.minimumWidth: Kirigami.Units.gridUnit * 26
    Layout.minimumHeight: Kirigami.Units.gridUnit * 24
    Layout.preferredWidth: Kirigami.Units.gridUnit * 28
    Layout.preferredHeight: Kirigami.Units.gridUnit * 28

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // 1. Header (السرعة الإجمالية الحية)
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Kirigami.Units.gridUnit * 4
            color: Kirigami.Theme.highlightColor

            RowLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.largeSpacing

                Kirigami.Icon {
                    source: "network-workgroup"
                    color: Kirigami.Theme.highlightedTextColor
                    Layout.preferredWidth: Kirigami.Units.iconSizes.medium
                    Layout.preferredHeight: Kirigami.Units.iconSizes.medium
                }

                ColumnLayout {
                    spacing: 0

                    PlasmaComponents.Label {
                        text: "NetWatch Monitor"
                        font.bold: true
                        font.pixelSize: 16
                        color: Kirigami.Theme.highlightedTextColor
                    }

                    PlasmaComponents.Label {
                        text: "Live Network Traffic"
                        font.pixelSize: 12
                        color: Kirigami.Theme.highlightedTextColor
                        opacity: 0.8
                    }

                }

                // Spacer
                Item {
                    Layout.fillWidth: true
                }

                ColumnLayout {
                    Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
                    spacing: 2

                    PlasmaComponents.Label {
                        text: "↓ " + formatSpeed(root.totalDl)
                        color: Kirigami.Theme.highlightedTextColor
                        font.bold: true
                        font.family: "monospace"
                        font.pixelSize: 14
                    }

                    PlasmaComponents.Label {
                        text: "↑ " + formatSpeed(root.totalUl)
                        color: Kirigami.Theme.highlightedTextColor
                        font.bold: true
                        font.family: "monospace"
                        font.pixelSize: 14
                    }

                }

            }

        }

        // 2. نظام التبويبات (Tabs)
        TabBar {
            id: bar

            Layout.fillWidth: true

            TabButton {
                text: qsTr("Devices")
            }

            TabButton {
                text: qsTr("History & Usage")
            }

            TabButton {
                text: qsTr("Settings")
            }

        }

        StackLayout {
            currentIndex: bar.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true

            // --------- TAB 1: الأجهزة المتصلة ---------
            Item {
                ListView {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.smallSpacing
                    clip: true
                    spacing: Kirigami.Units.smallSpacing
                    model: root.devices

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: Kirigami.Units.gridUnit * 3.5
                        color: index % 2 === 0 ? Kirigami.Theme.alternateBackgroundColor : "transparent"
                        radius: Kirigami.Units.smallSpacing
                        border.color: Kirigami.Theme.separatorColor
                        border.width: 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Kirigami.Units.smallSpacing
                            spacing: Kirigami.Units.largeSpacing

                            // نقطة حالة الاتصال
                            Rectangle {
                                width: 12
                                height: 12
                                radius: 6
                                color: modelData.is_online ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.negativeTextColor
                                Layout.alignment: Qt.AlignVCenter
                            }

                            // معلومات الجهاز
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0

                                PlasmaComponents.Label {
                                    text: modelData.display_name || modelData.hostname || modelData.vendor || "Unknown Device"
                                    font.bold: true
                                    elide: Text.ElideRight
                                }

                                PlasmaComponents.Label {
                                    text: "IP: " + (modelData.ip_address ? modelData.ip_address : "N/A") + " | MAC: " + modelData.mac_address
                                    font.pixelSize: Kirigami.Theme.smallFont.pixelSize
                                    color: Kirigami.Theme.disabledTextColor
                                }

                            }

                            // السرعة الحية
                            ColumnLayout {
                                property var deviceSpeed: root.liveSpeeds[modelData.mac_address]

                                Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
                                spacing: 0

                                PlasmaComponents.Label {
                                    text: "↓ " + formatSpeed(deviceSpeed ? deviceSpeed.rx_bps : 0)
                                    color: Kirigami.Theme.positiveTextColor
                                    font.family: "monospace"
                                    font.pixelSize: 12
                                }

                                PlasmaComponents.Label {
                                    text: "↑ " + formatSpeed(deviceSpeed ? deviceSpeed.tx_bps : 0)
                                    color: Kirigami.Theme.negativeTextColor
                                    font.family: "monospace"
                                    font.pixelSize: 12
                                }

                            }

                        }

                    }

                }

            }

            // --------- TAB 2: الاستهلاك التاريخي (Total Usage) ---------
            Item {
                ListView {
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.smallSpacing
                    clip: true
                    spacing: Kirigami.Units.smallSpacing
                    model: root.devices

                    delegate: Rectangle {
                        width: ListView.view.width
                        height: Kirigami.Units.gridUnit * 3.5
                        color: Kirigami.Theme.backgroundColor
                        radius: Kirigami.Units.smallSpacing
                        border.color: Kirigami.Theme.separatorColor
                        border.width: 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Kirigami.Units.smallSpacing
                            spacing: Kirigami.Units.largeSpacing

                            Kirigami.Icon {
                                source: "drive-harddisk"
                                color: Kirigami.Theme.textColor
                                Layout.preferredWidth: 24
                                Layout.preferredHeight: 24
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0

                                PlasmaComponents.Label {
                                    text: modelData.display_name || modelData.vendor || modelData.mac_address
                                    font.bold: true
                                    elide: Text.ElideRight
                                }

                                PlasmaComponents.Label {
                                    text: "Total Recorded Usage"
                                    font.pixelSize: Kirigami.Theme.smallFont.pixelSize
                                    color: Kirigami.Theme.disabledTextColor
                                }

                            }

                            ColumnLayout {
                                Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
                                spacing: 0

                                PlasmaComponents.Label {
                                    text: "DL: " + formatBytes(modelData.last_rx_bytes)
                                    color: Kirigami.Theme.positiveTextColor
                                    font.bold: true
                                    font.family: "monospace"
                                }

                                PlasmaComponents.Label {
                                    text: "UL: " + formatBytes(modelData.last_tx_bytes)
                                    color: Kirigami.Theme.negativeTextColor
                                    font.bold: true
                                    font.family: "monospace"
                                }

                            }

                        }

                    }

                }

            }

            // --------- TAB 3: الإعدادات ---------
            Item {
                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Kirigami.Units.largeSpacing

                    Kirigami.Icon {
                        source: "configure"
                        Layout.alignment: Qt.AlignHCenter
                        Layout.preferredWidth: 64
                        Layout.preferredHeight: 64
                        opacity: 0.5
                    }

                    PlasmaComponents.Label {
                        text: "NetWatch Configuration"
                        font.bold: true
                        font.pixelSize: 18
                        Layout.alignment: Qt.AlignHCenter
                    }

                    PlasmaComponents.Label {
                        text: "More charts and settings coming soon in v1.1"
                        color: Kirigami.Theme.disabledTextColor
                        Layout.alignment: Qt.AlignHCenter
                    }

                }

            }

        }

    }

}
