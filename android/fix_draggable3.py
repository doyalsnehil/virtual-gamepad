import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

# We need to replace the entire DraggableComponent function to make it robust.
old_func_pattern = r'@Composable\nfun DraggableComponent\(.*?\}\n\}\n\}'

new_func = """@Composable
fun DraggableComponent(
    state: com.example.virtualgamepad.data.ComponentState,
    isEditMode: Boolean,
    isSelected: Boolean,
    onSelect: () -> Unit,
    onStateChange: (com.example.virtualgamepad.data.ComponentState) -> Unit,
    content: @Composable () -> Unit
) {
    val density = androidx.compose.ui.platform.LocalDensity.current
    val currentState by androidx.compose.runtime.rememberUpdatedState(state)
    val currentOnStateChange by androidx.compose.runtime.rememberUpdatedState(onStateChange)
    
    // Use local state for dragging to avoid stuttering due to async recomposition
    var offsetX by androidx.compose.runtime.remember(state.id) { androidx.compose.runtime.mutableFloatStateOf(state.x) }
    var offsetY by androidx.compose.runtime.remember(state.id) { androidx.compose.runtime.mutableFloatStateOf(state.y) }

    // Sync from state if it changes externally (but ignore small diffs to prevent drag loops)
    androidx.compose.runtime.LaunchedEffect(state.x, state.y) {
        if (kotlin.math.abs(offsetX - state.x) > 1f) offsetX = state.x
        if (kotlin.math.abs(offsetY - state.y) > 1f) offsetY = state.y
    }

    Box(
        modifier = Modifier
            .offset(x = offsetX.dp, y = offsetY.dp)
            .scale(state.scale)
            .then(
                if (isEditMode) {
                    Modifier
                        .pointerInput(Unit) {
                            androidx.compose.foundation.gestures.detectTapGestures(onTap = { onSelect() })
                        }
                        .pointerInput(Unit) {
                            androidx.compose.foundation.gestures.detectDragGestures { change, dragAmount ->
                                change.consume()
                                val dxDp = with(density) { (dragAmount.x * currentState.scale).toDp().value }
                                val dyDp = with(density) { (dragAmount.y * currentState.scale).toDp().value }
                                offsetX += dxDp
                                offsetY += dyDp
                                currentOnStateChange(currentState.copy(
                                    x = offsetX, 
                                    y = offsetY
                                ))
                            }
                        }
                        .border(if (isSelected) 3.dp else 2.dp, if (isSelected) androidx.compose.ui.graphics.Color.Green else androidx.compose.ui.graphics.Color(0xAAFFFF00), androidx.compose.foundation.shape.RoundedCornerShape(8.dp))
                        .padding(8.dp)
                } else Modifier
            )
    ) {
        // We capture touch events inside to prevent inner components (like Joystick) from eating the gesture during edit mode.
        Box(modifier = Modifier.then(if (isEditMode) Modifier.pointerInput(Unit) { awaitPointerEventScope { while(true) { val event = awaitPointerEvent(androidx.compose.ui.input.pointer.PointerEventPass.Initial); event.changes.forEach { it.consume() } } } } else Modifier)) {
            content()
        }
    }
}"""

text = re.sub(old_func_pattern, new_func, text, flags=re.DOTALL)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
