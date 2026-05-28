from PIL import Image, ImageDraw, ImageFont
import os

sizes = [16, 32, 48, 64, 128, 256]
out_path = r"C:\Users\giris\Desktop\Odoo Tools\odoocare\src-tauri\icons\icon.ico"

images = []
for s in sizes:
    im = Image.new("RGBA", (s, s), (99, 102, 241))
    draw = ImageDraw.Draw(im)
    try:
        font = ImageFont.truetype("arial.ttf", int(s * 0.65))
        bbox = draw.textbbox((0, 0), "O", font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]
        draw.text(((s-tw)//2 - bbox[0], (s-th)//2 - bbox[1]), "O", font=font, fill="white")
    except Exception:
        margin = s // 5
        draw.ellipse([margin, margin, s-margin, s-margin], fill="white")
    images.append(im)

im0 = images[0]
im0.save(out_path, format="ICO", sizes=[(sz,sz) for sz in sizes], append_images=images[1:])
print(f"Written: {out_path} ({os.path.getsize(out_path)} bytes)")
