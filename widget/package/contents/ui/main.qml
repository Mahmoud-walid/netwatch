import QtQuick
import QtQuick.Layouts
import QtWebSockets
import org.kde.plasma.plasmoid

PlasmoidItem {
    id: root

    // نموذج البيانات الداخلي للواجهة
    property var devices: []
    property var liveSpeeds: ({
    })
    property int totalDl: 0
    property int totalUl: 0

    // الاتصال بخادم الـ API لجلب قائمة الأجهزة الأساسية
    function fetchDevices() {
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "http://127.0.0.1:3030/api/v1/devices");
        xhr.onreadystatechange = function() {
            if (xhr.readyState === XMLHttpRequest.DONE && xhr.status === 200) {
                var response = JSON.parse(xhr.responseText);
                root.devices = response.devices;
            }
        };
        xhr.send();
    }

    Component.onCompleted: {
        fetchDevices();
    }

    // قناة البث الحية (WebSocket)
    WebSocket {
        id: socket

        url: "ws://127.0.0.1:3030/api/v1/live"
        active: true
        onTextMessageReceived: function(message) {
            var data = JSON.parse(message);
            root.liveSpeeds = data;
            // حساب إجمالي السرعة للشبكة ككل
            var tempDl = 0;
            var tempUl = 0;
            for (var mac in data) {
                tempDl += data[mac].rx_bps;
                tempUl += data[mac].tx_bps;
            }
            root.totalDl = tempDl;
            root.totalUl = tempUl;
        }
        onStatusChanged: {
            if (socket.status === WebSocket.Error)
                console.log("WebSocket Error: " + socket.errorString);

        }
    }

    // الواجهة المصغرة
    compactRepresentation: CompactRepresentation {
    }

    // الواجهة الكاملة (الداشبورد)
    fullRepresentation: FullRepresentation {
    }

}
