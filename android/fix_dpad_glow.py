import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

wedges = """
val UpWedgeShape = androidx.compose.foundation.shape.GenericShape { size, _ ->
    moveTo(size.width * 0.5f, size.height * 0.5f)
    lineTo(0f, 0f)
    lineTo(size.width, 0f)
    close()
}
val DownWedgeShape = androidx.compose.foundation.shape.GenericShape { size, _ ->
    moveTo(size.width * 0.5f, size.height * 0.5f)
    lineTo(0f, size.height)
    lineTo(size.width, size.height)
    close()
}
val LeftWedgeShape = androidx.compose.foundation.shape.GenericShape { size, _ ->
    moveTo(size.width * 0.5f, size.height * 0.5f)
    lineTo(0f, 0f)
    lineTo(0f, size.height)
    close()
}
val RightWedgeShape = androidx.compose.foundation.shape.GenericShape { size, _ ->
    moveTo(size.width * 0.5f, size.height * 0.5f)
    lineTo(size.width, 0f)
    lineTo(size.width, size.height)
    close()
}

@Composable
fun DPad(onAxis: (Short, Short) -> Unit) {
"""

text = text.replace("@Composable\nfun DPad(onAxis: (Short, Short) -> Unit) {", wedges)

dpad_body_old = """    Box(
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
    }"""

dpad_body_new = """    Box(
        modifier = Modifier
            .size(140.dp)
            .clip(CircleShape)
            .background(BtnBg)
    ) {
        // Glows (behind buttons)
        val glowColor = Color(0x664A90E2)
        if (dy < 0) Box(modifier = Modifier.fillMaxSize().clip(UpWedgeShape).background(glowColor))
        if (dy > 0) Box(modifier = Modifier.fillMaxSize().clip(DownWedgeShape).background(glowColor))
        if (dx < 0) Box(modifier = Modifier.fillMaxSize().clip(LeftWedgeShape).background(glowColor))
        if (dx > 0) Box(modifier = Modifier.fillMaxSize().clip(RightWedgeShape).background(glowColor))

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
                .background(Color(0x99000000))
        )
    }"""

text = text.replace(dpad_body_old, dpad_body_new)

btn_old = """@Composable
fun DPadBtn(modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    Box(
        modifier = modifier
            .scale(scale)
            .clip(RoundedCornerShape(5.dp))
            .background(if (isPressed) Color(0xFF4A90E2) else Color(0x33000000))"""

btn_new = """@Composable
fun DPadBtn(modifier: Modifier = Modifier, onEvent: (Boolean) -> Unit) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.9f else 1f, tween(50), label = "scale")
    Box(
        modifier = modifier
            .scale(scale)
            .clip(RoundedCornerShape(5.dp))
            .background(Color(0x99000000)) // Always visible dark color, glow is handled by DPad wedges"""

text = text.replace(btn_old, btn_new)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
