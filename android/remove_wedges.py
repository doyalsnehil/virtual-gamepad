import re

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    text = f.read()

# Remove the wedge shapes
wedge_pattern = r'val UpWedgeShape =.*?val RightWedgeShape =.*?\n\}\n'
text = re.sub(wedge_pattern, '', text, flags=re.DOTALL)

# Remove the glowing wedge logic inside DPad
glow_logic_pattern = r'\s*// Glows \(behind buttons\)\s*val glowColor = Color\(0x664A90E2\)\s*if \(dy < 0\) Box\(modifier = Modifier\.fillMaxSize\(\)\.clip\(UpWedgeShape\)\.background\(glowColor\)\)\s*if \(dy > 0\) Box\(modifier = Modifier\.fillMaxSize\(\)\.clip\(DownWedgeShape\)\.background\(glowColor\)\)\s*if \(dx < 0\) Box\(modifier = Modifier\.fillMaxSize\(\)\.clip\(LeftWedgeShape\)\.background\(glowColor\)\)\s*if \(dx > 0\) Box\(modifier = Modifier\.fillMaxSize\(\)\.clip\(RightWedgeShape\)\.background\(glowColor\)\)'
text = re.sub(glow_logic_pattern, '', text, flags=re.DOTALL)

# Fix DPadBtn color to just be blue when pressed
text = text.replace(
    ".background(if (isPressed) androidx.compose.ui.graphics.Color.Transparent else Color(0x1AFFFFFF)) // Become transparent when pressed so the wedge glow shows perfectly",
    ".background(if (isPressed) Color(0x664A90E2) else Color(0x1AFFFFFF)) // Light blue when pressed"
)

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.write(text)
