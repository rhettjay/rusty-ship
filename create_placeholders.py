from PIL import Image
import os

# Create portraits directory
os.makedirs("assets/portraits", exist_ok=True)
os.makedirs("assets/bosses", exist_ok=True)

# Portrait colors for each character
portraits = {
    "captain_davey_portscan": (200, 30, 30),    # Red
    "blowfish": (255, 140, 0),                   # Orange
    "twofish": (30, 100, 200),                   # Blue
    "rufus_reverse": (150, 30, 200),             # Purple
    "molly_hashpass": (255, 100, 200),           # Pink
    "deadbeef": (50, 200, 50),                   # Green
    "narrator": (100, 100, 100),                 # Gray
}

bosses = {
    "blowfish": (255, 140, 0),
    "twofish": (30, 100, 200),
    "rufus_reverse": (150, 30, 200),
    "molly_hashpass": (255, 100, 200),
    "captain_davey": (200, 30, 30),
    "deadbeef": (50, 200, 50),
}

# Create portrait images (128x128)
for name, color in portraits.items():
    img = Image.new('RGBA', (128, 128), color)
    img.save(f"assets/portraits/{name}.png")
    print(f"Created assets/portraits/{name}.png")

# Create boss sprite images (80x80 for most, 120x120 for captain)
for name, color in bosses.items():
    size = 120 if name == "captain_davey" else 80
    img = Image.new('RGBA', (size, size), color)
    img.save(f"assets/bosses/{name}.png")
    print(f"Created assets/bosses/{name}.png")

print("All placeholder images created!")
