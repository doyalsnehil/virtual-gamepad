package com.example.virtualgamepad.ui.play
import androidx.compose.runtime.CompositionLocalProvider

import android.view.MotionEvent
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import android.content.pm.ActivityInfo
import android.app.Activity
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.virtualgamepad.data.NetworkManager
import kotlin.math.*

val BgColor = Color(0xFF0F1115)
val BtnBg = Color(0x0DFFFFFF) // rgba(255, 255, 255, 0.05)
val BtnBgActive = Color(0x33FFFFFF) // rgba(255, 255, 255, 0.2)
val BtnBorder = Color(0x1AFFFFFF) // rgba(255, 255, 255, 0.1)

val ColorA = Color(0xFF4CAF50)
val ColorB = Color(0xFFF44336)
val ColorX = Color(0xFF2196F3)
val ColorY = Color(0xFFFBC02D)

val LocalIsEditMode = androidx.compose.runtime.compositionLocalOf { false }

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun PlayScreen(serverUrl: String, layoutId: String, isEditMode: Boolean = false, onExit: () -> Unit) {
    val context = LocalContext.current
    val view = LocalView.current
    
    DisposableEffect(Unit) {
        val window = (context as? Activity)?.window
        if (window != null) {
            val insetsController = WindowCompat.getInsetsController(window, view)
            insetsController.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            insetsController.hide(WindowInsetsCompat.Type.systemBars())
        }
        val activity = context as? Activity
        val originalOrientation = activity?.requestedOrientation
        activity?.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        
        onDispose {
            if (window != null) {
                val insetsController = WindowCompat.getInsetsController(window, view)
                insetsController.show(WindowInsetsCompat.Type.systemBars())
            }
            if (originalOrientation != null) {
                activity.requestedOrientation = originalOrientation
            }
        }
    }
    
    fun sendButton(codeString: String, isPressed: Boolean) {
        if (isEditMode) return
        // msgType 0 is Key
        val codeByte = when (codeString) {
            "A" -> 0
            "B" -> 1
            "X" -> 2
            "Y" -> 3
            "LB" -> 4
            "RB" -> 5
            "START" -> 6
            "SELECT" -> 7
            "LS" -> 8
            "RS" -> 9
            "MODE" -> 10
            else -> return
        }
        NetworkManager.sendUdpInput(0.toByte(), codeByte.toByte(), if (isPressed) 1 else 0)
    }

    fun sendAxis(codeString: String, value: Short) {
        if (isEditMode) return
        // msgType 1 is Axis
        val codeByte = when (codeString) {
            "LX" -> 0
            "LY" -> 1
            "RX" -> 2
            "RY" -> 3
            "LT_ABS" -> 4
            "RT_ABS" -> 5
            "DX" -> 6
            "DY" -> 7
            else -> return
        }
        NetworkManager.sendUdpInput(1.toByte(), codeByte.toByte(), value)
    }

    var layout by remember { mutableStateOf(com.example.virtualgamepad.data.LayoutManager.getLayout(layoutId) ?: com.example.virtualgamepad.data.LayoutManager.createDefaultLayout(layoutId)) }
    var selectedComponentId by remember { mutableStateOf<String?>(null) }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(BgColor)
            .then(
                if (isEditMode) {
                    Modifier.pointerInput(Unit) {
                        detectTapGestures(onTap = { selectedComponentId = null })
                    }
                } else Modifier
            )
    ) {
        // Edit Mode Background
        if (isEditMode) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.5f)) // Dim the background
            ) {
                Text(
                    "Layout Editor Mode (Tap to select, drag to move)",
                    color = Color.White,
                    modifier = Modifier.align(Alignment.Center)
                )
            }
        }

        // Render Components Dynamically
        layout.components.forEachIndexed { index, comp ->
            DraggableComponent(
                state = comp,
                isEditMode = isEditMode,
                isSelected = isEditMode && selectedComponentId == comp.id,
                onSelect = { selectedComponentId = comp.id },
                onStateChange = { newState ->
                    val newComps = layout.components.toMutableList()
                    newComps[index] = newState
                    layout = layout.copy(components = newComps)
                }
            ) {
                when (comp.id) {
                    "shoulder_l" -> Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        GamepadShoulder(text = "LT", isTrigger = true) { sendAxis("LT_ABS", if (it) 255 else 0) }
                        GamepadShoulder(text = "LB", isTrigger = false) { sendButton("LB", it) }
                    }
                    "shoulder_r" -> Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        GamepadShoulder(text = "RB", isTrigger = false) { sendButton("RB", it) }
                        GamepadShoulder(text = "RT", isTrigger = true) { sendAxis("RT_ABS", if (it) 255 else 0) }
                    }
                    "menu" -> Row(
                        horizontalArrangement = Arrangement.spacedBy(24.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        GamepadMenuBtn(text = "Sel") { sendButton("SELECT", it) }
                        GamepadGuideBtn { sendButton("MODE", it) }
                        GamepadMenuBtn(text = "Str") { sendButton("START", it) }
                    }
                    "joystick_left" -> Joystick(onMove = { x, y ->
                        // Y axis inverted to match standard controllers. X axis is standard.
                        val absX = (x * 32767).toInt().toShort()
                        val absY = (y * 32767).toInt().toShort()
                        sendAxis("LX", absX)
                        sendAxis("LY", absY)
                    })
                    "joystick_right" -> Joystick(onMove = { x, y ->
                        val absX = (x * 32767).toInt().toShort()
                        val absY = (y * 32767).toInt().toShort()
                        sendAxis("RX", absX)
                        sendAxis("RY", absY)
                    })
                    "dpad" -> DPad(onAxis = { dx, dy ->
                        sendAxis("DX", dx)
                        sendAxis("DY", dy)
                    })
                    "abxy" -> ActionButtons(onButton = { key, pressed -> sendButton(key, pressed) })
                    "steering" -> AnalogSlider(SliderType.STEERING, label = "Steering") { value -> sendAxis("LX", value) }
                    "gas" -> AnalogSlider(SliderType.TRIGGER, label = "Gas") { value -> sendAxis("RT_ABS", value) }
                    "brake" -> AnalogSlider(SliderType.TRIGGER, label = "Brake") { value -> sendAxis("LT_ABS", value) }
                    "throttle" -> AnalogSlider(SliderType.THROTTLE, label = "Throttle") { value -> sendAxis("LY", value) }
                }
            }
        }

        // Save & Exit Button (Top Left Corner - Only in Edit Mode)
        if (isEditMode) {
            Button(
                onClick = {
                    com.example.virtualgamepad.data.LayoutManager.saveLayout(layout)
                    onExit()
                },
                modifier = Modifier.align(Alignment.TopStart).padding(16.dp),
                colors = ButtonDefaults.buttonColors(containerColor = Color.Red.copy(alpha = 0.8f))
            ) {
                Text("Save & Exit", color = Color.White, fontWeight = FontWeight.Bold)
            }
        }

        // Size Slider (Bottom Center)
        if (isEditMode && selectedComponentId != null) {
            val selectedIndex = layout.components.indexOfFirst { it.id == selectedComponentId }
            if (selectedIndex != -1) {
                val comp = layout.components[selectedIndex]
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 32.dp)
                        .background(Color(0xCC000000), RoundedCornerShape(16.dp))
                        .border(1.dp, Color.White.copy(alpha = 0.3f), RoundedCornerShape(16.dp))
                        .padding(horizontal = 24.dp, vertical = 8.dp)
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text("Size: ", color = Color.White, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.width(8.dp))
                        Slider(
                            value = comp.scale,
                            onValueChange = { newScale ->
                                val newComps = layout.components.toMutableList()
                                newComps[selectedIndex] = comp.copy(scale = newScale)
                                layout = layout.copy(components = newComps)
                            },
                            valueRange = 0.5f..3.0f,
                            modifier = Modifier.width(200.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun DraggableComponent(
    state: com.example.virtualgamepad.data.ComponentState,
    isEditMode: Boolean,
    isSelected: Boolean,
    onSelect: () -> Unit,
    onStateChange: (com.example.virtualgamepad.data.ComponentState) -> Unit,
    content: @Composable () -> Unit
) {
    val density = androidx.compose.ui.platform.LocalDensity.current
    val currentState by rememberUpdatedState(state)
    val currentOnStateChange by rememberUpdatedState(onStateChange)

    Box(
        modifier = Modifier
            .offset(x = state.x.dp, y = state.y.dp)
            .scale(state.scale)
            .then(
                if (isEditMode) {
                    Modifier
                        .pointerInput(Unit) {
                            detectTapGestures(onTap = { onSelect() })
                        }
                        .pointerInput(Unit) {
                            detectDragGestures { change, dragAmount ->
                                change.consume()
                                val dxDp = with(density) { (dragAmount.x * currentState.scale).toDp().value }
                                val dyDp = with(density) { (dragAmount.y * currentState.scale).toDp().value }
                                currentOnStateChange(currentState.copy(
                                    x = currentState.x + dxDp, 
                                    y = currentState.y + dyDp
                                ))
                            }
                        }
                        .border(if (isSelected) 3.dp else 2.dp, if (isSelected) Color.Green else Color(0xAAFFFF00), RoundedCornerShape(8.dp))
                        .padding(8.dp)
                } else Modifier
            )
    ) {
        // We capture touch events inside to prevent pointerInterop from eating the gesture during edit mode.
        Box(modifier = Modifier.then(if (isEditMode) Modifier.pointerInput(Unit) { awaitPointerEventScope { while(true) { awaitPointerEvent(); /* eat event */ } } } else Modifier)) {
            content()
        }
    }
}

enum class SliderType {
    STEERING, // Horizontal, center spring, maps to -32767..32767
    TRIGGER,  // Vertical, bottom spring, maps to 0..255
    THROTTLE  // Vertical, NO spring, maps to -32767..32767
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AnalogSlider(type: SliderType, label: String, onValue: (Short) -> Unit) {
    val isVertical = type != SliderType.STEERING
    var thumbPos by remember { mutableFloatStateOf(-1f) }
    var componentSize by remember { mutableStateOf(IntSize.Zero) }
    val length = 200.dp
    val thickness = 60.dp

    Box(
        modifier = Modifier
            .size(if (isVertical) thickness else length, if (isVertical) length else thickness)
            .onSizeChanged {
                componentSize = it
                if (thumbPos == -1f) {
                    val max = if (isVertical) it.height.toFloat() else it.width.toFloat()
                    thumbPos = when (type) {
                        SliderType.STEERING -> max / 2f
                        SliderType.TRIGGER -> max
                        SliderType.THROTTLE -> max
                    }
                }
            }
            .clip(RoundedCornerShape(30.dp))
            .background(Color(0x33FFFFFF))
            .border(2.dp, Color(0x33FFFFFF), RoundedCornerShape(30.dp))
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                val maxPos = if (isVertical) componentSize.height.toFloat() else componentSize.width.toFloat()
                if (maxPos == 0f) return@pointerInteropFilter false

                when (event.action) {
                    MotionEvent.ACTION_DOWN, MotionEvent.ACTION_MOVE -> {
                        val pos = if (isVertical) event.y else event.x
                        thumbPos = pos.coerceIn(0f, maxPos)

                        val value: Short = when (type) {
                            SliderType.STEERING -> {
                                val normalized = (thumbPos / maxPos) * 2f - 1f
                                (normalized * 32767).toInt().toShort()
                            }
                            SliderType.TRIGGER -> {
                                val normalized = 1f - (thumbPos / maxPos)
                                (normalized * 255).toInt().toShort()
                            }
                            SliderType.THROTTLE -> {
                                val normalized = 1f - (thumbPos / maxPos)
                                ((normalized * 2f - 1f) * -32767).toInt().toShort()
                            }
                        }
                        onValue(value)
                        true
                    }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                        when (type) {
                            SliderType.STEERING -> {
                                thumbPos = maxPos / 2f
                                onValue(0)
                            }
                            SliderType.TRIGGER -> {
                                thumbPos = maxPos
                                onValue(0)
                            }
                            SliderType.THROTTLE -> {}
                        }
                        true
                    }
                    else -> false
                }
            }
            }
,
        contentAlignment = Alignment.TopStart
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(label, color = Color.White.copy(alpha = 0.5f), fontWeight = FontWeight.Bold, fontSize = 12.sp)
        }

        if (thumbPos != -1f) {
            Box(
                modifier = Modifier
                    .offset {
                        if (isVertical) {
                            androidx.compose.ui.unit.IntOffset(0, (thumbPos - 30.dp.toPx()).roundToInt())
                        } else {
                            androidx.compose.ui.unit.IntOffset((thumbPos - 30.dp.toPx()).roundToInt(), 0)
                        }
                    }
                    .size(60.dp)
                    .clip(CircleShape)
                    .background(Color.White)
            )
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun Joystick(onMove: (Float, Float) -> Unit) {
    val radius = 150f
    var thumbPosition by remember { mutableStateOf(Offset.Zero) }
    var componentSize by remember { mutableStateOf(IntSize.Zero) }

    Canvas(
        modifier = Modifier
            .size(150.dp)
            .onSizeChanged { componentSize = it }
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                val canvasCenter = Offset(componentSize.width / 2f, componentSize.height / 2f)
                when (event.action) {
                    MotionEvent.ACTION_DOWN, MotionEvent.ACTION_MOVE -> {
                        val touch = Offset(event.x, event.y)
                        val delta = touch - canvasCenter
                        val distance = delta.getDistance()
                        
                        if (distance < radius) {
                            thumbPosition = touch
                        } else {
                            val ratio = radius / distance
                            thumbPosition = canvasCenter + (delta * ratio)
                        }
                        
                        val finalDelta = thumbPosition - canvasCenter
                        onMove(finalDelta.x / radius, finalDelta.y / radius)
                        true
                    }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                        thumbPosition = canvasCenter
                        onMove(0f, 0f)
                        true
                    }
                    else -> false
                }
            }
            }

    ) {
        val canvasCenter = Offset(size.width / 2f, size.height / 2f)
        if (thumbPosition == Offset.Zero) {
            thumbPosition = canvasCenter
        }
        
        // Base
        drawCircle(
            color = Color(0x05FFFFFF),
            radius = radius,
            center = canvasCenter
        )
        drawCircle(
            color = Color(0x1AFFFFFF),
            radius = radius,
            center = canvasCenter,
            style = Stroke(width = 2f)
        )
        
        // Thumb (solid white for nipplejs look)
        drawCircle(
            color = Color(0x80FFFFFF),
            radius = 50f,
            center = thumbPosition
        )
    }
}



@Composable
fun DPad(onAxis: (Short, Short) -> Unit) {

    var dx by remember { mutableStateOf<Short>(0) }
    var dy by remember { mutableStateOf<Short>(0) }
    
    fun updateAxis() {
        onAxis(dx, dy)
    }

    Box(
        modifier = Modifier
            .size(140.dp)
            .clip(CircleShape)
            .background(BtnBg)
    ) {

        // Up
        DPadBtn(modifier = Modifier.align(Alignment.TopCenter).size(50.dp, 40.dp).offset(y = 10.dp)) {
            dy = if (it) -32768 else 0; updateAxis()
        }
        // Down
        DPadBtn(modifier = Modifier.align(Alignment.BottomCenter).size(50.dp, 40.dp).offset(y = (-10).dp)) {
            dy = if (it) 32767 else 0; updateAxis()
        }
        // Left
        DPadBtn(modifier = Modifier.align(Alignment.CenterStart).size(40.dp, 50.dp).offset(x = 10.dp)) {
            dx = if (it) -32768 else 0; updateAxis()
        }
        // Right
        DPadBtn(modifier = Modifier.align(Alignment.CenterEnd).size(40.dp, 50.dp).offset(x = (-10).dp)) {
            dx = if (it) 32767 else 0; updateAxis()
        }
        
        // Center piece
        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .size(40.dp)
                .background(Color(0x1AFFFFFF))
        )
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DPadBtn(modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    Box(
        modifier = modifier
            .scale(scale)
            .clip(RoundedCornerShape(5.dp))
            .background(if (isPressed) Color(0x994A90E2) else Color(0x1AFFFFFF)) // Light blue when pressed
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
    )
}

@Composable
fun ActionButtons(onButton: (String, Boolean) -> Unit) {
    Box(modifier = Modifier.size(170.dp)) {
        GamepadBtn("Y", ColorY, Modifier.align(Alignment.TopCenter)) { onButton("Y", it) }
        GamepadBtn("A", ColorA, Modifier.align(Alignment.BottomCenter)) { onButton("A", it) }
        GamepadBtn("X", ColorX, Modifier.align(Alignment.CenterStart)) { onButton("X", it) }
        GamepadBtn("B", ColorB, Modifier.align(Alignment.CenterEnd)) { onButton("B", it) }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun GamepadBtn(text: String, textColor: Color, modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    val elevation by animateDpAsState(if (isPressed) 0.dp else 6.dp, tween(50), label = "elevation")
    
    Box(
        modifier = modifier
            .size(65.dp)
            .scale(scale)
            .shadow(elevation, CircleShape)
            .clip(CircleShape)
            .background(if (isPressed) textColor.copy(alpha = 0.25f) else BtnBg) // Glows with its own color
            .border(1.dp, if (isPressed) textColor.copy(alpha = 0.5f) else BtnBorder, CircleShape)
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
,
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.titleLarge.copy(
                fontWeight = FontWeight.ExtraBold,
                fontSize = 24.sp
            ),
            color = textColor
        )
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun GamepadShoulder(text: String, isTrigger: Boolean, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val height = if (isTrigger) 50.dp else 40.dp
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    val elevation by animateDpAsState(if (isPressed) 0.dp else 4.dp, tween(50), label = "elevation")
    
    Box(
        modifier = Modifier
            .size(80.dp, height)
            .scale(scale)
            .shadow(elevation, RoundedCornerShape(10.dp))
            .clip(RoundedCornerShape(10.dp))
            .background(if (isPressed) BtnBgActive else BtnBg)
            .border(1.dp, BtnBorder, RoundedCornerShape(10.dp))
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
,
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.ExtraBold),
            color = Color.White
        )
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun GamepadMenuBtn(text: String, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    
    Box(
        modifier = Modifier
            .size(40.dp, 25.dp)
            .scale(scale)
            .clip(RoundedCornerShape(12.dp))
            .background(if (isPressed) BtnBgActive else BtnBg)
            .border(1.dp, BtnBorder, RoundedCornerShape(12.dp))
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
,
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.ExtraBold, fontSize = 10.sp),
            color = Color.White
        )
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun GamepadGuideBtn(onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    val elevation by animateDpAsState(if (isPressed) 0.dp else 4.dp, tween(50), label = "elevation")
    
    Box(
        modifier = Modifier
            .size(50.dp)
            .scale(scale)
            .shadow(elevation, CircleShape)
            .clip(CircleShape)
            .background(if (isPressed) Color(0xFF111111) else Color(0xFF222222))
            .border(2.dp, Color(0xFF555555), CircleShape)
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
,
        contentAlignment = Alignment.Center
    ) {
        val glowColor = if (isPressed) Color(0xFF39FF14) else Color(0xEEFFFFFF) // Neon green or bright white
        Canvas(modifier = Modifier.fillMaxSize().padding(12.dp)) {
            val pathLeft = androidx.compose.ui.graphics.Path().apply {
                moveTo(size.width * 0.15f, size.height * 0.15f)
                quadraticTo(size.width * 0.6f, size.height * 0.5f, size.width * 0.15f, size.height * 0.85f)
            }
            val pathRight = androidx.compose.ui.graphics.Path().apply {
                moveTo(size.width * 0.85f, size.height * 0.15f)
                quadraticTo(size.width * 0.4f, size.height * 0.5f, size.width * 0.85f, size.height * 0.85f)
            }

            // Outer blur/glow
            drawPath(pathLeft, color = glowColor.copy(alpha = 0.4f), style = Stroke(width = 12f, cap = androidx.compose.ui.graphics.StrokeCap.Round))
            drawPath(pathRight, color = glowColor.copy(alpha = 0.4f), style = Stroke(width = 12f, cap = androidx.compose.ui.graphics.StrokeCap.Round))
            
            // Inner crisp line
            drawPath(pathLeft, color = glowColor, style = Stroke(width = 5f, cap = androidx.compose.ui.graphics.StrokeCap.Round))
            drawPath(pathRight, color = glowColor, style = Stroke(width = 5f, cap = androidx.compose.ui.graphics.StrokeCap.Round))
        }
    }
}
