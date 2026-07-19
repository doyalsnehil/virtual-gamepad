package com.example.virtualgamepad.data

import android.content.Context
import android.content.SharedPreferences
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class ComponentState(
    val id: String, // e.g. "joystick_left", "dpad", "abxy", "steering", "gas", "brake"
    var x: Float, // X position in screen percentage (0.0 to 1.0) or exact pixels. Let's use exact dp or pixels for simplicity.
    var y: Float,
    var scale: Float = 1f
)

@Serializable
data class GamepadLayout(
    val id: String,
    var name: String,
    val type: String, // "default", "racing", "flight"
    val components: MutableList<ComponentState>
)

object LayoutManager {
    private var prefs: SharedPreferences? = null

    fun init(context: Context) {
        prefs = context.getSharedPreferences("LayoutPrefs", Context.MODE_PRIVATE)
        
        // Populate defaults if empty
        if (getAllLayouts().isEmpty()) {
            saveLayout(createDefaultLayout())
            saveLayout(createRacingLayout())
            saveLayout(createFlightLayout())
        }
    }

    fun saveLayout(layout: GamepadLayout) {
        val jsonString = Json.encodeToString(layout)
        prefs?.edit()?.putString("layout_${layout.id}", jsonString)?.apply()
    }

    fun getLayout(id: String): GamepadLayout? {
        val jsonString = prefs?.getString("layout_$id", null) ?: return null
        return try {
            Json.decodeFromString<GamepadLayout>(jsonString)
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    fun getAllLayouts(): List<GamepadLayout> {
        val allEntries = prefs?.all ?: return emptyList()
        val layouts = mutableListOf<GamepadLayout>()

        for ((key, value) in allEntries) {
            if (key.startsWith("layout_") && value is String) {
                try {
                    val layout = Json.decodeFromString<GamepadLayout>(value)
                    layouts.add(layout)
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }
        }

        // Force update or create the built-in templates to ensure they have the fixed coordinates
        var defaultLayout = layouts.find { it.id == "default_1" }
        if (defaultLayout == null) {
            defaultLayout = createDefaultLayout()
            layouts.add(0, defaultLayout)
            saveLayout(defaultLayout)
        }

        var racingLayout = layouts.find { it.id == "racing_1" }
        // If the racing layout has the old Y coordinate for steering (240f) or menu (350f instead of 376f), overwrite it with the fixed version
        if (racingLayout == null || racingLayout.components.find { it.id == "menu" }?.x != 376f) {
            layouts.remove(racingLayout)
            racingLayout = createRacingLayout()
            layouts.add(racingLayout)
            saveLayout(racingLayout)
        }

        var flightLayout = layouts.find { it.id == "flight_1" }
        if (flightLayout == null || flightLayout.components.find { it.id == "menu" }?.x != 376f) {
            layouts.remove(flightLayout)
            flightLayout = createFlightLayout()
            layouts.add(flightLayout)
            saveLayout(flightLayout)
        }

        return layouts.sortedBy { it.name }
    }

    fun deleteLayout(id: String) {
        prefs?.edit()?.remove("layout_$id")?.apply()
    }

    fun exportLayout(layout: GamepadLayout): String {
        val jsonString = Json.encodeToString(layout)
        return android.util.Base64.encodeToString(jsonString.toByteArray(), android.util.Base64.NO_WRAP)
    }

    fun importLayout(base64String: String): GamepadLayout? {
        return try {
            val jsonString = String(android.util.Base64.decode(base64String, android.util.Base64.DEFAULT))
            val layout = Json.decodeFromString<GamepadLayout>(jsonString)
            // Assign a new ID so it doesn't overwrite if imported on the same device
            val newLayout = layout.copy(id = java.util.UUID.randomUUID().toString())
            saveLayout(newLayout)
            newLayout
        } catch (e: Exception) {
            e.printStackTrace()
            null
        }
    }

    fun duplicateLayout(layout: GamepadLayout) {
        val newLayout = layout.copy(
            id = java.util.UUID.randomUUID().toString(),
            name = "${layout.name} (Copy)",
            components = layout.components.map { it.copy() }.toMutableList()
        )
        saveLayout(newLayout)
    }

    fun createDefaultLayout(id: String = "default_1"): GamepadLayout {
        return GamepadLayout(
            id = id,
            name = "Default Pad",
            type = "default",
            components = mutableListOf(
                ComponentState("shoulder_l", 48f, 16f, 1f),
                ComponentState("shoulder_r", 690f, 16f, 1f),
                ComponentState("menu", 376f, 16f, 1f),
                ComponentState("joystick_left", 32f, 120f, 1f),
                ComponentState("dpad", 220f, 240f, 1f),
                ComponentState("joystick_right", 510f, 240f, 1f),
                ComponentState("abxy", 712f, 140f, 1f)
            )
        )
    }

    fun createRacingLayout(id: String = "racing_1"): GamepadLayout {
        return GamepadLayout(
            id = id,
            name = "Racing Wheel",
            type = "racing",
            components = mutableListOf(
                ComponentState("steering", 64f, 200f, 1.5f), // Big wheel on left
                ComponentState("gas", 800f, 140f, 1.2f),
                ComponentState("brake", 650f, 140f, 1.2f),
                ComponentState("menu", 376f, 16f, 1f)
            )
        )
    }

    fun createFlightLayout(id: String = "flight_1"): GamepadLayout {
        return GamepadLayout(
            id = id,
            name = "Flight Stick",
            type = "flight",
            components = mutableListOf(
                ComponentState("throttle", 64f, 140f, 1.2f), // Left stick stays in place
                ComponentState("joystick_right", 500f, 180f, 1.5f), // Right stick for pitch/roll
                ComponentState("abxy", 712f, 140f, 1f),
                ComponentState("menu", 376f, 16f, 1f)
            )
        )
    }
}
