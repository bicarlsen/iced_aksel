# Triangle Implementation Summary

### 1. Inner Stroke Geometry

Standard vector stroking is centered on the path edge. To achieve an "Inner Stroke" (where the stroke stays inside the defined bounds), we must shrink the geometry.

- **Geometry:** Equilateral Triangle.
- **Relationship:** The Apothem (distance from center to edge) is exactly half the Radius (distance from center to vertex).
- **Inset Logic:** To shift the edge inward by `width / 2` (centering the stroke inside), we must reduce the Radius by `width`.
    - `Stroke_Radius = Original_Radius - Stroke_Width`

### 2. Miter Stability (Safety Measure)

When the stroke width is very large relative to the triangle size, the inner triangle becomes microscopic or negative.
- **The Problem:** Stroking a tiny path with a massive pen causes "Miter Blowout" (visual artifacts like spinning rectangles).
- **The Threshold:** If `Stroke_Width >= Original_Radius * 0.6` (slightly more than the apothem), the hole is mathematically closed.
- **The Fix:** We detect this condition and simply render a **Solid Filled Triangle** using the stroke color instead of attempting to tessellate a degenerate path.
    

### 3. Anti-Aliasing Polish

- **The Problem:** Tiny sub-pixel gaps between the Fill and the Inner Stroke due to tessellation mismatches.
- **The Fix:** We shrink the Fill radius slightly (by `min(0.5px, width/2)`) to tuck it safely underneath the stroke body.
