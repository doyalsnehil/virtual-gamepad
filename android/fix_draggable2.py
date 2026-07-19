import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

# Fix the tap interceptor so it actually consumes the events, making the whole DraggableComponent clickable!
# The user said they had to click a "specific angle". This is because the content was eating the touches.
old_interceptor = r'Modifier\.pointerInput\(Unit\) \{ awaitPointerEventScope \{ while\(true\) \{ awaitPointerEvent\(\); /\* eat event \*/ \} \} \}'
new_interceptor = r'Modifier.pointerInput(Unit) { awaitPointerEventScope { while(true) { val event = awaitPointerEvent(androidx.compose.ui.input.pointer.PointerEventPass.Initial); event.changes.forEach { it.consume() } } } }'

text = re.sub(old_interceptor, new_interceptor, text)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
