package com.example.virtualgamepad.data

import android.content.Context
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import okhttp3.*
import okio.ByteString
import org.json.JSONObject
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.util.concurrent.TimeUnit
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

object NetworkManager {
    private var webSocket: WebSocket? = null
    private var udpSocket: DatagramSocket? = null
    private var serverAddress: InetAddress? = null
    
    var isConnected by mutableStateOf(false)
        private set
    var isApproved by mutableStateOf(false)
        private set
    var connectionStatus by mutableStateOf("Disconnected")
        private set
        
    private var deviceId: String? = null
    private var prefs: android.content.SharedPreferences? = null

    // Callbacks
    var onRumble: ((Long) -> Unit)? = null
    
    fun init(context: Context) {
        prefs = context.getSharedPreferences("VirtualGamepadPrefs", Context.MODE_PRIVATE)
        deviceId = prefs?.getString("device_id", null)
        
        if (deviceId == null) {
            val chars = "abcdefghijklmnopqrstuvwxyz0123456789"
            val randomString = (1..7).map { chars.random() }.joinToString("")
            deviceId = "android_$randomString"
            prefs?.edit()?.putString("device_id", deviceId)?.apply()
        }
    }

    fun getSavedUrls(): List<String> {
        return prefs?.getStringSet("saved_urls", emptySet())?.toList() ?: emptyList()
    }

    fun saveUrl(url: String) {
        val current = prefs?.getStringSet("saved_urls", emptySet())?.toMutableSet() ?: mutableSetOf()
        current.add(url)
        prefs?.edit()?.putStringSet("saved_urls", current)?.apply()
    }

    fun connect(serverUrl: String) {
        if (isConnected) return
        
        try {
            val host = serverUrl.split(":")[0]
            
            GlobalScope.launch(Dispatchers.IO) {
                try {
                    serverAddress = InetAddress.getByName(host)
                    udpSocket = DatagramSocket()
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }

            val client = OkHttpClient.Builder()
                .pingInterval(5, TimeUnit.SECONDS)
                .build()

            val request = Request.Builder()
                .url("ws://$serverUrl/ws")
                .build()

            webSocket = client.newWebSocket(request, object : WebSocketListener() {
                override fun onOpen(webSocket: WebSocket, response: Response) {
                    Log.d("NetworkManager", "WebSocket Opened")
                    isConnected = true
                    connectionStatus = "Waiting for approval..."
                    
                    val reqConn = "[{\"t\":\"req_conn\",\"id\":\"$deviceId\"}]"
                    webSocket.send(reqConn)
                }

                override fun onMessage(webSocket: WebSocket, text: String) {
                    Log.d("NetworkManager", "Message: $text")
                    try {
                        val jsonArray = org.json.JSONArray(text)
                        for (i in 0 until jsonArray.length()) {
                            val json = jsonArray.getJSONObject(i)
                            if (json.has("t")) {
                                when (json.getString("t")) {
                                    "approved" -> {
                                        isApproved = true
                                        connectionStatus = "Connected & Approved!"
                                    }
                                    "rejected" -> {
                                        isApproved = false
                                        connectionStatus = "Connection Rejected"
                                    }
                                    "rumble" -> {
                                        if (json.has("v")) {
                                            onRumble?.invoke(json.getLong("v"))
                                        }
                                    }
                                }
                            }
                        }
                    } catch (e: Exception) {
                        e.printStackTrace()
                    }
                }

                override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                    Log.d("NetworkManager", "WebSocket Closed: $reason")
                    isConnected = false
                    isApproved = false
                    connectionStatus = "Disconnected"
                }

                override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                    Log.e("NetworkManager", "WebSocket Error", t)
                    isConnected = false
                    isApproved = false
                    connectionStatus = "Connection Error"
                }
            })

        } catch (e: Exception) {
            e.printStackTrace()
            connectionStatus = "Error: ${e.message}"
        }
    }

    fun disconnect() {
        webSocket?.close(1000, "User disconnected")
        webSocket = null
        udpSocket?.close()
        udpSocket = null
        serverAddress = null
        isConnected = false
        isApproved = false
        connectionStatus = "Disconnected"
    }

    fun sendUdpInput(msgType: Byte, code: Byte, value: Short) {
        if (!isApproved || udpSocket == null || serverAddress == null) return

        GlobalScope.launch(Dispatchers.IO) {
            try {
                val deviceIdBytes = deviceId?.toByteArray() ?: ByteArray(0)
                val idLen = deviceIdBytes.size
                
                // Buffer size: 1 (id_len) + idLen + 4 (type, code, value_low, value_high)
                val buffer = ByteArray(1 + idLen + 4)
                buffer[0] = idLen.toByte()
                System.arraycopy(deviceIdBytes, 0, buffer, 1, idLen)
                
                val offset = 1 + idLen
                buffer[offset] = msgType
                buffer[offset + 1] = code
                buffer[offset + 2] = (value.toInt() and 0xFF).toByte()
                buffer[offset + 3] = ((value.toInt() shr 8) and 0xFF).toByte()

                val packet = DatagramPacket(buffer, buffer.size, serverAddress, 8001)
                udpSocket?.send(packet)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
}
