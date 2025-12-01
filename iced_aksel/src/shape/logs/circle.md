
### The Circle Implementation Log

1.  **Architecture (The Canvas):**
    * Moved from manual vertex generation to using **Lyon Tessellators** connected to our internal **MeshBuffer**.
    * Abstracted the complexity into a "Universal Adapter" (`tess.stroke_path`) that flattens curves into points, enabling unified support for **Dashed** and **Dotted** styles on any shape.

2.  **Coordinate Systems:**
    * Implemented dual-mode sizing: **`Length::Screen`** (fixed UI pixels, zoom-independent) vs **`Length::Plot`** (data units, zoom-dependent).
    * Solved aspect ratio distortion for plot-relative widths by using `min(dx, dy)` scaling logic.

3.  **Stroke Alignment (Inner Stroke):**
    * **Problem:** Standard vector strokes are centered on the edge, causing the shape to visually exceed its defined radius.
    * **Solution:** Implemented **Inner Alignment** by mathematically shrinking the stroke path radius by `width / 2.0`. This ensures the stroke stays strictly inside the user-defined bounds.

4.  **Visual Polish (Anti-Aliasing):**
    * **Problem:** "Bleeding" pixels where the Fill tessellation didn't perfectly overlap the Stroke tessellation due to different polygon approximations.
    * **Solution:** Applied a "Safety Shrink" to the **Fill Radius** (`stroke_width / 2.0` capped at `0.5px`). This tucks the fill safely under the stroke body, eliminating sub-pixel gaps without creating visible holes.

5. **Stroke dissapearing in size vs stroke width disparity**

* **Math:** If `Radius=6.5` and `Width=20`.
* **Inner Spine:** `6.5 - (20 / 2) = -3.5`.
* **Result:** The previous code checks `if stroke_radius > 0.1`. Since `-3.5` is not greater than `0.1`, it skips drawing the stroke entirely.

The fix is the same "Silhouette" logic we used for the Triangle: **If the stroke is so big it consumes the geometry, just draw a solid circle.**
