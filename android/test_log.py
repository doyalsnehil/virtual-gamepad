import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

text = text.replace('val dyDp = with(density) { (dragAmount.y * currentState.scale).toDp().value }',
                    'val dyDp = with(density) { (dragAmount.y * currentState.scale).toDp().value }\n                                android.util.Log.d("DRAG_TEST", "Dragging! dx: $dxDp, currentScale: ${currentState.scale}, stateScale: ${state.scale}, newScale: ${currentState.copy(x = currentState.x + dxDp, y = currentState.y + dyDp).scale}")')

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
