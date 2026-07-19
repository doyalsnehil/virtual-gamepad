import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

old_draggable = r"""fun DraggableComponent\(
    state: com.example.virtualgamepad.data.ComponentState,
    isEditMode: Boolean,
    isSelected: Boolean,
    onSelect: \(\) -> Unit,
    onStateChange: \(com.example.virtualgamepad.data.ComponentState\) -> Unit,
    content: @Composable \(\) -> Unit
\) \{
    val density = androidx.compose.ui.platform.LocalDensity.current
    // state.x and state.y are stored as dp values to be screen-independent
    var offsetX by remember\(state.x\) \{ mutableFloatStateOf\(state.x\) \}
    var offsetY by remember\(state.y\) \{ mutableFloatStateOf\(state.y\) \}

    Box\(
        modifier = Modifier
            .offset\(x = offsetX.dp, y = offsetY.dp\)
            .scale\(state.scale\)
            .then\(
                if \(isEditMode\) \{
                    Modifier
                        .pointerInput\(Unit\) \{
                            detectTapGestures\(onTap = \{ onSelect\(\) \}\)
                        \}
                        .pointerInput\(Unit\) \{
                            detectDragGestures \{ change, dragAmount ->
                                change.consume\(\)
                                val dxDp = with\(density\) \{ dragAmount.x.toDp\(\).value \}
                                val dyDp = with\(density\) \{ dragAmount.y.toDp\(\).value \}
                                offsetX \+= dxDp
                                offsetY \+= dyDp
                                onStateChange\(state.copy\(x = offsetX, y = offsetY\)\)
                            \}
                        \}
                        .border\(if \(isSelected\) 3.dp else 2.dp, if \(isSelected\) Color.Green else Color\(0xAAFFFF00\), RoundedCornerShape\(8.dp\)\)
                        .padding\(8.dp\)
                \} else Modifier
            \)
    \) \{"""

new_draggable = """fun DraggableComponent(
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
    ) {"""

text = re.sub(old_draggable, new_draggable, text)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
