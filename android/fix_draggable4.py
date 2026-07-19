import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

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
    
    var offsetX by androidx.compose.runtime.remember(state.id) { androidx.compose.runtime.mutableFloatStateOf(state.x) }
    var offsetY by androidx.compose.runtime.remember(state.id) { androidx.compose.runtime.mutableFloatStateOf(state.y) }

    androidx.compose.runtime.LaunchedEffect(state.x, state.y) {
        if (kotlin.math.abs(offsetX - state.x) > 1f) offsetX = state.x
        if (kotlin.math.abs(offsetY - state.y) > 1f) offsetY = state.y
    }

    Box(
        modifier = Modifier
            .offset(x = offsetX.dp, y = offsetY.dp)
            .scale(state.scale)
            // Removed padding here so the hit area matches exactly the content size
    ) {
        content()

        if (isEditMode) {
            // Invisible overlay to intercept all gestures and show the selection border
            Box(
                modifier = Modifier
                    .matchParentSize()
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
            )
        }
    }
}"""

text = re.sub(old_func_pattern, new_func, text, flags=re.DOTALL)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
