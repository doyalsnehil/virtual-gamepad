import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

# Fix DPadBtn color when pressed
text = text.replace(
    ".background(Color(0x33FFFFFF)) // Visible light-gray buttons on dark background",
    ".background(if (isPressed) androidx.compose.ui.graphics.Color.Transparent else Color(0x33FFFFFF)) // Become transparent when pressed so the wedge glow shows perfectly"
)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
