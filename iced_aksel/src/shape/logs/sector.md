
### **Features Implemented**
1.  **Inner Stroke Alignment:**
    * Instead of centering the stroke on the edge (standard vector behavior), we mathematically shift the geometry inward.
    * **Outer Arc:** Radius reduced by `width / 2`.
    * **Inner Arc:** Radius increased by `width / 2`.
    * **Radial Lines:** Angles shifted inward based on `asin(offset / radius)` to ensure the outer edge of the stroke aligns with the defined start/end angles.

2.  **Dual Mode (Pie & Donut):**
    * Handles `inner_radius < 0.5` as a **Pie Slice** (connected to a single center point).
    * Handles `inner_radius >= 0.5` as a **Donut Slice** (two arcs connected by lines).

### **Errors & Artifacts Fixed**

1.  **Compilation Fix (Trait API):**
    * Removed the empty `&[]` attribute arguments from `builder.begin()`, `line_to()`, etc., matching your specific version of the `lyon` crate.

2.  **Safety 1: Radial Consumption:**
    * **Issue:** If the stroke was thicker than the radius (Pie) or the gap between rings (Donut), the math would produce negative radii.
    * **Fix:** Detected if `stroke_width >= shape_size`. If true, renders a **Solid Silhouette** instead of attempting to stroke.

3.  **Safety 2: Angular Consumption:**
    * **Issue:** On very thin slices, the angular shift required for the stroke could cross over, swapping the start and end angles.
    * **Fix:** Detected if `sweep_out <= 0`. If true, renders a solid silhouette.

4.  **Safety 3: Geometric Inversion (The "Star" Artifact):**
    * **Issue:** On small shapes with thick strokes, the "Virtual Center" (where radial lines meet to form a sharp corner) was pushed *outside* the arc radius to accommodate the stroke width. This caused the geometry to flip inside-out, creating a spiked/star artifact.
    * **Fix:** Added the check `if dist_shift >= s_out`. If the corner point is pushed beyond the outer boundary, we fall back to a solid silhouette.
