import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

# Fix Center Piece color
text = text.replace(
    ".background(Color(0x99000000))",
    ".background(Color(0x1AFFFFFF))"
)

# Fix DPadBtn color
text = text.replace(
    ".background(Color(0x99000000)) // Always visible dark color, glow is handled by DPad wedges",
    ".background(Color(0x33FFFFFF)) // Visible light-gray buttons on dark background"
)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
