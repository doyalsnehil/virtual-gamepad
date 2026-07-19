import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

old_box = """    Box(
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
    ) {"""

new_box = """    CompositionLocalProvider(LocalIsEditMode provides isEditMode) {
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
        ) {"""

# Replace old_box with new_box
text = text.replace(old_box, new_box)

# Need to add a closing brace for CompositionLocalProvider at the end of the file.
# The Box is closed right before the end of PlayScreen.
# Wait, let's find the closing brace of the Box.
# It is better to just do this properly.
