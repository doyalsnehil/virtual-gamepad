with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'r') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if line.startswith('fun DraggableComponent('):
        insert_idx = i - 2
        lines.insert(insert_idx, "    }\n")
        break

with open('app/src/main/java/com/example/virtualgamepad/ui/play/PlayScreen.kt', 'w') as f:
    f.writelines(lines)
