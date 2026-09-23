# Creating & Using Realistic 3D Cats in Blender

This guide explains how to create, download, customize, and export a realistic 3D cat model using **Blender** and use it directly inside the desktop cat widget.

---

## 1. How It Works

This project is built with **Bevy**, which has native, high-performance support for **glTF 2.0 / GLB** 3D models (`.glb` / `.gltf`).
Any 3D model exported from Blender as a `.glb` file will render with:
- Full PBR materials (realistic fur textures, normal maps, roughness, specular reflections)
- Skeletal rigs and bone hierarchies
- Baked vertex colors and lighting

The engine loads the model directly from:
`assets/models/cat.glb`

---

## 2. Where to Get Free Realistic 3D Cat Models

If you do not want to model a cat from scratch in Blender, you can download ready-made realistic cat models:

1. **[Sketchfab](https://sketchfab.com/search?q=cat+realistic&type=models&features=downloadable)**:
   - Filter by: **Downloadable** and **Free / CC License**.
   - Look for models tagged **Rigged** or **Animated** if you want bone movement.
   - Download in **glTF** or **FBX** format (FBX can be imported into Blender with one click).
2. **[BlenderKit](https://www.blenderkit.com/)**:
   - A free official add-on for Blender (`Edit` > `Preferences` > `Add-ons` > `BlenderKit`).
   - Search "Cat" inside Blender's 3D viewport and drag-and-drop realistic cats directly into your scene.
3. **[CGTrader](https://www.cgtrader.com/free-3d-models/animals/mammal/cat)**:
   - Filter by **Free** and search for realistic domestic shorthair, tabby, or fluffy cats.

---

## 3. Preparing the Model in Blender

Open Blender and import your cat model (`File` > `Import` > `.gltf` / `.fbx` / `.obj`).

### A. Origin & Grounding (Critical)
1. In the 3D viewport, place the cat so its **paws touch the ground plane** (`Z = 0`).
2. Move the model so it is centered at `X = 0` and `Y = 0`.
3. Select your cat mesh (and armature if rigged), press `Ctrl + A` and select **Apply All Transforms** (Location, Rotation, Scale).
4. Set the origin to 3D cursor at feet:
   - Press `Shift + S` > `Cursor to World Origin`.
   - Right-click model > `Set Origin` > `Origin to 3D Cursor`.

### B. Size & Scale
- In Blender's Scene Properties (right panel), verify **Units** is set to **Metric (Meters)**.
- A realistic adult cat is approximately:
  - **Height**: 0.25m - 0.35m (from paws to shoulders/ears)
  - **Length**: 0.45m - 0.60m (from chest to base of tail)
- If your downloaded model is too large or small, press `S` to scale it, then press `Ctrl + A` > **Apply Scale**.

### C. Orientation
- In Blender, have the cat's face point towards **-Y** (the standard Blender front view, accessible by pressing `Numpad 1`).
- When exported with `+Y Up`, the front of the cat will face `-Z` in glTF. The desktop widget automatically detects this and rotates the model 180° so it faces **directly out of your monitor at you**.

### D. Interactive Head Tracking (Armatures)
- If your cat model has a bone armature:
  - Rename the head bone to **`Head`** or **`head`** in the bone properties panel.
  - The app's `attach_head_to_gltf` system automatically detects this bone and makes the cat's head turn and track your mouse cursor in real-time!

### E. Materials & Textures
- Use the **Principled BSDF** shader in Blender.
- Connect:
  - **Base Color** texture (the fur pattern / color map) to `Base Color`.
  - **Roughness** texture or set `Roughness` to ~`0.70` (matte fur).
  - **Normal Map** texture (for realistic fur clumps and depth) connected via a `Normal Map` node to `Normal`.

---

## 4. Exporting as glTF Binary (`.glb`)

Once your cat looks great in Blender:

1. Select your cat mesh (and its armature/skeleton if present).
2. Click **File** > **Export** > **glTF 2.0 (.glb/.gltf)**.
3. In the export settings sidebar on the right:
   - **Format**: `glTF Binary (.glb)` (packs all textures and geometry into a single portable file).
   - **Include**:
     - Check `Limit to: Selected Objects`.
   - **Transform**:
     - Check `+Y Up` (default).
   - **Geometry**:
     - Check `Apply Modifiers` (applies subdivision surface, mirror, etc.).
     - Check `Normals` and `Tangents`.
   - **Animation** (if your model has animations):
     - Check `Group by NLA Track`.
4. Name the file **`cat.glb`** and save it to:
   `assets/models/cat.glb`

---

## 5. Running & Using Your New Model

1. Drop your exported `cat.glb` into:
   ```
   cookie-cat/assets/models/cat.glb
   ```
2. Start the widget:
   ```bash
   cargo run
   ```
3. Your realistic 3D Blender cat will load immediately!

### Switching Between Models
- Press **`M`** on your keyboard anytime while the app is running to toggle between:
  - **Realistic 3D Blender Model** (`cat.glb`)
  - **Procedural Stylized Cat**

---

## 6. Tips for Best Realistic Results

- **PBR Fur**: Use normal maps. Normal maps give the impression of millions of individual hair strands without the performance cost of geometric hair curves.
- **Eye Reflections**: Keep the eyes as a separate material with `Roughness: 0.05` and `Specular: 0.9` for realistic glossy reflections under the studio lighting.
- **Lighting**: The app includes warm key light and cool rim light shaders configured specifically for animal silhouette rendering.
