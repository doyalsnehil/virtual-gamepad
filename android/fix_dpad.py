import re

with open("app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt", "r") as f:
    content = f.read()

# Remove the Wedge shapes
shapes_regex = r"val UpWedgeShape.*?fun DPad"
content = re.sub(shapes_regex, "fun DPad", content, flags=re.DOTALL)

# Replace the new DPad with the old DPad structure
old_dpad = """fun DPad(onAxis: (Short, Short) -> Unit) {
    var dx by remember { mutableStateOf<Short>(0) }
    var dy by remember { mutableStateOf<Short>(0) }
    
    fun updateAxis() {
        onAxis(dx, dy)
    }

    Box(
        modifier = Modifier
            .size(160.dp)
            .clip(CircleShape)
            .background(Color(0x1AFFFFFF))
            .border(2.dp, Color(0x33FFFFFF), CircleShape)
    ) {
        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .size(130.dp)
                .border(2.dp, Color(0x33FFFFFF), CircleShape)
        )
        DPadBtn(
            shape = UpWedgeShape,
            modifier = Modifier.align(Alignment.TopCenter).size(60.dp, 60.dp).offset(y = 15.dp)
        ) { dy = if (it) -32768 else 0; updateAxis() }
        DPadBtn(
            shape = DownWedgeShape,
            modifier = Modifier.align(Alignment.BottomCenter).size(60.dp, 60.dp).offset(y = (-15).dp)
        ) { dy = if (it) 32767 else 0; updateAxis() }
        DPadBtn(
            shape = LeftWedgeShape,
            modifier = Modifier.align(Alignment.CenterStart).size(60.dp, 60.dp).offset(x = 15.dp)
        ) { dx = if (it) -32768 else 0; updateAxis() }
        DPadBtn(
            shape = RightWedgeShape,
            modifier = Modifier.align(Alignment.CenterEnd).size(60.dp, 60.dp).offset(x = (-15).dp)
        ) { dx = if (it) 32767 else 0; updateAxis() }
    }
}"""

new_dpad = """fun DPad(onAxis: (Short, Short) -> Unit) {
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
                .background(Color(0x4D000000))
        )
    }
}"""

content = content.replace(old_dpad, new_dpad)

old_dpadbtn = """@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DPadBtn(shape: androidx.compose.ui.graphics.Shape, modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.95f else 1f, tween(50), label = "scale")
    Box(
        modifier = modifier
            .scale(scale)
            .clip(shape)
            .background(if (isPressed) Color(0xFF4A90E2) else Color(0x33000000)) // Visible translucent wedges
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }

    )
}"""

new_dpadbtn = """@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DPadBtn(modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    Box(
        modifier = modifier
            .scale(scale)
            .clip(RoundedCornerShape(5.dp))
            .background(if (isPressed) Color(0xFF4A90E2) else Color(0x33000000))
            .let { m -> if (LocalIsEditMode.current) m else m.pointerInteropFilter { event ->
                when (event.action) {
                    MotionEvent.ACTION_DOWN -> { isPressed = true; onEvent(true); true }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> { isPressed = false; onEvent(false); true }
                    else -> false
                }
            }
            }
    )
}"""

content = content.replace(old_dpadbtn, new_dpadbtn)

with open("app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt", "w") as f:
    f.write(content)

